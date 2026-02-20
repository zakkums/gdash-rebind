//! Key name to key code parsing and default keys.

use evdev::KeyCode;
use std::collections::HashSet;

/// Parse key names (e.g. "KEY_DOT", "KEY_SLASH" or "DOT", "SLASH") into key codes.
/// Unknown names are skipped (with no effect).
pub fn parse_key_names(key_names: &[String]) -> HashSet<KeyCode> {
    let mut key_codes = HashSet::new();
    for name in key_names {
        let normalized = name.trim().trim_start_matches("KEY_");
        if let Some(code) = key_name_to_code(normalized) {
            key_codes.insert(code);
            log::debug!("Mapped {} -> {:?}", name, code);
        } else {
            log::warn!("unknown key name '{}', skipping", name);
        }
    }
    key_codes
}

/// Map a key name (without KEY_ prefix) to KeyCode. Covers common keys used by kmrebind.
fn key_name_to_code(name: &str) -> Option<KeyCode> {
    let code = match name.to_uppercase().as_str() {
        "DOT" => KeyCode::KEY_DOT,
        "SLASH" => KeyCode::KEY_SLASH,
        "SPACE" => KeyCode::KEY_SPACE,
        "ENTER" => KeyCode::KEY_ENTER,
        "A" => KeyCode::KEY_A,
        "B" => KeyCode::KEY_B,
        "C" => KeyCode::KEY_C,
        "D" => KeyCode::KEY_D,
        "E" => KeyCode::KEY_E,
        "F" => KeyCode::KEY_F,
        "G" => KeyCode::KEY_G,
        "H" => KeyCode::KEY_H,
        "I" => KeyCode::KEY_I,
        "J" => KeyCode::KEY_J,
        "K" => KeyCode::KEY_K,
        "L" => KeyCode::KEY_L,
        "M" => KeyCode::KEY_M,
        "N" => KeyCode::KEY_N,
        "O" => KeyCode::KEY_O,
        "P" => KeyCode::KEY_P,
        "Q" => KeyCode::KEY_Q,
        "R" => KeyCode::KEY_R,
        "S" => KeyCode::KEY_S,
        "T" => KeyCode::KEY_T,
        "U" => KeyCode::KEY_U,
        "V" => KeyCode::KEY_V,
        "W" => KeyCode::KEY_W,
        "X" => KeyCode::KEY_X,
        "Y" => KeyCode::KEY_Y,
        "Z" => KeyCode::KEY_Z,
        "ESC" => KeyCode::KEY_ESC,
        "TAB" => KeyCode::KEY_TAB,
        "LEFTSHIFT" => KeyCode::KEY_LEFTSHIFT,
        "RIGHTSHIFT" => KeyCode::KEY_RIGHTSHIFT,
        "LEFTCTRL" => KeyCode::KEY_LEFTCTRL,
        "RIGHTCTRL" => KeyCode::KEY_RIGHTCTRL,
        "LEFTALT" => KeyCode::KEY_LEFTALT,
        "RIGHTALT" => KeyCode::KEY_RIGHTALT,
        "CAPSLOCK" => KeyCode::KEY_CAPSLOCK,
        "1" => KeyCode::KEY_1,
        "2" => KeyCode::KEY_2,
        "3" => KeyCode::KEY_3,
        "4" => KeyCode::KEY_4,
        "5" => KeyCode::KEY_5,
        "6" => KeyCode::KEY_6,
        "7" => KeyCode::KEY_7,
        "8" => KeyCode::KEY_8,
        "9" => KeyCode::KEY_9,
        "0" => KeyCode::KEY_0,
        _ => return None,
    };
    Some(code)
}

/// Default keys to map: KEY_DOT and KEY_SLASH.
pub fn default_key_names() -> Vec<String> {
    vec!["KEY_DOT".into(), "KEY_SLASH".into()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_default_keys() {
        let names = vec!["KEY_DOT".to_string(), "KEY_SLASH".to_string()];
        let codes = parse_key_names(&names);
        assert_eq!(codes.len(), 2);
        assert!(codes.contains(&KeyCode::KEY_DOT));
        assert!(codes.contains(&KeyCode::KEY_SLASH));
    }

    #[test]
    fn default_key_names_parse_to_dot_slash() {
        let names = default_key_names();
        let codes = parse_key_names(&names);
        assert_eq!(codes.len(), 2);
        assert!(codes.contains(&KeyCode::KEY_DOT));
        assert!(codes.contains(&KeyCode::KEY_SLASH));
    }

    #[test]
    fn parse_without_prefix() {
        let names = vec!["DOT".to_string(), "SLASH".to_string()];
        let codes = parse_key_names(&names);
        assert_eq!(codes.len(), 2);
        assert!(codes.contains(&KeyCode::KEY_DOT));
        assert!(codes.contains(&KeyCode::KEY_SLASH));
    }

    #[test]
    fn parse_invalid_skipped() {
        let names = vec!["INVALID_KEY".to_string()];
        let codes = parse_key_names(&names);
        assert!(codes.is_empty());
    }

    #[test]
    fn parse_mixed_valid_invalid() {
        let names = vec![
            "KEY_DOT".to_string(),
            "NO_SUCH_KEY".to_string(),
            "KEY_SPACE".to_string(),
        ];
        let codes = parse_key_names(&names);
        assert_eq!(codes.len(), 2);
        assert!(codes.contains(&KeyCode::KEY_DOT));
        assert!(codes.contains(&KeyCode::KEY_SPACE));
    }

    #[test]
    fn parse_empty() {
        let codes = parse_key_names(&[]);
        assert!(codes.is_empty());
    }

    #[test]
    fn parse_case_insensitive_suffix() {
        // After "KEY_" prefix, suffix is matched case-insensitively (via to_uppercase).
        let names = vec!["KEY_dot".to_string(), "KEY_slash".to_string()];
        let codes = parse_key_names(&names);
        assert_eq!(codes.len(), 2);
        assert!(codes.contains(&KeyCode::KEY_DOT));
        assert!(codes.contains(&KeyCode::KEY_SLASH));
    }

    #[test]
    fn parse_trims_whitespace() {
        let names = vec!["  KEY_DOT  ".to_string(), "  SLASH  ".to_string()];
        let codes = parse_key_names(&names);
        assert_eq!(codes.len(), 2);
        assert!(codes.contains(&KeyCode::KEY_DOT));
        assert!(codes.contains(&KeyCode::KEY_SLASH));
    }
}
