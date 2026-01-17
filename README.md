# CompressO CLI

Fast video compression CLI tool powered by FFmpeg.

> Fork of [CompressO](https://github.com/codeforreal1/compressO) - converted from GUI to command-line interface.

## Features

- Fast video compression with customizable quality
- Multiple presets: `thunderbolt` (fast) and `ironclad` (quality)
- Format conversion (mp4, mov, webm, avi, mkv)
- Video transformations (rotate, flip, crop)
- Resize and change FPS
- Progress bar with real-time feedback
- Cross-platform (Windows, macOS, Linux)

## Installation

### Prerequisites

- **FFmpeg** must be installed and available in your PATH
  - Windows: `winget install FFmpeg` or download from [ffmpeg.org](https://ffmpeg.org/download.html)
  - macOS: `brew install ffmpeg`
  - Linux: `sudo apt install ffmpeg` or equivalent

### Build from source

```bash
# Clone the repository
git clone https://github.com/VladimirPozdnyakov/compressO-CLI.git
cd compressO-CLI

# Build release version
cargo build --release

# Binary will be at target/release/compresso (or compresso.exe on Windows)
```

#### Windows Build Notes

On Windows, it's recommended to use the GNU toolchain to avoid conflicts with MSVC:

```powershell
# Install MSYS2 (if not installed)
winget install MSYS2.MSYS2

# Install MinGW-w64 GCC (run in MSYS2 terminal)
pacman -S mingw-w64-x86_64-gcc

# Add MinGW to PATH
$env:PATH = "C:\msys64\mingw64\bin;$env:PATH"

# Switch Rust to GNU toolchain
rustup default stable-x86_64-pc-windows-gnu

# Build
cargo build --release
```

Alternatively, you can use the MSVC toolchain if you have Visual Studio Build Tools installed with the C++ workload.

## Usage

```bash
compresso <INPUT> [OUTPUT] [OPTIONS]
```

### Basic Examples

```bash
# Simple compression (uses default settings)
compresso video.mp4

# Specify output file
compresso video.mp4 output.mp4

# Set quality (0-100, higher = better quality)
compresso video.mp4 -q 80

# Use high-quality preset (slower)
compresso video.mp4 -p ironclad

# Convert to different format
compresso video.mp4 output.webm -f webm
```

### Advanced Examples

```bash
# Resize video
compresso video.mp4 --width 1280 --height 720

# Change frame rate
compresso video.mp4 --fps 30

# Remove audio
compresso video.mp4 --mute

# Rotate video
compresso video.mp4 --rotate 90

# Flip video
compresso video.mp4 --flip-h  # horizontal
compresso video.mp4 --flip-v  # vertical

# Crop video (WxH:X:Y)
compresso video.mp4 --crop 1920x1080:0:0

# Combine options
compresso video.mp4 -q 75 -p ironclad --width 1280 --height 720 --fps 30 --mute -y

# Show video info without compressing
compresso video.mp4 --info
```

### Options

| Option | Short | Description |
|--------|-------|-------------|
| `--quality` | `-q` | Compression quality (0-100, default: 70) |
| `--preset` | `-p` | Preset: `thunderbolt` (fast) or `ironclad` (quality) |
| `--format` | `-f` | Output format: mp4, mov, webm, avi, mkv |
| `--width` | | Output video width |
| `--height` | | Output video height |
| `--fps` | | Output frame rate |
| `--mute` | | Remove audio |
| `--rotate` | | Rotate: 90, 180, 270, -90, -180, -270 |
| `--flip-h` | | Flip horizontally |
| `--flip-v` | | Flip vertically |
| `--crop` | | Crop: WxH:X:Y or W:H:X:Y |
| `--overwrite` | `-y` | Overwrite output without asking |
| `--verbose` | `-v` | Show verbose output |
| `--info` | | Show video info only |
| `--help` | `-h` | Show help |
| `--version` | `-V` | Show version |

## Output Example

```
  CompressO CLI v1.0.0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Video Information
──────────────────────────────
  File: video.mp4
  Size: 245.60 MB
  Duration: 00:05:32.45
  Resolution: 1920x1080
  Frame rate: 29.97 fps

Compression Settings
──────────────────────────────
  Input: video.mp4
  Output: video_compressed.mp4
  Preset: thunderbolt (fast)
  Quality: 70%

⠋ [00:01:23] [████████████████░░░░░░░░░░░░░░] 53% Compressing...

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✓ Compression complete!

  Original: 245.60 MB
  Compressed: 89.20 MB
  Saved: 156.40 MB (63.7%)
  Time: 155.32s

  Output: video_compressed.mp4
```

## Presets

### Thunderbolt (default)
- Fast compression
- Good quality for most use cases
- Uses `-c:v libx264 -crf <quality>`

### Ironclad
- Best quality output
- Slower compression
- Uses additional encoding options for maximum quality

## Quality Guide

| Quality | CRF | Use Case |
|---------|-----|----------|
| 100 | 24 | Near-lossless, large file |
| 80 | 26 | High quality, good compression |
| 70 | 28 | Balanced (default) |
| 50 | 30 | Smaller file, visible compression |
| 30 | 33 | Very small file, noticeable quality loss |

## License

[AGPL-3.0](LICENSE)

This software uses FFmpeg under the LGPLv2.1.

## Credits

- Original GUI app: [CompressO](https://github.com/codeforreal1/compressO) by codeforreal1
- CLI version by: Fox(ElectroNic)
