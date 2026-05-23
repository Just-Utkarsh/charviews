# charviews

> Live webcam feed as ASCII art — right in your terminal.

`charviews` is a fast, lightweight CLI tool written in Rust that captures your webcam in real time and renders it as ASCII art directly in the terminal. No GUI, no browser — just raw characters and your camera.

---

## Features

- **Live ASCII rendering** — streams frames from your webcam and draws them as ASCII art in real time
- **Native camera access** — uses `v4l2` (Video4Linux2) for direct, low-latency webcam capture on Linux
- **Terminal-native** — renders cleanly inside any terminal emulator using `crossterm`
- **Configurable via CLI** — camera index, resolution, and other options controlled through command-line flags (powered by `clap`)
- **Minimal dependencies** — small binary, no heavy runtime requirements

---

## Demo



https://github.com/user-attachments/assets/c18450a2-8eaf-477e-9dee-db4ff2b58830




---

## Requirements

- **Rust:** 1.70+ (2021 edition)
- A working webcam accessible at `/dev/video*`

---

## Installation

### Arch Linux — AUR (Recommended)

The easiest way to install on Arch Linux or any Arch-based distro (Manjaro, EndeavourOS, etc.):

```bash
yay -S charviews
```

This installs the `charviews` binary system-wide and keeps it updatable via your AUR helper.

---

### Build from Source

Clone the repo and build locally — no installation needed:

```bash
git clone https://github.com/Just-Utkarsh/charviews.git
cd charviews
cargo build --release
```

Then run it directly:

```bash
./target/release/charviews
```

Or move the binary to somewhere on your `$PATH` to use it anywhere:

```bash
sudo mv ./target/release/charviews /usr/local/bin/
charviews
```

---

## Usage

```bash
charviews [OPTIONS]
```

### Options

| Flag | Description |
|------|-------------|
| `-i`, `--index <N>` | Camera device index (default: `0`, i.e. `/dev/video0`) |
| `-h`, `--help` | Print help information |
| `-V`, `--version` | Print version |

### Examples

Use the default webcam:

```bash
charviews
```

Use a secondary camera (e.g. `/dev/video2`):

```bash
charviews --index 2
```

---

## How It Works

1. **Camera capture** — `nokhwa` opens the webcam via v4l2 and grabs frames.
2. **Grayscale mapping** — each pixel's brightness is mapped to an ASCII character from a density ramp (e.g. `@#S%;:,. `).
3. **Terminal rendering** — `crossterm` writes the frame to stdout, using cursor control to update in place each frame without flickering.

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| [`nokhwa`](https://crates.io/crates/nokhwa) | Webcam capture via v4l2 (native input feature) |
| [`crossterm`](https://crates.io/crates/crossterm) | Cross-platform terminal control |
| [`clap`](https://crates.io/crates/clap) | CLI argument parsing |
| [`anyhow`](https://crates.io/crates/anyhow) | Ergonomic error handling |

---

## Contributing

Contributions are welcome! Feel free to open issues or pull requests for:

- New features (e.g. color ASCII output, frame rate control, recording)
- Bug fixes
- Performance improvements
- Support for other platforms (macOS, Windows)

---

## License

Licensed under:

- [MIT License](./LICENSE-MIT)

at your option.

---
