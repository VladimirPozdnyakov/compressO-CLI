use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompressoError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid input file: {0}")]
    InvalidInput(String),

    #[error("Invalid output path: {0}")]
    InvalidOutput(String),

    #[error("FFmpeg not found. Please install FFmpeg or use bundled version.")]
    FfmpegNotFound,

    #[error("FFmpeg error: {0}")]
    FfmpegError(String),

    #[error("Compression cancelled by user")]
    Cancelled,

    #[error("Video is corrupted or unsupported")]
    CorruptedVideo,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, CompressoError>;
