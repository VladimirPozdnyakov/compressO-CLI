use std::fs;
use std::path::Path;

use crate::domain::FileMetadata;
use crate::error::{CompressoError, Result};

/// Get metadata of a file from its path
pub fn get_file_metadata(path: &str) -> Result<FileMetadata> {
    let file_path = Path::new(path);

    if !file_path.exists() {
        return Err(CompressoError::FileNotFound(path.to_string()));
    }

    let metadata = fs::metadata(path)?;
    let mime_type = infer::get_from_path(path)
        .ok()
        .flatten()
        .map(|m| m.to_string())
        .unwrap_or_default();

    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let extension = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_string();

    Ok(FileMetadata {
        path: path.to_string(),
        file_name,
        mime_type,
        extension,
        size: metadata.len(),
    })
}

/// Check if file is a valid video file
pub fn is_video_file(path: &str) -> bool {
    let valid_extensions = ["mp4", "mov", "webm", "avi", "mkv", "m4v", "wmv", "flv"];

    if let Some(ext) = Path::new(path).extension() {
        if let Some(ext_str) = ext.to_str() {
            return valid_extensions.contains(&ext_str.to_lowercase().as_str());
        }
    }

    // Also check by MIME type
    if let Ok(Some(kind)) = infer::get_from_path(path) {
        return kind.mime_type().starts_with("video/");
    }

    false
}

/// Format bytes to human-readable size
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format duration in seconds to human-readable time
#[allow(dead_code)]
pub fn format_duration(seconds: f64) -> String {
    if seconds < 0.0 {
        return "0s".to_string();
    }

    let total_seconds = seconds.round() as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let secs = total_seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

/// Generate output path from input path
pub fn generate_output_path(input: &str, format: Option<&str>) -> String {
    let input_path = Path::new(input);
    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let extension = format.unwrap_or_else(|| {
        input_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("mp4")
    });

    let parent = input_path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());

    let output_name = format!("{}_compressed.{}", stem, extension);

    if parent.is_empty() || parent == "." {
        output_name
    } else {
        format!("{}/{}", parent, output_name)
    }
}

/// Check if file exists
pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Get all video files from a directory
pub fn get_video_files_in_directory(dir_path: &str) -> Result<Vec<String>> {
    let path = Path::new(dir_path);

    if !path.exists() {
        return Err(CompressoError::FileNotFound(dir_path.to_string()));
    }

    if !path.is_dir() {
        return Err(CompressoError::InvalidInput(format!(
            "{} is not a directory",
            dir_path
        )));
    }

    let mut video_files = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_file() {
            if let Some(path_str) = entry_path.to_str() {
                if is_video_file(path_str) {
                    video_files.push(path_str.to_string());
                }
            }
        }
    }

    video_files.sort();
    Ok(video_files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1048576), "1.00 MB");
        assert_eq!(format_size(1073741824), "1.00 GB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0.0), "0s");
        assert_eq!(format_duration(30.0), "30s");
        assert_eq!(format_duration(45.7), "46s");
        assert_eq!(format_duration(90.0), "1m 30s");
        assert_eq!(format_duration(125.0), "2m 5s");
        assert_eq!(format_duration(3600.0), "1h 0m 0s");
        assert_eq!(format_duration(3661.0), "1h 1m 1s");
        assert_eq!(format_duration(5025.0), "1h 23m 45s");
        assert_eq!(format_duration(330.0), "5m 30s");
        assert_eq!(format_duration(-10.0), "0s");
    }

    #[test]
    fn test_generate_output_path() {
        assert_eq!(
            generate_output_path("video.mp4", None),
            "video_compressed.mp4"
        );
        assert_eq!(
            generate_output_path("video.mp4", Some("webm")),
            "video_compressed.webm"
        );
    }
}
