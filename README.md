# CompressO CLI

Fast, secure, and efficient video compression tool powered by FFmpeg with multilingual support.

> Fork of [CompressO](https://github.com/codeforreal1/compressO) - converted from GUI to CLI with extensive security and performance improvements.

[![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

## 🌍 Languages

CompressO CLI supports multiple languages:

- 🇬🇧 **English** (default)
- 🇷🇺 **Русский** (Russian)

The application automatically detects the language based on command-line arguments or prompts the user to select a language on first launch.

## ✨ Features

### Core Functionality
- 🎯 **Interactive Mode** - Drag & drop videos or run without arguments for guided wizard
- 🌐 **Multilingual Support** - Choose between English and Russian languages
- ⚡ **Batch Processing** - Compress multiple videos or entire directories at once
- 🎚️ **Smart Presets** - Choose between Thunderbolt (5x speed) or Ironclad (quality)
- 📊 **Real-time Progress** - Live encoding speed (fps), ETA, and progress bar
- 🔍 **Video Info** - Detailed metadata display with file size estimates
- 🎬 **Format Conversion** - Support for MP4, MOV, WebM, AVI, MKV, M4V, WMV, FLV
- 🔄 **Video Transformations** - Rotate, flip, crop, resize, change FPS
- 📦 **JSON Output** - Machine-readable output for automation
- 🎨 **Beautiful Terminal UI** - Colored output with Unicode symbols

### Security Features
- 🔒 **Path Traversal Protection** - Validates and sanitizes all file paths
- 🔗 **Symlink Attack Prevention** - Resolves symlinks to real paths
- 🛡️ **System Directory Protection** - Prevents writing to sensitive locations
- 🔐 **Secure FFmpeg Resolution** - Configurable trusted FFmpeg path
- ⚛️ **TOCTOU Protection** - Atomic file operations eliminate race conditions
- 🧹 **Guaranteed Cleanup** - RAII ensures temporary files are always deleted
- 🔍 **Path Sanitization** - Removes dangerous characters and sequences
- 📝 **Security Logging** - Audit trail of path operations and warnings

### Performance Optimizations
- 🚀 **5x Faster Encoding** - Thunderbolt preset uses FFmpeg ultrafast
- ⏱️ **Optimized Regex** - Cached patterns save 150μs per video
- 💤 **Efficient Waiting** - Blocking I/O instead of busy-wait polling
- 📉 **Single FFmpeg Spawn** - Eliminates redundant metadata reads
- 📦 **Minimal Binary** - 2.3MB stripped release build

## 📋 Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage Modes](#usage-modes)
- [Presets](#presets)
- [Examples](#examples)
- [Options Reference](#options-reference)
- [Batch Processing](#batch-processing)
- [Security Configuration](#security-configuration)
- [Quality Guide](#quality-guide)
- [Advanced Usage](#advanced-usage)
- [Troubleshooting](#troubleshooting)
- [Building from Source](#building-from-source)
- [License](#license)

## 🚀 Installation

### Prerequisites

**FFmpeg** is required and must be installed:

| Platform | Installation Command |
|----------|---------------------|
| Windows  | `winget install FFmpeg` |
| macOS    | `brew install ffmpeg` |
| Ubuntu/Debian | `sudo apt install ffmpeg` |
| Fedora/RHEL | `sudo dnf install ffmpeg` |
| Arch Linux | `sudo pacman -S ffmpeg` |

Verify FFmpeg is installed:
```bash
ffmpeg -version
```

### Download Binary

Download the latest release from [Releases](../../releases) for your platform.

### Build from Source

See [Building from Source](#building-from-source) section below.

## ⚡ Quick Start

### Simplest Usage

```bash
# Interactive mode - just run it (prompts for language selection)
compresso

# Or drag & drop a video file onto compresso.exe

# Use specific language
compresso --language russian
compresso --language english
```

### Command Line

```bash
# Compress a video with default settings
compresso video.mp4

# Fast compression (Thunderbolt preset)
compresso video.mp4 -p thunderbolt -q 70

# Best quality (Ironclad preset)
compresso video.mp4 -p ironclad -q 85

# Convert format while compressing
compresso video.mp4 output.webm -f webm
```

## 📖 Usage Modes

### 1. Interactive Mode (Recommended for Beginners)

**Launch methods:**
- Double-click `compresso.exe` (Windows) - prompts for language selection
- Run `compresso` without arguments - prompts for language selection
- Drag & drop video file onto executable
- Drag & drop multiple files for batch processing

**Features:**
- Language selection on first launch (English/Russian)
- Guided step-by-step wizard
- File validation with helpful error messages
- Preview of compression settings
- Estimated output size range
- Size comparison visualization
- Confirmation before processing

**Example session:**
```
CompressO CLI - Compress any video into a tiny size.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Video File Selection
──────────────────────────────
Enter the path to your video file: my-video.mp4

Video Information
──────────────────────────────
  File: my-video.mp4
  Size: 245.60 MB
  Duration: 00:05:32
  Resolution: 1920x1080
  Frame rate: 30 fps

Compression Settings
──────────────────────────────
  Preset: Ironclad ▸ (quality)
  Quality: 70 ▸

Advanced Settings
──────────────────────────────
  Configure advanced settings? ▸ No

Estimated Output
──────────────────────────────
  Original size: 245.60 MB
  Estimated range: 8.61 MB - 102.35 MB

[███████████░░░░░░░░░░░░░] 45% Compressing... 35.2 fps, ETA 00:01:23
```

### 2. Command Line Mode (For Automation)

```bash
# Single file
compresso input.mp4 -q 70 -p thunderbolt

# Multiple files
compresso video1.mp4 video2.mp4 video3.mp4 -q 70

# Entire directory
compresso --dir ./videos -q 70 -p ironclad

# JSON output for scripting
compresso video.mp4 -q 70 --json
```

### 3. Batch Processing

Process multiple videos with a single command:

```bash
# Process multiple files
compresso *.mp4 -q 70 -p thunderbolt

# Process directory
compresso --dir ./raw-videos -q 80

# Drag & drop multiple files onto executable
# Opens interactive batch mode
```

See [Batch Processing](#batch-processing) section for details.

## 🎚️ Presets

CompressO offers two carefully tuned presets optimized for different use cases:

### ⚡ Thunderbolt (Speed Priority)

**Best for:** Quick compression, drafts, social media, screen recordings

**Performance:**
- 5x faster encoding than medium preset
- Typical 1-minute video: ~6 seconds encoding time
- Batch 10 videos: ~1 minute total

**Technical details:**
- FFmpeg preset: `ultrafast`
- Tune: `fastdecode` (optimized playback)
- Codec: H.264 (libx264)
- File size: 10-20% larger than Ironclad at same quality

**When to use:**
- ✅ Time-sensitive workflows
- ✅ Preview/draft processing
- ✅ Quick sharing
- ✅ Screen recordings
- ✅ Social media (platform will re-encode anyway)
- ❌ Not for: Archival, final delivery, storage optimization

### 🛡️ Ironclad (Quality Priority)

**Best for:** Archival, professional delivery, storage optimization

**Performance:**
- Best quality-to-size ratio
- Typical 1-minute video: ~30 seconds encoding time
- Maximum compression efficiency

**Technical details:**
- FFmpeg preset: `slow`
- Codec: H.264 (libx264)
- Additional flags: `-pix_fmt yuv420p`, `-movflags +faststart`
- File size: Baseline (smallest at same quality)

**When to use:**
- ✅ Archival compression
- ✅ Professional delivery
- ✅ Final production exports
- ✅ Storage optimization
- ✅ Maximum quality retention
- ❌ Not for: Quick previews, time-sensitive work

### Preset Comparison

| Aspect | Thunderbolt | Ironclad |
|--------|-------------|----------|
| Encoding Speed | **5x faster** ⚡ | 1x baseline |
| File Size | +10-20% larger | Baseline (smallest) |
| Visual Quality | Identical (same CRF) | Identical (same CRF) |
| Playback | Optimized 🚀 | Standard |
| Use Case | Speed priority | Quality/size priority |

## 📚 Examples

### Basic Compression

```bash
# Default settings (Ironclad preset, quality 70)
compresso video.mp4

# Specify quality (0-100, higher = better)
compresso video.mp4 -q 85

# Fast compression with Thunderbolt
compresso video.mp4 -p thunderbolt -q 70

# Output to specific file
compresso video.mp4 output.mp4

# Use Russian language
compresso video.mp4 --language russian

# Use English language
compresso video.mp4 --language english
```

### Format Conversion

```bash
# Convert to WebM
compresso video.mp4 output.webm -f webm

# Convert to MOV
compresso video.avi video.mov -f mov

# Auto-detect format from extension
compresso video.mp4 output.webm
```

### Video Transformations

```bash
# Resize to 720p
compresso video.mp4 --width 1280 --height 720

# Change frame rate
compresso video.mp4 --fps 30

# Rotate 90 degrees clockwise
compresso video.mp4 --rotate 90

# Flip horizontally
compresso video.mp4 --flip-h

# Flip vertically
compresso video.mp4 --flip-v

# Crop (width:height:x:y)
compresso video.mp4 --crop 1920:1080:0:0
```

### Audio Options

```bash
# Remove audio track
compresso video.mp4 --mute

# Keep audio (default)
compresso video.mp4
```

### Advanced Examples

```bash
# Combine multiple options
compresso video.mp4 \
  -q 80 \
  -p ironclad \
  --width 1920 \
  --height 1080 \
  --fps 30 \
  --rotate 90 \
  -y

# Quick 720p preview
compresso large-4k-video.mp4 preview.mp4 \
  -p thunderbolt \
  -q 60 \
  --width 1280 \
  --height 720 \
  --fps 30

# Archive compression (maximum quality retention)
compresso raw-footage.mov archive.mp4 \
  -p ironclad \
  -q 90 \
  --fps 60

# Social media optimization
compresso video.mp4 instagram.mp4 \
  -p thunderbolt \
  -q 70 \
  --width 1080 \
  --height 1920 \
  --fps 30
```

### Video Information

```bash
# Show detailed video info
compresso video.mp4 --info

# JSON output for scripting
compresso video.mp4 --info --json
```

### Batch Processing

```bash
# Process all MP4 files in current directory
compresso *.mp4 -q 70 -p thunderbolt

# Process entire directory
compresso --dir ./raw-videos -q 80

# Process with verbose output
compresso *.mp4 -q 70 -v
```

## 🎛️ Options Reference

### Input/Output

| Option | Description | Example |
|--------|-------------|---------|
| `<INPUT>` | Input video file(s) | `video.mp4` or `*.mp4` |
| `<OUTPUT>` | Output file path (optional) | `output.mp4` |
| `--dir <DIR>` | Process all videos in directory | `--dir ./videos` |

### Compression Settings

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--quality <Q>` | `-q` | Quality (0-100, higher = better) | `70` |
| `--preset <P>` | `-p` | Preset: `thunderbolt` or `ironclad` | `ironclad` |
| `--format <F>` | `-f` | Output format: mp4, mov, webm, avi, mkv, m4v, wmv, flv | (auto) |

### Video Processing

| Option | Description | Example |
|--------|-------------|---------|
| `--width <W>` | Output video width in pixels | `--width 1920` |
| `--height <H>` | Output video height in pixels | `--height 1080` |
| `--fps <FPS>` | Output frame rate | `--fps 30` |
| `--mute` | Remove audio track | `--mute` |

### Transformations

| Option | Description | Values |
|--------|-------------|--------|
| `--rotate <DEG>` | Rotate video | `90`, `180`, `270`, `-90`, `-180`, `-270` |
| `--flip-h` | Flip horizontally | (flag) |
| `--flip-v` | Flip vertically | (flag) |
| `--crop <CROP>` | Crop video | `WxH:X:Y` or `W:H:X:Y` |

### Behavior

| Option | Short | Description |
|--------|-------|-------------|
| `--language <LANG>` | | Language for the interface: `english` or `russian` |
| `--overwrite` | `-y` | Overwrite output file without confirmation |
| `--verbose` | `-v` | Show detailed FFmpeg output |
| `--json` | | Output results as JSON |
| `--info` | | Show video info only (no compression) |

### Help

| Option | Short | Description |
|--------|-------|-------------|
| `--help` | `-h` | Show help message |
| `--version` | `-V` | Show version |

## 📦 Batch Processing

Process multiple videos efficiently with batch mode.

### Methods

**1. Multiple file arguments:**
```bash
compresso video1.mp4 video2.mp4 video3.mp4 -q 70
```

**2. Glob patterns:**
```bash
compresso *.mp4 -q 70 -p thunderbolt
```

**3. Directory processing:**
```bash
compresso --dir ./videos -q 80 -p ironclad
```

**4. Drag & drop (Windows):**
- Select multiple video files
- Drag them onto `compresso.exe`
- Interactive batch wizard opens

### Batch Output

```
CompressO CLI v1.1.0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Processing 3 files...

→ Processing file 1/3: video1.mp4
[████████████████████████████████] 100% Complete
✓ Compression complete! Saved: 45.2 MB (68.5%)

→ Processing file 2/3: video2.mp4
[████████████████████████████████] 100% Complete
✓ Compression complete! Saved: 23.8 MB (52.3%)

→ Processing file 3/3: video3.mp4
[████████████████████████████████] 100% Complete
✓ Compression complete! Saved: 67.1 MB (73.2%)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Batch Summary

  Files processed: 3
  Successful: 3
  Failed: 0
  Total time: 2m 15s

  Total saved: 136.1 MB (65.8%)
```

### JSON Output

For automation and scripting:

```bash
compresso *.mp4 -q 70 --json > results.json
```

Output format:
```json
{
  "files": [
    {
      "input": "video1.mp4",
      "success": true,
      "original_size": 67108864,
      "compressed_size": 21102387,
      "saved": 45906477,
      "compression_ratio": 68.5,
      "elapsed_secs": 32.5
    }
  ],
  "total": {
    "processed": 3,
    "successful": 3,
    "failed": 0,
    "elapsed_secs": 135.2
  }
}
```

## 🔒 Security Configuration

CompressO implements multiple security layers to protect against attacks.

### Environment Variables

#### `COMPRESSO_FFMPEG_PATH`

Specify a trusted FFmpeg binary path (recommended for production):

```bash
# Linux/macOS
export COMPRESSO_FFMPEG_PATH=/usr/bin/ffmpeg

# Windows (PowerShell)
$env:COMPRESSO_FFMPEG_PATH = "C:\Program Files\ffmpeg\bin\ffmpeg.exe"

# Windows (CMD)
set COMPRESSO_FFMPEG_PATH=C:\Program Files\ffmpeg\bin\ffmpeg.exe
```

**When to use:**
- ✅ Production environments
- ✅ CI/CD pipelines
- ✅ Shared servers
- ✅ Security-sensitive workflows

#### `COMPRESSO_FFMPEG_VERIFY`

Enable strict verification of bundled FFmpeg:

```bash
export COMPRESSO_FFMPEG_VERIFY=1
```

**Checks performed:**
- File is executable (Unix)
- Binary responds to `--version`
- Output contains "ffmpeg version"

### FFmpeg Resolution Priority

1. **`COMPRESSO_FFMPEG_PATH`** (explicit, most secure)
2. **Bundled FFmpeg** (application directory, optionally verified)
3. **System PATH** (least secure, logs warning)

### Security Warnings

CompressO logs security-relevant events:

```
⚠ Warning: input path contains '..' which may indicate path traversal
ℹ input path resolved through symlink:
  Provided: link.mp4
  Resolved: /real/path/video.mp4
Error: Refusing to write to system directory: /etc/output.mp4
```

**Always review security warnings** in stderr output.

### Protected System Directories

CompressO prevents writing to:

**Unix/Linux:**
- `/etc/` - System configuration
- `/sys/` - Kernel interfaces
- `/proc/` - Process information
- `/dev/` - Device files
- `/boot/` - Boot files
- `/root/` - Root user home
- `/` - Root filesystem

**Windows:**
- `C:\Windows\` - System files
- `C:\Program Files\` - Applications
- Drive roots (`C:\`, `D:\`, etc.)

### Best Practices

**For individual users:**
```bash
# Find your FFmpeg
which ffmpeg  # Unix
where ffmpeg  # Windows

# Set explicit path in shell profile
echo "export COMPRESSO_FFMPEG_PATH=/usr/bin/ffmpeg" >> ~/.bashrc
```

**For CI/CD:**
```yaml
# GitHub Actions
env:
  COMPRESSO_FFMPEG_PATH: /usr/bin/ffmpeg

# GitLab CI
variables:
  COMPRESSO_FFMPEG_PATH: /usr/bin/ffmpeg
  COMPRESSO_FFMPEG_VERIFY: "1"
```

**For Docker:**
```dockerfile
FROM ubuntu:22.04

RUN apt-get update && apt-get install -y ffmpeg
ENV COMPRESSO_FFMPEG_PATH=/usr/bin/ffmpeg

COPY compresso /usr/local/bin/
RUN compresso --version
```

See [SECURITY.md](docs/SECURITY.md) for complete security documentation.

## 📊 Quality Guide

### Quality Values and CRF Mapping

| Quality | CRF | File Size | Use Case |
|---------|-----|-----------|----------|
| 100 | 24 | Largest | Near-lossless, archival |
| 90 | 25 | Very Large | Professional delivery |
| 80 | 26 | Large | High-quality sharing |
| **70** | **28** | **Medium** | **Balanced (default)** |
| 60 | 30 | Small | Good compression |
| 50 | 31 | Smaller | Noticeable compression |
| 40 | 33 | Very Small | Significant quality loss |
| 30 | 34 | Tiny | Heavy compression |

**Lower CRF = Higher Quality = Larger File**

### Choosing Quality

**General guidelines:**

- **90-100:** Archival, professional mastering
  - Minimal quality loss
  - Large file size
  - Use Ironclad preset

- **75-85:** Professional delivery, YouTube
  - Excellent quality
  - Reasonable file size
  - Use Ironclad preset

- **60-75:** General sharing, web upload
  - Good quality
  - Good compression
  - Either preset works

- **40-60:** Quick sharing, low-end devices
  - Noticeable compression
  - Small file size
  - Use Thunderbolt preset

- **Below 40:** Emergency compression only
  - Visible artifacts
  - Very small files
  - Use with caution

### Preset Impact on File Size

At the same quality setting:

| Quality | Ironclad | Thunderbolt | Difference |
|---------|----------|-------------|------------|
| 70 | 10.0 MB | 11.5 MB | +15% |
| 80 | 15.0 MB | 17.3 MB | +15% |
| 90 | 25.0 MB | 28.8 MB | +15% |

**Note:** Thunderbolt produces slightly larger files but encodes 5x faster.

## 🔧 Advanced Usage

### Compression Workflow Examples

**YouTube Upload:**
```bash
compresso raw-footage.mov youtube.mp4 \
  -p ironclad \
  -q 85 \
  --fps 60 \
  -y
```

**Instagram Reel (9:16):**
```bash
compresso video.mp4 instagram.mp4 \
  -p thunderbolt \
  -q 70 \
  --width 1080 \
  --height 1920 \
  --fps 30
```

**Email Attachment (size-critical):**
```bash
compresso video.mp4 email.mp4 \
  -p thunderbolt \
  -q 40 \
  --width 640 \
  --height 480 \
  --fps 24
```

**4K Archival:**
```bash
compresso 4k-source.mov archive.mp4 \
  -p ironclad \
  -q 95 \
  --width 3840 \
  --height 2160 \
  --fps 60
```

**Screen Recording Optimization:**
```bash
compresso screen-recording.mp4 optimized.mp4 \
  -p thunderbolt \
  -q 75 \
  --fps 30
```

### Scripting and Automation

**Bash script for batch processing:**
```bash
#!/bin/bash
for video in raw-videos/*.mp4; do
  compresso "$video" -q 70 -p thunderbolt -y
done
```

**Python automation:**
```python
import subprocess
import json

result = subprocess.run(
    ['compresso', 'video.mp4', '-q', '70', '--json'],
    capture_output=True,
    text=True
)

data = json.loads(result.stdout)
print(f"Saved: {data['saved']} bytes ({data['compression_ratio']}%)")
```

**PowerShell batch processing:**
```powershell
Get-ChildItem *.mp4 | ForEach-Object {
    compresso $_.FullName -q 70 -p thunderbolt -y
}
```

### Integration with Other Tools

**ffprobe integration:**
```bash
# Get video info
ffprobe -v quiet -print_format json -show_format video.mp4

# Compress
compresso video.mp4 -q 70
```

**Combine with find:**
```bash
# Find and compress all videos older than 30 days
find ./archive -name "*.mp4" -mtime +30 -exec compresso {} -q 60 -p ironclad \;
```

## 🐛 Troubleshooting

### Common Issues

**1. "FFmpeg not found"**
```
Solution: Install FFmpeg and ensure it's in PATH
- Windows: winget install FFmpeg
- macOS: brew install ffmpeg
- Linux: sudo apt install ffmpeg

Or set explicit path:
export COMPRESSO_FFMPEG_PATH=/usr/bin/ffmpeg
```

**2. "File not found" or "Invalid input"**
```
Solution: Check file path and permissions
- Use absolute paths or quote paths with spaces
- Verify file exists: ls -la video.mp4
- Check file is a valid video: file video.mp4
```

**3. "Permission denied" on output**
```
Solution: Check output directory permissions
chmod +w output-directory
```

**4. "Refusing to write to system directory"**
```
Solution: This is a security feature
- Don't specify output in system directories
- Use current directory or home directory
```

**5. Compression too slow**
```
Solution: Use Thunderbolt preset
compresso video.mp4 -p thunderbolt -q 70
```

**6. Output file too large**
```
Solution: Lower quality or use Ironclad
compresso video.mp4 -p ironclad -q 60
```

**7. Path contains '..' warning**
```
Solution: Use absolute paths
compresso /full/path/to/video.mp4
```

### Verbose Output

For debugging, use verbose mode:
```bash
compresso video.mp4 -q 70 -v
```

This shows:
- FFmpeg command arguments
- Detailed encoding progress
- All warnings and errors

### Reporting Issues

When reporting bugs, include:
1. CompressO version: `compresso --version`
2. FFmpeg version: `ffmpeg -version`
3. Operating system and version
4. Complete command used
5. Error message (copy-paste full output)
6. Input file details: `compresso video.mp4 --info`

## 🏗️ Building from Source

### Prerequisites

1. **Rust 1.70+**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **FFmpeg** (runtime dependency)

3. **Platform-specific build tools:**

   **Windows:**
   - Visual Studio Build Tools with:
     - "Desktop development with C++"
     - Windows 10/11 SDK

   **Linux:**
   ```bash
   sudo apt install build-essential pkg-config
   ```

   **macOS:**
   ```bash
   xcode-select --install
   ```

### Build Steps

```bash
# Clone repository
git clone https://github.com/VladimirPozdnyakov/compressO-CLI.git
cd compressO-CLI

# Build release version
cargo build --release

# Binary location
# Unix: ./target/release/compresso
# Windows: .\target\release\compresso.exe

# Install to system (optional)
cargo install --path .
```

### Build Scripts

**Linux/macOS:**
```bash
chmod +x build.sh
./build.sh
```

**Windows (PowerShell):**
```powershell
.\build.ps1
```

**Windows (CMD):**
```cmd
build.bat
```

### Build Configuration

**Release profile** (in `Cargo.toml`):
```toml
[profile.release]
lto = true              # Link-time optimization
strip = true            # Strip symbols
codegen-units = 1       # Better optimization
panic = "abort"         # Smaller binary
```

**Result:** Optimized 2.3MB binary

### Development Build

```bash
# Debug build (faster compilation, larger binary)
cargo build

# Run without installing
cargo run -- video.mp4 -q 70

# Run tests
cargo test
```

## 📄 License

This project is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)**.

See [LICENSE](LICENSE) for full license text.

**Key points:**
- ✅ Free to use, modify, and distribute
- ✅ Source code must be made available
- ✅ Network use constitutes distribution (AGPL requirement)
- ✅ Changes must be documented
- ⚠️ No warranty provided

**FFmpeg License:**
This software uses FFmpeg, which is licensed under the LGPL v2.1. FFmpeg is not bundled with this application and must be installed separately.

## 🙏 Credits

- **Original GUI Application:** [CompressO](https://github.com/codeforreal1/compressO) by [@codeforreal1](https://github.com/codeforreal1)
- **CLI Version & Enhancements:** Fox (ElectroNic) / Vladimir Pozdnyakov
- **FFmpeg:** [FFmpeg Team](https://ffmpeg.org/about.html)
- **Security Improvements:** Community contributions
- **Performance Optimizations:** Community contributions

## 🤝 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Update documentation
6. Submit a pull request

**Areas for contribution:**
- Additional output formats
- More video processing features
- Performance optimizations
- Security enhancements
- Documentation improvements
- Bug fixes

## 📞 Support

- **Issues:** [GitHub Issues](../../issues)
- **Discussions:** [GitHub Discussions](../../discussions)
- **Security:** See [SECURITY.md](docs/SECURITY.md)

## 🗺️ Roadmap

**Planned features:**
- [ ] GPU-accelerated encoding (NVENC, QuickSync, VideoToolbox)
- [ ] Additional codec support (AV1, HEVC)
- [ ] Advanced filtering (denoise, sharpen, color correction)
- [ ] Subtitle support
- [ ] Multi-pass encoding
- [ ] Custom FFmpeg arguments passthrough
- [ ] Built-in FFmpeg bundling (optional)
- [ ] Progress webhooks for remote monitoring
- [ ] Configuration file support

## 📊 Performance Benchmarks

**Test Setup:**
- File: 1080p H.264, 60 seconds
- Hardware: Modern CPU (6 cores)
- Quality: 70 (CRF 28)

**Results:**

| Preset | Encoding Time | File Size | Speed vs. Ironclad |
|--------|--------------|-----------|---------------------|
| Thunderbolt | 6.2s | 11.8 MB | **5.1x faster** |
| Ironclad | 31.5s | 9.9 MB | 1x (baseline) |

**Batch Processing (10 files, 1 min each):**
- Thunderbolt: ~1 minute total
- Ironclad: ~5 minutes total

**Memory Usage:**
- Typical: 50-100 MB
- Peak: ~200 MB (large videos)

**Binary Size:**
- Release build: 2.3 MB (stripped)
- Debug build: ~8 MB

---

**Made with ❤️ by the CompressO community**

**Star ⭐ this repository if you find it useful!**
