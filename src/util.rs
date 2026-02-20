//! Logging and signal handling.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

static VERBOSE: AtomicBool = AtomicBool::new(false);

/// Configure verbosity. When true, debug-level messages are shown.
pub fn set_verbose(verbose: bool) {
    VERBOSE.store(verbose, Ordering::Relaxed);
}

pub fn is_verbose() -> bool {
    VERBOSE.load(Ordering::Relaxed)
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
