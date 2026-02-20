//! Key mapping state machine with reference counting.
//! Optimized for low latency: bitset + refcount, no hashing or allocation in hot path.

use evdev::KeyCode;
use std::collections::HashSet;

const BITSET_LEN: usize = 16; // 1024 bits, covers Linux KEY_MAX
const MAX_KEY: u16 = (BITSET_LEN * 64) as u16;

#[inline(always)]
fn bitset_get(bitset: &[u64; BITSET_LEN], code: u16) -> bool {
    (code as usize) < MAX_KEY as usize
        && (bitset[code as usize / 64] & (1u64 << (code % 64))) != 0
}

#[inline(always)]
fn bitset_set(bitset: &mut [u64; BITSET_LEN], code: u16) {
    if (code as usize) < MAX_KEY as usize {
        bitset[code as usize / 64] |= 1u64 << (code % 64);
    }
}

#[inline(always)]
fn bitset_clear(bitset: &mut [u64; BITSET_LEN], code: u16) {
    if (code as usize) < MAX_KEY as usize {
        bitset[code as usize / 64] &= !(1u64 << (code % 64));
    }
}

/// Manages the state machine for mapping keyboard keys to mouse button.
/// Reference counting: mouse button stays pressed until all mapped keys are released.
pub struct KeyMapper {
    /// Which key codes are mapped (bitset, O(1) lookup).
    mapped: [u64; BITSET_LEN],
    /// Which mapped keys are currently pressed (bitset).
    pressed: [u64; BITSET_LEN],
    /// Number of mapped keys currently pressed.
    pressed_count: u32,
    mouse_button_state: bool,
    /// List of mapped KeyCodes (for get_active_keys and verbose logging).
    mapped_list: Vec<KeyCode>,
}

impl KeyMapper {
    pub fn new(mapped_keys: HashSet<KeyCode>) -> Self {
        let mut mapped = [0u64; BITSET_LEN];
        let mapped_list: Vec<KeyCode> = mapped_keys.iter().copied().collect();
        for &kc in &mapped_list {
            let code = kc.0;
            bitset_set(&mut mapped, code);
        }
        Self {
            mapped,
            pressed: [0u64; BITSET_LEN],
            pressed_count: 0,
            mouse_button_state: false,
            mapped_list,
        }
    }

    /// Returns true if this key code is one of the mapped keys (O(1) bitset lookup).
    #[inline(always)]
    pub fn is_mapped(&self, key_code: KeyCode) -> bool {
        let code = key_code.0;
        code < MAX_KEY && bitset_get(&self.mapped, code)
    }

    /// Process a key event. Returns true if mouse button state changed.
    /// Hot path: no global state, no allocation, no hashing.
    #[inline(always)]
    pub fn process_key_event(&mut self, key_code: KeyCode, is_press: bool) -> bool {
        let code = key_code.0;
        if code >= MAX_KEY || !bitset_get(&self.mapped, code) {
            return false;
        }
        let was_pressed = bitset_get(&self.pressed, code);
        let mut state_changed = false;

        if is_press {
            if !was_pressed {
                bitset_set(&mut self.pressed, code);
                self.pressed_count = self.pressed_count.saturating_add(1);
                if !self.mouse_button_state {
                    self.mouse_button_state = true;
                    state_changed = true;
                }
            }
        } else {
            if was_pressed {
                bitset_clear(&mut self.pressed, code);
                self.pressed_count = self.pressed_count.saturating_sub(1);
                if self.pressed_count == 0 && self.mouse_button_state {
                    self.mouse_button_state = false;
                    state_changed = true;
                }
            }
        }
        state_changed
    }

    #[inline(always)]
    pub fn get_mouse_button_state(&self) -> bool {
        self.mouse_button_state
    }

    #[allow(dead_code)]
    pub fn get_active_keys(&self) -> HashSet<KeyCode> {
        self.mapped_list
            .iter()
            .filter(|&&kc| bitset_get(&self.pressed, kc.0))
            .copied()
            .collect()
    }

    pub fn reset(&mut self) {
        self.pressed = [0u64; BITSET_LEN];
        self.pressed_count = 0;
        self.mouse_button_state = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use evdev::KeyCode;

    fn mapped_keys() -> HashSet<KeyCode> {
        [KeyCode::KEY_DOT, KeyCode::KEY_SLASH].into_iter().collect()
    }

    /// Single-key mode: only one key is mapped; press/release directly controls mouse button.
    #[test]
    fn single_key_mapping_only_one_key_mapped() {
        let one_key: HashSet<KeyCode> = [KeyCode::KEY_SPACE].into_iter().collect();
        let mut mapper = KeyMapper::new(one_key);
        assert!(mapper.is_mapped(KeyCode::KEY_SPACE));
        assert!(!mapper.is_mapped(KeyCode::KEY_DOT));
        assert!(mapper.process_key_event(KeyCode::KEY_SPACE, true));
        assert!(mapper.get_mouse_button_state());
        assert!(mapper.process_key_event(KeyCode::KEY_SPACE, false));
        assert!(!mapper.get_mouse_button_state());
    }

    #[test]
    fn single_key_press_release() {
        let mut mapper = KeyMapper::new(mapped_keys());
        assert!(mapper.process_key_event(KeyCode::KEY_DOT, true));
        assert!(mapper.get_mouse_button_state());
        assert_eq!(mapper.get_active_keys().len(), 1);
        assert!(mapper.process_key_event(KeyCode::KEY_DOT, false));
        assert!(!mapper.get_mouse_button_state());
        assert_eq!(mapper.get_active_keys().len(), 0);
    }

    #[test]
    fn two_keys_reference_counting() {
        let mut mapper = KeyMapper::new(mapped_keys());
        assert!(mapper.process_key_event(KeyCode::KEY_DOT, true));
        assert!(mapper.get_mouse_button_state());
        assert!(!mapper.process_key_event(KeyCode::KEY_SLASH, true));
        assert!(mapper.get_mouse_button_state());
        assert_eq!(mapper.get_active_keys().len(), 2);
        assert!(!mapper.process_key_event(KeyCode::KEY_DOT, false));
        assert!(mapper.get_mouse_button_state());
        assert_eq!(mapper.get_active_keys().len(), 1);
        assert!(mapper.process_key_event(KeyCode::KEY_SLASH, false));
        assert!(!mapper.get_mouse_button_state());
        assert_eq!(mapper.get_active_keys().len(), 0);
    }

    #[test]
    fn reverse_order_release() {
        let mut mapper = KeyMapper::new(mapped_keys());
        mapper.process_key_event(KeyCode::KEY_DOT, true);
        mapper.process_key_event(KeyCode::KEY_SLASH, true);
        assert!(mapper.get_mouse_button_state());
        assert!(!mapper.process_key_event(KeyCode::KEY_SLASH, false));
        assert!(mapper.get_mouse_button_state());
        assert!(mapper.process_key_event(KeyCode::KEY_DOT, false));
        assert!(!mapper.get_mouse_button_state());
    }

    #[test]
    fn rapid_press_release() {
        let mut mapper = KeyMapper::new(mapped_keys());
        mapper.process_key_event(KeyCode::KEY_DOT, true);
        mapper.process_key_event(KeyCode::KEY_SLASH, true);
        assert!(mapper.get_mouse_button_state());
        mapper.process_key_event(KeyCode::KEY_DOT, false);
        mapper.process_key_event(KeyCode::KEY_SLASH, false);
        assert!(!mapper.get_mouse_button_state());
    }

    #[test]
    fn unmapped_key_ignored() {
        let mut mapper = KeyMapper::new(mapped_keys());
        let initial = mapper.get_mouse_button_state();
        assert!(!mapper.process_key_event(KeyCode::KEY_A, true));
        assert_eq!(mapper.get_mouse_button_state(), initial);
    }

    #[test]
    fn reset() {
        let mut mapper = KeyMapper::new(mapped_keys());
        mapper.process_key_event(KeyCode::KEY_DOT, true);
        mapper.process_key_event(KeyCode::KEY_SLASH, true);
        assert!(mapper.get_mouse_button_state());
        mapper.reset();
        assert!(!mapper.get_mouse_button_state());
        assert_eq!(mapper.get_active_keys().len(), 0);
    }

    #[test]
    fn double_press_same_key() {
        let mut mapper = KeyMapper::new(mapped_keys());
        assert!(mapper.process_key_event(KeyCode::KEY_DOT, true));
        assert!(mapper.get_mouse_button_state());
        assert!(!mapper.process_key_event(KeyCode::KEY_DOT, true));
        assert!(mapper.get_mouse_button_state());
        assert_eq!(mapper.get_active_keys().len(), 1);
    }
}
