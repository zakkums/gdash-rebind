//! Library for kmrebind (used by binary and benchmarks).

pub mod cli;
pub mod config;
pub mod device_discovery;
pub mod error;
pub mod event_loop;
pub mod key_mapper;
pub mod options;
pub mod uinput_emitter;
pub mod util;

pub use config::parse_key_names;
pub use error::Error;
pub use key_mapper::KeyMapper;
pub use options::RunOptions;

/// Run the remapper with the given options. Returns `Ok(())` on clean exit, `Err` on setup or runtime failure.
pub fn run(options: RunOptions) -> Result<(), Error> {
    util::init_log(options.verbose);
    log::info!("kmrebind starting...");

    let mapped_keys = options.mapped_keys()?;
    log::info!("Mapped keys: {:?} -> {:?}", options.keys, mapped_keys);

    let mut keyboard_device = device_discovery::select_keyboard_device(options.device_path.as_deref())?;
    let mut uinput_emitter = uinput_emitter::UInputEmitter::new(Some(&keyboard_device), options.dry_run)?;

    let shutdown: util::ShutdownFlag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    if let Err(e) = util::setup_signal_handlers(shutdown.clone()) {
        log::warn!("{}", Error::SignalHandler(e));
    }

    let mut key_mapper = key_mapper::KeyMapper::new(mapped_keys.clone());

    event_loop::run(
        &mut keyboard_device,
        &mut key_mapper,
        &mut uinput_emitter,
        &shutdown,
    )?;

    uinput_emitter.cleanup();
    log::info!("kmrebind stopped");
    Ok(())
}
