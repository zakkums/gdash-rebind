# kmrebind

A kernel-level keyboard remapper for Linux that maps keyboard keys to mouse button clicks. Optimized for low latency and timing consistency, designed for rhythm games like Geometry Dash.

## What It Does

kmrebind reads raw keyboard events from `/dev/input/event*` and emits mouse button clicks via `/dev/uinput`. By default, it maps the `.` (dot) and `/` (slash) keys to the left mouse button (BTN_LEFT).

**Key Features:**
- Kernel-level operation (no X11/Wayland dependencies)
- Exclusive keyboard grab (mapped keys don't type into the system)
- Reference counting (both keys must be released before mouse button releases)
- Zero artificial delays (optimized for rhythm games)
- Automatic keyboard device detection
- Graceful shutdown with cleanup

## Safety Warnings

⚠️ **IMPORTANT:**
- This tool requires access to `/dev/input/event*` and `/dev/uinput`
- It will grab your keyboard exclusively while running
- The mapped keys (`.` and `/` by default) will NOT type into applications while the remapper is active
- Other keys should work normally, but the exclusive grab may interfere with some applications
- Always run in a terminal where you can easily stop it (Ctrl+C)
- Do NOT run as root unless absolutely necessary

## Requirements

- Linux with evdev/uinput (e.g. Debian or compatible distribution)
- Rust toolchain (e.g. [rustup](https://rustup.rs))
- Access to `/dev/input/event*` and `/dev/uinput` (udev rules + `input` group)

## Installation

### 1. Install Rust (if needed)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Setup udev Rules (Required for Non-Root Access)

```bash
sudo cp udev/99-kmrebind.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Add your user to the `input` group:

```bash
sudo usermod -aG input $USER
```

**You must log out and log back in** for the group membership to take effect.

### 3. Build

```bash
cd /path/to/LMBREBIND
make build
# Binary: ./target/release/kmrebind
```

## Usage

### Basic Usage

```bash
make run
# Or: ./target/release/kmrebind
```

Defaults: map KEY_DOT and KEY_SLASH to left mouse button.

### Command-Line Options

```bash
./target/release/kmrebind [OPTIONS]

Options:
  --keys KEY1 KEY2    Key names to map (default: KEY_DOT KEY_SLASH)
  --device PATH       Explicit keyboard device path (e.g., /dev/input/event3)
  --verbose           Enable verbose logging
  --dry-run           Print what would be emitted without creating uinput device
  -h, --help          Show help message
```

### Examples

```bash
# Default keys (KEY_DOT, KEY_SLASH)
./target/release/kmrebind

# Map different keys
./target/release/kmrebind --keys KEY_SPACE KEY_ENTER

# Specify keyboard device
./target/release/kmrebind --device /dev/input/event3

# Verbose / dry run
./target/release/kmrebind --verbose
./target/release/kmrebind --dry-run --verbose
```

## How to Identify Your Keyboard Device

```bash
# List input devices (evtest if installed)
evtest

# Or check device nodes
ls -la /dev/input/event*
```

The program auto-detects keyboards by default; use `--device` to override.

## How to Stop

Press **Ctrl+C**. The program will release the keyboard grab, release the mouse button if pressed, close uinput, and exit cleanly.

## Troubleshooting

### Permission Denied

1. Install udev rules and add your user to the `input` group (see Installation).
2. Log out and back in after adding the group.
3. Verify: `groups` (should include `input`), `ls -l /dev/input/event*` (group `input`).

### No Keyboard Device Found

1. Check devices: `ls /dev/input/event*`
2. Run with `--verbose` or specify `--device /dev/input/eventX`
3. Ensure read permission on the device.

### UInput Device Creation Failed

1. Check `/dev/uinput` exists: `ls -l /dev/uinput`
2. Ensure udev rules and `input` group.
3. Load module if needed: `sudo modprobe uinput`

### Keys Still Type / Other Keys Don't Work

- Only one instance should run; use `--verbose` or `--device` to confirm which device is used.
- Exclusive grab blocks all keys from the system; non-mapped keys are passed through via a virtual keyboard. If something still fails, try another device with `--device`.

## Running as a Systemd Service (Optional)

1. Edit `systemd/kmrebind.service` and set `ExecStart` to the full path of the binary, e.g.:
   `ExecStart=/path/to/LMBREBIND/target/release/kmrebind`
2. Copy to your user systemd directory:

```bash
mkdir -p ~/.config/systemd/user
cp systemd/kmrebind.service ~/.config/systemd/user/
```

3. Enable and start:

```bash
systemctl --user enable kmrebind.service
systemctl --user start kmrebind.service
systemctl --user status kmrebind.service
```

## Development

### Tests

```bash
make test
# Or: cargo test
```

- **Unit tests:** Key mapper state machine (7 tests), config key parsing (6 tests).
- **Integration tests:** CLI `--help`, invalid `--device`, invalid `--keys`, unknown flags (5 tests).

### Benchmarks (quality and delay)

```bash
make bench
# Or: cargo bench
```

Benchmarks measure the hot path (key_mapper) in release build:

| Benchmark | Typical result |
|-----------|----------------|
| `process_key_event` (press) | ~1.2 ns per call |
| `process_key_event` (release) | ~1.2 ns per call |
| Full cycle (press dot, slash, release both) | ~84 ns per “click” |

The hot path uses a bitset + refcount (no hashing or allocation). In-process handling is ~1–2 ns per key event; a full click (4 events) is ~84 ns. Kernel I/O (evdev read + uinput write) dominates real-world latency.

### Project Structure

```
LMBREBIND/
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── config.rs
│   ├── device_discovery.rs
│   ├── event_loop.rs
│   ├── key_mapper.rs
│   ├── uinput_emitter.rs
│   └── util.rs
├── udev/
│   └── 99-kmrebind.rules
├── systemd/
│   └── kmrebind.service
├── benches/
│   └── latency.rs       # Criterion benchmarks
├── tests/
│   └── cli_tests.rs     # Integration tests
├── Cargo.toml
├── Makefile
├── Progress.md
└── README.md
```

## Technical Details

- **Device discovery:** Scans `/dev/input/event*` for keyboards (EV_KEY + typical keys).
- **Exclusive grab:** Prevents mapped keys from typing.
- **Event loop:** Blocking read, reference-counted state machine, BTN_LEFT and pass-through via uinput.
- **Latency:** No sleeps or polling; direct evdev/uinput; minimal overhead.

## License

MIT License
