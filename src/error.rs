//! Application error type for kmrebind.

use std::fmt;

/// Errors that can occur during setup or runtime.
#[derive(Debug)]
pub enum Error {
    /// Requested device path does not exist.
    DeviceNotFound(String),
    /// Failed to open the input device.
    OpenDevice {
        path: String,
        source: std::io::Error,
    },
    /// No keyboard devices found (permissions or udev).
    NoKeyboards,
    /// UInput device creation failed (e.g. permission to /dev/uinput).
    UInputFailed(String),
    /// No valid keys specified (all names unknown or empty list).
    InvalidKeys,
    /// Failed to grab the keyboard device.
    GrabFailed(String),
    /// Failed to set up signal handlers.
    SignalHandler(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::DeviceNotFound(path) => write!(f, "Device path does not exist: {}", path),
            Error::OpenDevice { path, source } => {
                write!(f, "Failed to open {}: {}", path, source)
            }
            Error::NoKeyboards => write!(
                f,
                "No keyboard devices found. Check: groups (input), udev rules, --device /dev/input/eventX"
            ),
            Error::UInputFailed(msg) => write!(f, "UInput failed: {}", msg),
            Error::InvalidKeys => write!(f, "No valid keys specified"),
            Error::GrabFailed(msg) => write!(f, "Failed to grab device: {}", msg),
            Error::SignalHandler(e) => write!(f, "Could not set signal handlers: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::OpenDevice { source, .. } => Some(source),
            Error::SignalHandler(e) => Some(e),
            _ => None,
        }
    }
}
