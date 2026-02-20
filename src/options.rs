//! Run options for the lib API (embedding or testing without CLI).

use std::collections::HashSet;

use evdev::KeyCode;

/// Options for running the remapper. Build from CLI args or use programmatically.
#[derive(Clone, Debug)]
pub struct RunOptions {
    /// Key names to map to left mouse button (e.g. ["KEY_DOT", "KEY_SLASH"]).
    pub keys: Vec<String>,
    /// Explicit keyboard device path, or None for auto-detect.
    pub device_path: Option<String>,
    /// If true, do not create uinput devices; only log what would be emitted.
    pub dry_run: bool,
    /// If true, enable debug-level logging (same as RUST_LOG=debug for kmrebind).
    pub verbose: bool,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            keys: vec!["KEY_DOT".into(), "KEY_SLASH".into()],
            device_path: None,
            dry_run: false,
            verbose: false,
        }
    }
}

impl RunOptions {
    /// Keys parsed to key codes. Returns error if no valid keys.
    pub fn mapped_keys(&self) -> Result<HashSet<KeyCode>, crate::error::Error> {
        let keys = crate::config::parse_key_names(&self.keys);
        if keys.is_empty() {
            return Err(crate::error::Error::InvalidKeys);
        }
        Ok(keys)
    }
}
