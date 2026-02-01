use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum CompressoError {
    FileNotFound(String),
    InvalidInput(String),
    InvalidOutput(String),
    FfmpegNotFound,
    FfmpegError(String),
    Cancelled,
    CorruptedVideo,
    Io(std::io::Error),
}

impl fmt::Display for CompressoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::localization::t;

        match self {
            CompressoError::FileNotFound(path) => write!(f, "{}: {}", t("file_not_found"), path),
            CompressoError::InvalidInput(msg) => write!(f, "{}: {}", t("invalid_input_file"), msg),
            CompressoError::InvalidOutput(path) => write!(f, "{}: {}", t("invalid_output_path"), path),
            CompressoError::FfmpegNotFound => write!(f, "{}", t("ffmpeg_not_found")),
            CompressoError::FfmpegError(msg) => write!(f, "{}: {}", t("ffmpeg_error"), msg),
            CompressoError::Cancelled => write!(f, "{}", t("compression_cancelled_by_user")),
            CompressoError::CorruptedVideo => write!(f, "{}", t("video_corrupted_or_unsupported")),
            CompressoError::Io(io_error) => write!(f, "{}: {}", t("io_error"), io_error),
        }
    }
}

impl Error for CompressoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CompressoError::Io(io_error) => Some(io_error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CompressoError {
    fn from(error: std::io::Error) -> Self {
        CompressoError::Io(error)
    }
}

pub type Result<T> = std::result::Result<T, CompressoError>;
