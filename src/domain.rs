use serde::{Deserialize, Serialize};

/// Result of a successful video compression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionResult {
    pub file_name: String,
    pub file_path: String,
    pub original_size: u64,
    pub compressed_size: u64,
}

/// File metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: String,
    pub file_name: String,
    pub mime_type: String,
    pub extension: String,
    pub size: u64,
}

/// Video information extracted from FFmpeg
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub duration: Option<String>,
    pub duration_seconds: Option<f64>,
    pub dimensions: Option<(u32, u32)>,
    pub fps: Option<f32>,
}

/// Crop coordinates for video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CropCoordinates {
    pub width: u32,
    pub height: u32,
    pub x: u32,
    pub y: u32,
}

/// Flip options for video
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlipOptions {
    pub horizontal: bool,
    pub vertical: bool,
}

/// Video transformation options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VideoTransforms {
    pub crop: Option<CropCoordinates>,
    pub rotate: Option<i32>,
    pub flip: Option<FlipOptions>,
}

/// Compression preset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Preset {
    /// Fast compression with good quality
    #[default]
    Thunderbolt,
    /// Best quality, slower compression
    Ironclad,
}

impl std::str::FromStr for Preset {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "thunderbolt" | "fast" => Ok(Preset::Thunderbolt),
            "ironclad" | "quality" => Ok(Preset::Ironclad),
            _ => Err(format!("Unknown preset: {}. Use 'thunderbolt' or 'ironclad'", s)),
        }
    }
}

impl std::fmt::Display for Preset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Preset::Thunderbolt => write!(f, "thunderbolt"),
            Preset::Ironclad => write!(f, "ironclad"),
        }
    }
}

/// Supported output formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Mp4,
    Mov,
    Webm,
    Avi,
    Mkv,
}

impl OutputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Mp4 => "mp4",
            OutputFormat::Mov => "mov",
            OutputFormat::Webm => "webm",
            OutputFormat::Avi => "avi",
            OutputFormat::Mkv => "mkv",
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "mp4" => Some(OutputFormat::Mp4),
            "mov" => Some(OutputFormat::Mov),
            "webm" => Some(OutputFormat::Webm),
            "avi" => Some(OutputFormat::Avi),
            "mkv" => Some(OutputFormat::Mkv),
            _ => None,
        }
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_extension(s).ok_or_else(|| {
            format!(
                "Unknown format: {}. Supported: mp4, mov, webm, avi, mkv",
                s
            )
        })
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.extension())
    }
}

/// Compression configuration
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    pub input_path: String,
    pub output_path: Option<String>,
    pub format: Option<OutputFormat>,
    pub preset: Preset,
    pub quality: u8,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<u32>,
    pub mute: bool,
    pub transforms: VideoTransforms,
    pub overwrite: bool,
    pub verbose: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            input_path: String::new(),
            output_path: None,
            format: None,
            preset: Preset::default(),
            quality: 70,
            width: None,
            height: None,
            fps: None,
            mute: false,
            transforms: VideoTransforms::default(),
            overwrite: false,
            verbose: false,
        }
    }
}
