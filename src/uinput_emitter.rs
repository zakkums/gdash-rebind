//! Uinput device creation and mouse/keyboard event emission.

use crate::error::Error;
use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, Device, InputEvent, KeyCode};

const EV_KEY: u16 = 1;

/// Manages uinput mouse and optional virtual keyboard for pass-through.
pub struct UInputEmitter {
    dry_run: bool,
    uinput_mouse: Option<VirtualDevice>,
    uinput_keyboard: Option<VirtualDevice>,
}

impl UInputEmitter {
    pub fn new(keyboard_device: Option<&Device>, dry_run: bool) -> Result<Self, Error> {
        let mut uinput_mouse = None;
        let mut uinput_keyboard = None;
        if !dry_run {
            let mouse = create_mouse_device()?;
            uinput_mouse = Some(mouse);
            if let Some(kb) = keyboard_device {
                if let Ok(kbd) = create_keyboard_device(kb) {
                    uinput_keyboard = Some(kbd);
                }
            }
        }
        Ok(Self {
            dry_run,
            uinput_mouse,
            uinput_keyboard,
        })
    }

    pub fn emit_button_press(&mut self) {
        if self.dry_run {
            eprintln!("[DRY RUN] Would emit: BTN_LEFT PRESS");
            return;
        }
        if let Some(ref mut dev) = self.uinput_mouse {
            let ev = InputEvent::new(EV_KEY, KeyCode::BTN_LEFT.0, 1);
            if let Err(e) = dev.emit(&[ev]) {
                eprintln!("error: Failed to emit button press: {}", e);
            }
        }
    }

    pub fn emit_button_release(&mut self) {
        if self.dry_run {
            eprintln!("[DRY RUN] Would emit: BTN_LEFT RELEASE");
            return;
        }
        if let Some(ref mut dev) = self.uinput_mouse {
            let ev = InputEvent::new(EV_KEY, KeyCode::BTN_LEFT.0, 0);
            if let Err(e) = dev.emit(&[ev]) {
                eprintln!("error: Failed to emit button release: {}", e);
            }
        }
    }

    pub fn emit_key_event(&mut self, key_code: KeyCode, value: i32) {
        if self.dry_run {
            eprintln!("[DRY RUN] Would emit key: {:?} = {}", key_code, value);
            return;
        }
        if let Some(ref mut dev) = self.uinput_keyboard {
            let ev = InputEvent::new(EV_KEY, key_code.0, value);
            if let Err(e) = dev.emit(&[ev]) {
                if crate::util::is_verbose() {
                    eprintln!("warning: Failed to pass through key {:?}: {}", key_code, e);
                }
            }
        }
    }

    pub fn cleanup(&mut self) {
        if self.uinput_mouse.take().is_some() {
            eprintln!("Closed uinput mouse device");
        }
        if self.uinput_keyboard.take().is_some() {
            eprintln!("Closed uinput keyboard device");
        }
    }
}

fn create_mouse_device() -> Result<VirtualDevice, Error> {
    let mut keys = AttributeSet::<KeyCode>::new();
    keys.insert(KeyCode::BTN_LEFT);
    VirtualDevice::builder()
        .map_err(|e: std::io::Error| Error::UInputFailed(e.to_string()))?
        .name("kmrebind-virtual-mouse")
        .with_keys(&keys)
        .map_err(|e: std::io::Error| Error::UInputFailed(e.to_string()))?
        .build()
        .map_err(|e: std::io::Error| Error::UInputFailed(e.to_string()))
}

fn create_keyboard_device(device: &Device) -> Result<VirtualDevice, Error> {
    let keys = device.supported_keys().ok_or_else(|| {
        Error::UInputFailed("Keyboard has no supported keys".to_string())
    })?;
    VirtualDevice::builder()
        .map_err(|e: std::io::Error| Error::UInputFailed(e.to_string()))?
        .name("kmrebind-virtual-keyboard")
        .with_keys(keys)
        .map_err(|e: std::io::Error| Error::UInputFailed(e.to_string()))?
        .build()
        .map_err(|e: std::io::Error| Error::UInputFailed(e.to_string()))
}
