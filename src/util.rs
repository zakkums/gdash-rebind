//! Logging and signal handling.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Initialize logging. Call once at startup. If `verbose` is true, sets debug level for kmrebind.
pub fn init_log(verbose: bool) {
    let level = if verbose { log::LevelFilter::Debug } else { log::LevelFilter::Info };
    let _ = env_logger::Builder::from_default_env()
        .filter_module("kmrebind", level)
        .try_init();
}

/// True when debug-level logging is enabled (e.g. RUST_LOG=debug or --verbose).
pub fn is_verbose() -> bool {
    log::log_enabled!(log::Level::Debug)
}

/// Shutdown flag. Create in main, pass to setup_signal_handlers and event loop.
pub type ShutdownFlag = Arc<AtomicBool>;

pub fn should_shutdown(flag: &ShutdownFlag) -> bool {
    flag.load(Ordering::Relaxed)
}

/// Install SIGINT and SIGTERM handlers that set the given flag to true.
pub fn setup_signal_handlers(flag: ShutdownFlag) -> Result<(), std::io::Error> {
    signal_hook::flag::register(signal_hook::consts::signal::SIGINT, flag.clone())?;
    signal_hook::flag::register(signal_hook::consts::signal::SIGTERM, flag)?;
    Ok(())
}
