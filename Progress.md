# LMBREBIND – Progress

Kernel-level keyboard→mouse remapper (Rust). This file tracks status and history.

---

## Status: **Complete**

- **Implementation:** Rust-only; Python code removed.
- **Build:** `make build` → `target/release/kmrebind`
- **Run:** `make run` or `./target/release/kmrebind`
- **Tests:** `make test` (cargo test, 7 unit tests)

---

## Module layout (Rust)

| Module | File | Role |
|--------|------|------|
| main | `src/main.rs` | Entrypoint |
| cli | `src/cli.rs` | Args, wiring, signal setup |
| config | `src/config.rs` | Key name → key code |
| device_discovery | `src/device_discovery.rs` | Keyboard detection |
| event_loop | `src/event_loop.rs` | Grab, read, dispatch, cleanup |
| key_mapper | `src/key_mapper.rs` | State machine + tests |
| uinput_emitter | `src/uinput_emitter.rs` | Virtual mouse/keyboard |
| util | `src/util.rs` | Verbosity, shutdown flag, signals |

---

## Cleanup (done)

- Removed Python package `src/kmrebind/`, tests `tests/`, `pyproject.toml`, `requirements.txt`, `scripts/`.
- Makefile and README are Rust-only.
- Systemd service uses Rust binary path.

---

## Behaviour

- Default: KEY_DOT, KEY_SLASH → BTN_LEFT.
- Reference counting; exclusive grab; pass-through for other keys.
- Graceful shutdown (SIGINT/SIGTERM); same udev rules.
