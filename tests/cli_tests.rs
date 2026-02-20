//! Integration tests: run the kmrebind binary and check exit codes and output.

use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_kmrebind"))
}

#[test]
fn cli_help_exits_zero() {
    let out = bin().arg("--help").output().unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("kmrebind"));
    assert!(stdout.contains("--keys"));
    assert!(stdout.contains("--device"));
    assert!(stdout.contains("--verbose"));
    assert!(stdout.contains("--dry-run"));
}

#[test]
fn cli_invalid_device_exits_nonzero() {
    let out = bin()
        .arg("--device")
        .arg("/dev/input/nonexistent_12345")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("does not exist") || stderr.contains("Failed to open"));
}

#[test]
fn cli_invalid_key_exits_nonzero() {
    let out = bin()
        .arg("--keys")
        .arg("NOT_A_REAL_KEY_NAME_XYZ")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("No valid keys"));
}

#[test]
fn cli_unknown_flag_exits_nonzero() {
    let out = bin().arg("--nonexistent").output().unwrap();
    assert!(!out.status.success());
}

#[test]
fn cli_default_keys_parsed() {
    // Just check that with default keys we get past parsing (will fail later on device open if no device)
    let out = bin().arg("--device").arg("/dev/input/nonexistent_99999").output().unwrap();
    // Should fail at device open, not at key parse
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Mapped keys") || stderr.contains("KEY_DOT") || stderr.contains("does not exist") || stderr.contains("Failed to open"));
}

#[test]
fn cli_single_key_accepted() {
    // Single key is valid; should get past key parsing and fail at device open
    let out = bin()
        .arg("--keys")
        .arg("KEY_SPACE")
        .arg("--device")
        .arg("/dev/input/nonexistent_99999")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Mapped keys") || stderr.contains("KEY_SPACE"));
    assert!(stderr.contains("does not exist") || stderr.contains("Failed to open"));
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn cli_dry_run_fails_at_device_not_at_uinput() {
    // With --dry-run we still need a keyboard device; should fail at device open, not at uinput
    let out = bin()
        .arg("--dry-run")
        .arg("--device")
        .arg("/dev/input/nonexistent_dryrun")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Mapped keys") || stderr.contains("KEY_DOT"));
    assert!(stderr.contains("does not exist") || stderr.contains("Failed to open"));
}
