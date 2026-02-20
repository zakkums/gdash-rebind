//! Keyboard device discovery and selection.

use evdev::{Device, KeyCode};
use std::path::Path;
use std::path::PathBuf;

/// Find all input devices that appear to be keyboards (EV_KEY + typical keys).
pub fn find_keyboard_devices() -> Vec<(PathBuf, Device)> {
    let mut keyboards = Vec::new();
    for (path, device) in evdev::enumerate() {
        if is_likely_keyboard(&device) {
            keyboards.push((path, device));
        }
    }
    if keyboards.is_empty() {
        eprintln!("warning: list_devices returned no keyboards. Check permissions (input group, udev rules).");
    }
    keyboards
}

fn is_likely_keyboard(device: &Device) -> bool {
    let keys = match device.supported_keys() {
        Some(k) => k,
        None => return false,
    };
    let typical = [
        KeyCode::KEY_ESC,
        KeyCode::KEY_1,
        KeyCode::KEY_2,
        KeyCode::KEY_3,
        KeyCode::KEY_4,
        KeyCode::KEY_5,
        KeyCode::KEY_6,
        KeyCode::KEY_7,
        KeyCode::KEY_8,
        KeyCode::KEY_9,
        KeyCode::KEY_0,
        KeyCode::KEY_Q,
        KeyCode::KEY_W,
        KeyCode::KEY_E,
        KeyCode::KEY_TAB,
    ];
    typical.iter().any(|k| keys.contains(*k))
}

/// Select keyboard: by explicit path or auto-detect (first alphabetically by path).
pub fn select_keyboard_device(device_path: Option<&str>) -> Result<Device, String> {
    if let Some(path) = device_path {
        if !Path::new(path).exists() {
            return Err(format!("Device path does not exist: {}", path));
        }
        let device = Device::open(path).map_err(|e| format!("Failed to open {}: {}", path, e))?;
        eprintln!("Using specified device: {:?} ({})", device.name(), path);
        return Ok(device);
    }
    let mut keyboards = find_keyboard_devices();
    if keyboards.is_empty() {
        return Err(
            "No keyboard devices found. Check: groups (input), udev rules, --device /dev/input/eventX".into(),
        );
    }
    keyboards.sort_by(|a, b| a.0.cmp(&b.0));
    let (path, device) = keyboards.remove(0);
    eprintln!("Auto-selected keyboard: {:?} ({:?})", device.name(), path);
    if !keyboards.is_empty() {
        eprintln!("Found {} keyboard(s), using: {:?}", keyboards.len() + 1, path);
    }
    Ok(device)
}
