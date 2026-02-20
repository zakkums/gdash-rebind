//! Command-line interface and main wiring.

use clap::Parser;
use crate::config::parse_key_names;
use crate::device_discovery::select_keyboard_device;
use crate::event_loop::run as event_loop_run;
use crate::key_mapper::KeyMapper;
use crate::uinput_emitter::UInputEmitter;
use crate::util;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[derive(clap::Parser)]
#[command(name = "kmrebind", about = "Remap keyboard keys to mouse button at kernel level")]
pub struct Args {
    #[arg(long, value_name = "KEY", num_args = 1.., default_values = ["KEY_DOT", "KEY_SLASH"])]
    pub keys: Vec<String>,

    #[arg(long, help = "Explicit keyboard device path (e.g. /dev/input/event3)")]
    pub device: Option<String>,

    #[arg(long)]
    pub verbose: bool,

    #[arg(long, help = "Print what would be emitted, no uinput device")]
    pub dry_run: bool,
}

pub fn run() -> i32 {
    let args = Args::parse();
    util::set_verbose(args.verbose);
    eprintln!("kmrebind starting...");

    let mapped_keys = parse_key_names(&args.keys);
    if mapped_keys.is_empty() {
        eprintln!("error: No valid keys specified");
        return 1;
    }
    eprintln!("Mapped keys: {:?} -> {:?}", args.keys, mapped_keys);

    let mut keyboard_device = match select_keyboard_device(args.device.as_deref()) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: {}", e);
            eprintln!("See README.md for udev rules and input group.");
            return 1;
        }
    };

    let mut uinput_emitter = match UInputEmitter::new(Some(&keyboard_device), args.dry_run) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("error: {}", e);
            eprintln!("Make sure you have permissions to /dev/uinput");
            return 1;
        }
    };

    let shutdown: util::ShutdownFlag = Arc::new(AtomicBool::new(false));
    if let Err(e) = util::setup_signal_handlers(shutdown.clone()) {
        eprintln!("warning: Could not set signal handlers: {}", e);
    }

    let mut key_mapper = KeyMapper::new(mapped_keys.clone());

    event_loop_run(
        &mut keyboard_device,
        &mut key_mapper,
        &mut uinput_emitter,
        &shutdown,
    );

    uinput_emitter.cleanup();
    key_mapper.reset();
    eprintln!("kmrebind stopped");
    0
}
