use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::FileMetadata;
use crate::error::{CompressoError, Result};

/// Get metadata of a file from its path
///
/// Note: This function avoids TOCTOU race conditions by directly attempting
/// to read metadata without pre-checking file existence.
pub fn get_file_metadata(path: &str) -> Result<FileMetadata> {
    let file_path = Path::new(path);

    // Atomically get metadata (will fail if file doesn't exist)
    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(CompressoError::FileNotFound(path.to_string()));
        }
        Err(e) => return Err(e.into()),
    };
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

/// Generate output path from input path with security validation
///
/// # Security
///
/// This function protects against:
/// - Path traversal attacks (../ sequences)
/// - Symlink attacks (resolves symlinks to real paths)
/// - Writing outside expected directories
///
/// The output path is always generated in the same directory as the
/// canonicalized input file, preventing writes to unexpected locations.
///
pub fn generate_output_path(input: &str, format: Option<&str>) -> Result<String> {
    // Reject obviously malicious input early.
    if input.contains('\0') {
        return Err(CompressoError::InvalidInput(
            "input path contains null bytes".to_string(),
        ));
    }
    if input.contains("..") {
        return Err(CompressoError::InvalidInput(format!(
            "input path contains '..' (path traversal): {}",
            input
        )));
    }

    let input_path = Path::new(input);

    // Get the file stem, sanitizing any path traversal attempts
    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(sanitize_filename)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "output".to_string());

    let extension = format.unwrap_or_else(|| {
        input_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("mp4")
    });

    // Canonicalize the input to resolve symlinks and produce an absolute path.
    // At this call site the input file should already exist, so canonicalize
    // is expected to succeed. If it does not, we deliberately *fail* rather
    // than fall back to a relative parent (the old behavior silently dropped
    // `..` by collapsing to `parent().file_name()`, masking traversal).
    let parent = match fs::canonicalize(input_path) {
        Ok(canonical) => canonical
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".")),
        Err(_) => {
            // Input doesn't exist here — refuse rather than guess a location.
            return Err(CompressoError::FileNotFound(input.to_string()));
        }
    };

    let output_name = format!("{}_compressed.{}", stem, extension);

    let result = if parent.as_os_str().is_empty() || parent == Path::new(".") {
        PathBuf::from(output_name)
    } else {
        parent.join(output_name)
    };

    Ok(result.to_string_lossy().into_owned())
}

/// Sanitize filename to prevent path traversal while preserving Unicode.
///
/// This strips characters that are *structurally* dangerous to a filename
/// (path separators, null bytes, control characters) and any `..` traversal
/// sequences, but deliberately preserves legitimate non-ASCII characters
/// (Cyrillic, CJK, emoji, etc.). The previous implementation used an
/// alphanumeric allow-list which silently destroyed Cyrillic filenames such as
/// `видео.mp4` -> `_compressed.mp4`.
///
/// Note: this is applied to a *stem* (no directory components), as produced by
/// `Path::file_stem()`, but is defensive in case a separator slips through.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| {
            // Reject path separators (both Unix and Windows), null, and other
            // C0 control characters. Everything else (including non-ASCII
            // letters, spaces, punctuation) is preserved.
            *c != '/' && *c != '\\' && *c != '\0' && !c.is_control()
        })
        .collect::<String>()
        .replace("..", "") // remove any traversal sequences
        .trim_matches('.') // strip leading/trailing dots (hidden-file / traversal abuse)
        .trim() // strip stray whitespace
        .to_string()
}

/// Check if file exists
///
/// # Security Warning
///
/// This function is subject to TOCTOU (Time-of-Check Time-of-Use) race conditions.
/// The file may be created, deleted, or replaced between this check and subsequent use.
///
/// For security-critical operations, prefer atomic operations like:
/// - `std::fs::OpenOptions::new().create_new(true)` for exclusive creation
/// - Direct file operations that fail atomically if the file doesn't exist
///
/// This function should only be used for non-critical checks like user feedback.
pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Get all video files from a directory
///
/// Note: This function avoids TOCTOU race conditions by directly attempting
/// to read the directory without pre-checking existence.
pub fn get_video_files_in_directory(dir_path: &str) -> Result<Vec<String>> {
    let path = Path::new(dir_path);

    let mut video_files = Vec::new();

    // Atomically open directory (will fail if doesn't exist or not a directory)
    let read_dir = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(CompressoError::FileNotFound(dir_path.to_string()));
        }
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            return Err(CompressoError::InvalidInput(format!(
                "Permission denied: {}",
                dir_path
            )));
        }
        Err(_) => {
            return Err(CompressoError::InvalidInput(format!(
                "{} is not a valid directory",
                dir_path
            )));
        }
    };

    for entry in read_dir {
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
    fn test_sanitize_filename_preserves_unicode() {
        // Cyrillic must survive (was destroyed by the old alphanumeric filter).
        assert_eq!(sanitize_filename("видео"), "видео");
        // CJK too.
        assert_eq!(sanitize_filename("视频"), "视频");
        // Spaces, dots, dashes, underscores preserved.
        assert_eq!(sanitize_filename("my video - 1.mp4"), "my video - 1.mp4");
    }

    #[test]
    fn test_sanitize_filename_strips_dangerous() {
        // Path separators removed.
        assert_eq!(sanitize_filename("a/b\\c"), "abc");
        // Null bytes removed.
        assert_eq!(sanitize_filename("a\0b"), "ab");
        // Control characters removed.
        assert_eq!(sanitize_filename("a\x07b"), "ab");
        // Traversal sequences removed.
        assert_eq!(sanitize_filename(".."), "");
        assert_eq!(sanitize_filename("..secret"), "secret");
        // Leading/trailing dots trimmed.
        assert_eq!(sanitize_filename(".hidden."), "hidden");
    }

    #[test]
    fn test_generate_output_path_rejects_traversal() {
        // '..' must be rejected, not silently collapsed.
        let res = generate_output_path("../secret/video.mp4", None);
        assert!(res.is_err(), "expected path-traversal to be rejected");

        // Null bytes rejected.
        let res = generate_output_path("vi\0deo.mp4", None);
        assert!(res.is_err(), "expected null bytes to be rejected");
    }

    #[test]
    fn test_generate_output_path_rejects_missing_input() {
        // Non-existent input must now error (no silent relative fallback).
        let res = generate_output_path("definitely_nonexistent_video.mp4", None);
        assert!(res.is_err());
    }

    #[test]
    fn test_generate_output_path_for_existing_file() {
        // Use this very source file as a guaranteed-existing input.
        let src = env!("CARGO_MANIFEST_DIR").to_string() + "/src/fs.rs";
        let res = generate_output_path(&src, Some("webm"));
        assert!(res.is_ok(), "{:?}", res);
        let out = res.unwrap();
        assert!(
            out.ends_with("_compressed.webm"),
            "expected _compressed.webm suffix, got {out}"
        );
    }
}
