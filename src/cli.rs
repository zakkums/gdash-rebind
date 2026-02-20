//! Command-line interface and main wiring.

use crate::error::Error;
use clap::Parser;

#[derive(clap::Parser)]
#[command(name = "kmrebind", about = "Remap keyboard keys to mouse button at kernel level")]
pub struct Args {
    #[arg(
        long,
        value_name = "KEY",
        num_args = 1..,
        default_values = ["KEY_DOT", "KEY_SLASH"],
        help = "One or more key names to map to left mouse button (default: KEY_DOT KEY_SLASH)"
    )]
    pub keys: Vec<String>,

    #[arg(long, help = "Explicit keyboard device path (e.g. /dev/input/event3)")]
    pub device: Option<String>,

    #[arg(long)]
    pub verbose: bool,

    #[arg(long, help = "Print what would be emitted, no uinput device")]
    pub dry_run: bool,
}

fn print_error(e: &Error) {
    log::error!("{}", e);
    match e {
        Error::NoKeyboards | Error::DeviceNotFound(_) | Error::OpenDevice { .. } => {
            log::error!("See README.md for udev rules and input group.");
        }
        Error::UInputFailed(_) => {
            log::error!("Make sure you have permissions to /dev/uinput");
        }
        _ => {}
    }
}

pub fn run() -> i32 {
    let args = Args::parse();
    let options = crate::RunOptions {
        keys: args.keys,
        device_path: args.device,
        dry_run: args.dry_run,
        verbose: args.verbose,
    };
    match crate::run(options) {
        Ok(()) => 0,
        Err(e) => {
            print_error(&e);
            1
        }
    }
}
