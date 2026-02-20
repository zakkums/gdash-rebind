//! Main event loop: read keyboard events, drive key mapper, emit mouse/keyboard.
//! Uses poll() with a timeout so shutdown (Ctrl+C) is checked without waiting for a key.

use crate::error::Error;
use crate::key_mapper::KeyMapper;
use crate::uinput_emitter::UInputEmitter;
use crate::util;
use crate::util::ShutdownFlag;
use evdev::{Device, EventSummary};
use nix::errno::Errno;
use nix::poll::{poll, PollFd, PollFlags};
use std::os::fd::AsRawFd;

/// Poll timeout in ms; shutdown is checked at least this often.
const POLL_TIMEOUT_MS: u16 = 200;

/// Convert evdev key value to press/release. 0 = release, 1 = press, 2 = repeat (treat as press).
/// Returns None for other values (e.g. 3+).
pub(crate) fn key_value_to_press_release(value: i32) -> Option<bool> {
    match value {
        0 => Some(false),
        1 | 2 => Some(true),
        _ => None,
    }
}

pub fn run(
    keyboard_device: &mut Device,
    key_mapper: &mut KeyMapper,
    uinput_emitter: &mut UInputEmitter,
    shutdown: &ShutdownFlag,
) -> Result<(), Error> {
    keyboard_device.grab().map_err(|e| Error::GrabFailed(e.to_string()))?;
    let mut fetch_err = None::<std::io::Error>;
    loop {
        if util::should_shutdown(shutdown) {
            break;
        }
        let fd = unsafe { std::os::fd::BorrowedFd::borrow_raw(keyboard_device.as_raw_fd()) };
        let mut poll_fds = [PollFd::new(fd, PollFlags::POLLIN)];
        match poll(&mut poll_fds, POLL_TIMEOUT_MS) {
            Ok(0) => continue,
            Ok(_) => {
                if poll_fds[0].revents().is_some_and(|r| r.contains(PollFlags::POLLIN)) {
                    let events = match keyboard_device.fetch_events() {
                        Ok(iter) => iter,
                        Err(e) => {
                            fetch_err = Some(e);
                            break;
                        }
                    };
                    for event in events {
                        process_event(event, key_mapper, uinput_emitter);
                    }
                }
            }
            Err(e) => {
                // EINTR = interrupted by signal (e.g. Ctrl+C); treat as normal shutdown.
                if e == Errno::EINTR {
                    break;
                }
                fetch_err = Some(std::io::Error::other(format!("poll: {}", e)));
                break;
            }
        }
    }
    cleanup(keyboard_device, key_mapper, uinput_emitter);
    if let Some(e) = fetch_err {
        return Err(Error::GrabFailed(format!("{}", e)));
    }
    Ok(())
}

fn process_event(
    event: evdev::InputEvent,
    key_mapper: &mut KeyMapper,
    uinput_emitter: &mut UInputEmitter,
) {
    if let EventSummary::Key(_, key_code, value) = event.destructure() {
        let is_press = match key_value_to_press_release(value) {
            Some(p) => p,
            None => return,
        };
        match key_mapper.process_key_event(key_code, is_press) {
            Some(true) => {
                let pressed = key_mapper.get_mouse_button_state();
                if pressed {
                    uinput_emitter.emit_button_press();
                } else {
                    uinput_emitter.emit_button_release();
                }
                log::debug!("Mouse button: {}", if pressed { "PRESS" } else { "RELEASE" });
            }
            Some(false) => {}
            None => uinput_emitter.emit_key_event(key_code, value),
        }
    }
}

fn cleanup(
    keyboard_device: &mut Device,
    key_mapper: &mut KeyMapper,
    uinput_emitter: &mut UInputEmitter,
) {
    if let Err(e) = keyboard_device.ungrab() {
        log::warn!("Error releasing grab: {}", e);
    }
    log::info!("Released keyboard device grab");
    if key_mapper.get_mouse_button_state() {
        log::info!("Releasing mouse button on cleanup");
        uinput_emitter.emit_button_release();
    }
    key_mapper.reset();
}

#[cfg(test)]
mod tests {
    use super::key_value_to_press_release;

    #[test]
    fn key_value_parsing() {
        assert_eq!(key_value_to_press_release(0), Some(false));
        assert_eq!(key_value_to_press_release(1), Some(true));
        assert_eq!(key_value_to_press_release(2), Some(true));
        assert_eq!(key_value_to_press_release(3), None);
        assert_eq!(key_value_to_press_release(-1), None);
    }
}
