use clap::{Parser, ValueEnum};

use crate::domain::{CompressionConfig, CropCoordinates, FlipOptions, OutputFormat, Preset, VideoTransforms};

#[derive(Parser, Debug)]
#[command(
    name = "compresso",
    author = "CompressO Team (main), Fox(ElectroNic) (CLI-edition)",
    version,
    about = "Fast video compression CLI tool powered by FFmpeg",
    long_about = "CompressO CLI - Compress any video into a tiny size.\n\n\
                  Examples:\n  \
                  compresso video.mp4\n  \
                  compresso video.mp4 -q 80 -p ironclad\n  \
                  compresso video.mp4 output.webm -f webm\n  \
                  compresso video.mp4 --width 1280 --height 720 --fps 30"
)]
pub struct Cli {
    /// Input video file path(s) - can specify multiple files
    #[arg(required_unless_present = "dir")]
    pub input: Vec<String>,

    /// Process all videos in a directory
    #[arg(long, conflicts_with = "input")]
    pub dir: Option<String>,

    /// Output file path (only for single file, default: <input>_compressed.<ext>)
    #[arg()]
    pub output: Option<String>,

    /// Compression quality (0-100, higher = better quality, larger file)
    #[arg(short, long, default_value = "70", value_parser = clap::value_parser!(u8).range(0..=100))]
    pub quality: u8,

    /// Compression preset
    #[arg(short, long, value_enum, default_value = "thunderbolt")]
    pub preset: PresetArg,

    /// Output format (mp4, mov, webm, avi, mkv)
    #[arg(short, long)]
    pub format: Option<FormatArg>,

    /// Output video width
    #[arg(long)]
    pub width: Option<u32>,

    /// Output video height
    #[arg(long)]
    pub height: Option<u32>,

    /// Output video FPS (frames per second)
    #[arg(long)]
    pub fps: Option<u32>,

    /// Remove audio from video
    #[arg(long)]
    pub mute: bool,

    /// Rotate video (90, 180, 270, -90, -180, -270)
    #[arg(long, value_parser = parse_rotation)]
    pub rotate: Option<i32>,

    /// Flip video horizontally
    #[arg(long)]
    pub flip_h: bool,

    /// Flip video vertically
    #[arg(long)]
    pub flip_v: bool,

    /// Crop video (format: WxH:X:Y, e.g., 1920x1080:0:0)
    #[arg(long, value_parser = parse_crop)]
    pub crop: Option<CropCoordinates>,

    /// Overwrite output file without asking
    #[arg(short = 'y', long)]
    pub overwrite: bool,

    /// Show verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Show video info without compressing
    #[arg(long)]
    pub info: bool,

    /// Output results as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PresetArg {
    /// Fast compression with good quality
    Thunderbolt,
    /// Best quality, slower compression
    Ironclad,
}

impl From<PresetArg> for Preset {
    fn from(arg: PresetArg) -> Self {
        match arg {
            PresetArg::Thunderbolt => Preset::Thunderbolt,
            PresetArg::Ironclad => Preset::Ironclad,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FormatArg {
    Mp4,
    Mov,
    Webm,
    Avi,
    Mkv,
}

impl From<FormatArg> for OutputFormat {
    fn from(arg: FormatArg) -> Self {
        match arg {
            FormatArg::Mp4 => OutputFormat::Mp4,
            FormatArg::Mov => OutputFormat::Mov,
            FormatArg::Webm => OutputFormat::Webm,
            FormatArg::Avi => OutputFormat::Avi,
            FormatArg::Mkv => OutputFormat::Mkv,
        }
    }
}

fn parse_rotation(s: &str) -> Result<i32, String> {
    let angle: i32 = s.parse().map_err(|_| "Invalid rotation angle")?;
    match angle {
        90 | 180 | 270 | -90 | -180 | -270 => Ok(angle),
        _ => Err("Rotation must be 90, 180, 270, -90, -180, or -270".to_string()),
    }
}

fn parse_crop(s: &str) -> Result<CropCoordinates, String> {
    // Format: WxH:X:Y or W:H:X:Y
    let parts: Vec<&str> = s.split(':').collect();

    if parts.len() == 3 {
        // WxH:X:Y format
        let dims: Vec<&str> = parts[0].split('x').collect();
        if dims.len() != 2 {
            return Err("Crop format: WxH:X:Y or W:H:X:Y".to_string());
        }
        Ok(CropCoordinates {
            width: dims[0].parse().map_err(|_| "Invalid width")?,
            height: dims[1].parse().map_err(|_| "Invalid height")?,
            x: parts[1].parse().map_err(|_| "Invalid X offset")?,
            y: parts[2].parse().map_err(|_| "Invalid Y offset")?,
        })
    } else if parts.len() == 4 {
        // W:H:X:Y format
        Ok(CropCoordinates {
            width: parts[0].parse().map_err(|_| "Invalid width")?,
            height: parts[1].parse().map_err(|_| "Invalid height")?,
            x: parts[2].parse().map_err(|_| "Invalid X offset")?,
            y: parts[3].parse().map_err(|_| "Invalid Y offset")?,
        })
    } else {
        Err("Crop format: WxH:X:Y or W:H:X:Y".to_string())
    }
}

impl Cli {
    pub fn to_config(&self) -> CompressionConfig {
        let flip = if self.flip_h || self.flip_v {
            Some(FlipOptions {
                horizontal: self.flip_h,
                vertical: self.flip_v,
            })
        } else {
            None
        };

        let transforms = VideoTransforms {
            crop: self.crop.clone(),
            rotate: self.rotate,
            flip,
        };

        CompressionConfig {
            input_path: self.input.first().cloned().unwrap_or_default(),
            output_path: self.output.clone(),
            format: self.format.map(|f| f.into()),
            preset: self.preset.into(),
            quality: self.quality,
            width: self.width,
            height: self.height,
            fps: self.fps,
            mute: self.mute,
            transforms,
            overwrite: self.overwrite,
            verbose: self.verbose,
            json: self.json,
        }
    }
}
