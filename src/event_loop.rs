//! Main event loop: read keyboard events, drive key mapper, emit mouse/keyboard.

use crate::key_mapper::KeyMapper;
use crate::uinput_emitter::UInputEmitter;
use crate::util;
use crate::util::ShutdownFlag;
use evdev::{Device, EventSummary};

pub fn run(
    keyboard_device: &mut Device,
    key_mapper: &mut KeyMapper,
    uinput_emitter: &mut UInputEmitter,
    shutdown: &ShutdownFlag,
) {
    eprintln!("Starting event loop...");
    eprintln!("Grabbing keyboard device exclusively");
    if let Err(e) = keyboard_device.grab() {
        eprintln!("error: Failed to grab device: {}", e);
        return;
    }
    loop {
        if util::should_shutdown(shutdown) {
            break;
        }
        let events = match keyboard_device.fetch_events() {
            Ok(iter) => iter,
            Err(e) => {
                eprintln!("error: fetch_events: {}", e);
                break;
            }
        };
        for event in events {
            if let EventSummary::Key(_, key_code, value) = event.destructure() {
                let is_press = value == 1 || value == 2;
                let is_release = value == 0;
                if !is_press && !is_release {
                    continue;
                }
                if key_mapper.is_mapped(key_code) {
                    let state_changed = key_mapper.process_key_event(key_code, is_press);
                    if state_changed {
                        let pressed = key_mapper.get_mouse_button_state();
                        if pressed {
                            uinput_emitter.emit_button_press();
                        } else {
                            uinput_emitter.emit_button_release();
                        }
                        if util::is_verbose() {
                            eprintln!("Mouse button: {}", if pressed { "PRESS" } else { "RELEASE" });
                        }
                    }
                } else {
                    uinput_emitter.emit_key_event(key_code, value);
                }
            }
        }
    }
    cleanup(keyboard_device, key_mapper, uinput_emitter);
}

fn cleanup(
    keyboard_device: &mut Device,
    key_mapper: &mut KeyMapper,
    uinput_emitter: &mut UInputEmitter,
) {
    if let Err(e) = keyboard_device.ungrab() {
        eprintln!("warning: Error releasing grab: {}", e);
    }
    eprintln!("Released keyboard device grab");
    if key_mapper.get_mouse_button_state() {
        eprintln!("Releasing mouse button on cleanup");
        uinput_emitter.emit_button_release();
    }
    key_mapper.reset();
}
