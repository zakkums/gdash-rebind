# LMBREBIND – Progress

Kernel-level keyboard→mouse remapper (Rust). This file tracks status, roadmap, and history.

---

## Status: **Stable (roadmap in progress)**

- **Implementation:** Rust-only; Python code removed.
- **Build:** `make build` → `target/release/kmrebind` (release: LTO, codegen-units=1, strip).
- **Run:** `make run` or `./target/release/kmrebind`
- **Tests:** `make test` (23 tests: 16 unit + 7 integration).
- **Hot path:** Bitset + refcount in key_mapper; no HashSet in event loop; no global state in `process_key_event`.

---

## Module layout (Rust)

| Module | File | Role |
|--------|------|------|
| main | `src/main.rs` | Entrypoint |
| cli | `src/cli.rs` | Args, wiring, signal setup |
| config | `src/config.rs` | Key name → key code |
| device_discovery | `src/device_discovery.rs` | Keyboard detection |
| error | `src/error.rs` | Application error type |
| event_loop | `src/event_loop.rs` | Grab, poll, read, dispatch, cleanup |
| key_mapper | `src/key_mapper.rs` | State machine + tests |
| options | `src/options.rs` | RunOptions for lib API |
| uinput_emitter | `src/uinput_emitter.rs` | Virtual mouse/keyboard |
| util | `src/util.rs` | Verbosity, shutdown flag, signals |

---

## Development roadmap

Items below are ordered by impact and dependency. Check off as done.

### 1. Error handling & types

- [x] **1.1** Add `src/error.rs`: define `Error` enum (`DeviceNotFound`, `UInputFailed`, `NoKeyboards`, `InvalidKeys`, etc.) and implement `std::error::Error` + `Display`.
- [x] **1.2** Replace `Result<T, String>` with `Result<T, Error>` in public APIs (device_discovery, uinput_emitter, event_loop, cli).
- [x] **1.3** Manual impl; no extra dependencies.

### 2. Shutdown responsiveness

- [x] **2.1** Event loop: use `nix::poll` on keyboard device fd with 200 ms timeout.
- [x] **2.2** When poll returns (timeout or events), check shutdown flag; when POLLIN, call `fetch_events()`.
- [x] **2.3** evdev `Device::as_raw_fd()`; nix `poll` feature; `BorrowedFd::borrow_raw` for `PollFd::new`.

### 3. Logging

- [x] **3.1** Add `log` + `env_logger`; replace `eprintln!` and `util::set_verbose` with log levels.
- [x] **3.2** `--verbose` sets debug level for kmrebind; `RUST_LOG=debug` also works; `info`/`warn`/`error` for normal messages.
- [x] **3.3** `util::is_verbose()` now uses `log::log_enabled!(log::Level::Debug)`.

### 4. Config: key name coverage

- [ ] **4.1** Option A: Use crate `string_to_input_event_codes` (or similar) for string→key code if available and compatible.
- [ ] **4.2** Option B: Build a static map at startup from evdev `KeyCode` (e.g. iterate and use `Debug`/`format!("{:?}")` to get "KEY_DOT" and normalize).
- [ ] **4.3** Document supported key names in README; add test that default keys and a few extras parse correctly.

### 5. Tests & docs

- [x] **5.1** Unit test: `is_mapped_only_for_configured_keys`; `default_key_names_parse_to_dot_slash`.
- [x] **5.2** README: **Performance** subsection added.
- [x] **5.3** Integration test: `cli_dry_run_fails_at_device_not_at_uinput`.

### 6. API & modularization

- [x] **6.1** `RunOptions` in `src/options.rs`; `kmrebind::run(options) -> Result<(), Error>` in lib.
- [x] **6.2** `main` → `cli::run()` → parse args → build `RunOptions` → `kmrebind::run(options)`.

### 7. Cleanup & polish

- [x] **7.1** Removed `#[allow(dead_code)]` from `get_active_keys`, `default_key_names` (used in tests).
- [x] **7.2** `make clippy` runs `cargo clippy -- -D warnings`.
- [ ] **7.3** Optional: CHANGELOG.md for version history.

---

## Behaviour (current)

- **Keys:** One or more keys can be mapped to BTN_LEFT. Default is KEY_DOT + KEY_SLASH (two keys for rhythm-game use). With a single key, press = mouse down, release = mouse up. With multiple keys, reference counting: button stays down until all mapped keys are released.
- Exclusive grab; pass-through for other keys.
- Graceful shutdown (SIGINT/SIGTERM); same udev rules.
- Shutdown (Ctrl+C) is checked every 200 ms via `poll()` timeout, so exit does not require a key event.

---

## History / cleanup (done)

- Removed Python package `src/kmrebind/`, tests `tests/`, `pyproject.toml`, `requirements.txt`, `scripts/`.
- Makefile and README are Rust-only.
- Systemd service uses Rust binary path.
- Hot path: removed HashSet from event loop; added `KeyMapper::is_mapped`; moved verbose logging out of `process_key_event`; release profile (LTO, codegen-units=1, strip).
- Error type (`src/error.rs`); shutdown responsiveness via `nix::poll` (200 ms).
- Benchmarks: one key one click (~56 ns), two keys two independent clicks (~74 ns); no chord/rhythm benchmark.
- Logging: `log` + `env_logger`; `--verbose` or `RUST_LOG=debug`; `make clippy`.
- Lib API: `RunOptions`, `kmrebind::run(options)`; thin CLI.
