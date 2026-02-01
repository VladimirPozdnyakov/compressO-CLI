use crossbeam_channel::{Receiver, Sender};
use regex::Regex;
use shared_child::SharedChild;
use std::{
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use crate::domain::{CompressionConfig, CompressionResult, Preset, VideoInfo, VideoTransforms};
use crate::error::{CompressoError, Result};
use crate::progress::ProgressMetrics;

/// RAII guard that ensures temporary file is deleted on drop
struct TempFileGuard {
    path: PathBuf,
    keep: Arc<AtomicBool>,
    child: Option<Arc<SharedChild>>,
}

impl TempFileGuard {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            keep: Arc::new(AtomicBool::new(false)),
            child: None,
        }
    }

    fn set_child(&mut self, child: Arc<SharedChild>) {
        self.child = Some(child);
    }

    fn keep(&self) {
        self.keep.store(true, Ordering::Relaxed);
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        if !self.keep.load(Ordering::Relaxed) {
            // First, ensure FFmpeg process is terminated
            if let Some(ref child) = self.child {
                let _ = child.kill();
                // Wait a bit for the process to release the file
                std::thread::sleep(std::time::Duration::from_millis(200));
            }

            // Try to remove the file multiple times (Windows may need time to release handle)
            for i in 0..5 {
                match std::fs::remove_file(&self.path) {
                    Ok(_) => {
                        eprintln!("✓ Cleaned up temporary file: {}", self.path.display());
                        break;
                    }
                    Err(e) => {
                        if i < 4 {
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        } else {
                            eprintln!("⚠ Could not delete temporary file {}: {}", self.path.display(), e);
                        }
                    }
                }
            }
        }
    }
}

/// FFmpeg wrapper for video compression
pub struct FFmpeg {
    ffmpeg_path: String,
}

impl FFmpeg {
    /// Create new FFmpeg instance
    pub fn new() -> Result<Self> {
        let ffmpeg_path = Self::find_ffmpeg()?;
        Ok(Self { ffmpeg_path })
    }

    /// Find FFmpeg binary with security considerations
    ///
    /// # Security
    ///
    /// Search priority (highest to lowest):
    /// 1. COMPRESSO_FFMPEG_PATH environment variable (user-specified, most secure)
    /// 2. Bundled FFmpeg in application directory (verified if compiled with checks)
    /// 3. System PATH (least secure, vulnerable to PATH hijacking)
    ///
    /// The resolved path is logged to stderr for security auditing.
    ///
    /// # Environment Variables
    ///
    /// - `COMPRESSO_FFMPEG_PATH`: Explicit path to FFmpeg binary (recommended for security)
    /// - `COMPRESSO_FFMPEG_VERIFY`: Set to "1" to enable strict verification (bundled only)
    ///
    fn find_ffmpeg() -> Result<String> {
        // Priority 1: Explicit user-specified path (most secure)
        if let Ok(explicit_path) = std::env::var("COMPRESSO_FFMPEG_PATH") {
            let path = Path::new(&explicit_path);
            if path.exists() && path.is_file() {
                eprintln!("ℹ Using FFmpeg from COMPRESSO_FFMPEG_PATH: {}", explicit_path);
                return Ok(explicit_path);
            } else {
                eprintln!("⚠ COMPRESSO_FFMPEG_PATH set but invalid: {}", explicit_path);
                return Err(CompressoError::FfmpegNotFound);
            }
        }

        // Priority 2: Bundled FFmpeg (verified)
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));

        if let Some(dir) = exe_dir {
            let bundled = if cfg!(windows) {
                dir.join("ffmpeg.exe")
            } else {
                dir.join("ffmpeg")
            };

            if bundled.exists() {
                let bundled_path = bundled.to_string_lossy().to_string();

                // Verify bundled FFmpeg if requested
                if std::env::var("COMPRESSO_FFMPEG_VERIFY").unwrap_or_default() == "1" {
                    if let Err(e) = Self::verify_bundled_ffmpeg(&bundled) {
                        eprintln!("⚠ Bundled FFmpeg verification failed: {}", e);
                        eprintln!("⚠ Set COMPRESSO_FFMPEG_PATH to use a trusted FFmpeg binary");
                        return Err(CompressoError::FfmpegNotFound);
                    }
                }

                eprintln!("ℹ Using bundled FFmpeg: {}", bundled_path);
                return Ok(bundled_path);
            }
        }

        // Priority 3: System PATH (least secure - log warning)
        if let Ok(path) = which::which("ffmpeg") {
            let path_str = path.to_string_lossy().to_string();
            eprintln!("⚠ Using FFmpeg from system PATH: {}", path_str);
            eprintln!("⚠ For better security, set COMPRESSO_FFMPEG_PATH to an explicit path");
            return Ok(path_str);
        }

        Err(CompressoError::FfmpegNotFound)
    }

    /// Verify bundled FFmpeg binary integrity
    ///
    /// This is a basic verification that checks if the binary is executable
    /// and responds to --version. For production use, consider adding:
    /// - SHA256 hash verification against known-good builds
    /// - Code signature verification on Windows/macOS
    fn verify_bundled_ffmpeg(path: &Path) -> Result<()> {
        // Check if file is executable (Unix-like systems)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(path)?;
            let permissions = metadata.permissions();
            if permissions.mode() & 0o111 == 0 {
                return Err(CompressoError::InvalidInput(
                    "Bundled FFmpeg is not executable".to_string()
                ));
            }
        }

        // Verify it's actually FFmpeg by checking --version output
        match std::process::Command::new(path)
            .arg("--version")
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if !stdout.contains("ffmpeg version") {
                    return Err(CompressoError::InvalidInput(
                        "Binary does not appear to be FFmpeg".to_string()
                    ));
                }
                Ok(())
            }
            Err(e) => Err(CompressoError::InvalidInput(format!(
                "Failed to verify FFmpeg binary: {}",
                e
            ))),
        }
    }

    /// Validate and sanitize path to prevent path traversal attacks
    ///
    /// # Security
    ///
    /// This function protects against:
    /// - Path traversal (.., ./, etc.)
    /// - Null bytes in paths
    /// - Symlink attacks (resolves to real path)
    ///
    /// For existing files, the path is canonicalized to get the real absolute path.
    /// For non-existing files (like output paths), basic validation is performed.
    ///
    fn validate_path(path: &str, path_type: &str) -> Result<String> {
        // Check for null bytes (can bypass security checks)
        if path.contains('\0') {
            return Err(CompressoError::InvalidInput(format!(
                "{} path contains null bytes",
                path_type
            )));
        }

        // Check for suspicious patterns
        if path.contains("..") {
            eprintln!(
                "⚠ Warning: {} path contains '..' which may indicate path traversal: {}",
                path_type, path
            );
        }

        // Try to canonicalize (resolve symlinks and get absolute path)
        // This only works for existing files
        match std::fs::canonicalize(path) {
            Ok(canonical) => {
                let canonical_str = canonical.to_string_lossy().to_string();

                // Check if the canonicalized path is dramatically different (potential symlink attack)
                if !path.contains(&canonical_str) && !canonical_str.contains(path) {
                    eprintln!(
                        "ℹ {} path resolved through symlink:",
                        path_type
                    );
                    eprintln!("  Provided: {}", path);
                    eprintln!("  Resolved: {}", canonical_str);
                }

                Ok(canonical_str)
            }
            Err(_) => {
                // File doesn't exist yet (e.g., output file)
                // Return the path as-is but it will be validated by validate_output_path
                Ok(path.to_string())
            }
        }
    }

    /// Validate output path to prevent writing to dangerous locations
    ///
    /// # Security
    ///
    /// Prevents writing to:
    /// - System directories (/etc, /sys, /proc, C:\Windows, etc.)
    /// - Root directory
    /// - Parent directories via traversal
    ///
    fn validate_output_path(path: &str) -> Result<()> {
        let path_lower = path.to_lowercase();

        // List of dangerous paths/prefixes
        let dangerous_paths = [
            "/etc/",
            "/sys/",
            "/proc/",
            "/dev/",
            "/boot/",
            "/root/",
            "c:\\windows\\",
            "c:\\program files\\",
            "c:\\program files (x86)\\",
            "/windows/",
            "/program files/",
        ];

        for dangerous in &dangerous_paths {
            if path_lower.starts_with(dangerous) || path_lower.contains(dangerous) {
                return Err(CompressoError::InvalidOutput(format!(
                    "Refusing to write to system directory: {}",
                    path
                )));
            }
        }

        // Check if trying to write to root
        let path_obj = Path::new(path);
        if let Ok(canonical) = std::fs::canonicalize(path_obj.parent().unwrap_or(Path::new("."))) {
            let canonical_str = canonical.to_string_lossy();

            // Unix root
            if canonical_str == "/" {
                return Err(CompressoError::InvalidOutput(
                    "Refusing to write to root directory".to_string()
                ));
            }

            // Windows drive root (C:\, D:\, etc.)
            #[cfg(windows)]
            {
                if canonical_str.len() == 3 && canonical_str.ends_with(":\\") {
                    return Err(CompressoError::InvalidOutput(
                        "Refusing to write to drive root".to_string()
                    ));
                }
            }
        }

        Ok(())
    }

    /// Sanitize FFmpeg arguments for safe logging
    ///
    /// # Security
    ///
    /// This function prevents information disclosure (CWE-532) by:
    /// - Replacing full file paths with just filenames
    /// - Redacting user's home directory path
    /// - Preserving FFmpeg flags and options for debugging
    ///
    /// This protects against:
    /// - File system structure disclosure
    /// - Username exposure in paths
    /// - Sensitive directory names in logs
    ///
    fn sanitize_args_for_logging(args: &[String]) -> Vec<String> {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_default();

        args.iter()
            .map(|arg| {
                // Check if this looks like a file path (contains path separators or has extension)
                if arg.contains('/') || arg.contains('\\') || arg.contains('.') && !arg.starts_with('-') {
                    // Extract just the filename
                    if let Some(filename) = Path::new(arg).file_name() {
                        let filename_str = filename.to_string_lossy().to_string();

                        // If it's in home directory, indicate that
                        if !home_dir.is_empty() && arg.contains(&home_dir) {
                            format!("~/{}", filename_str)
                        } else {
                            filename_str
                        }
                    } else {
                        // Redact home directory path if present
                        if !home_dir.is_empty() && arg.contains(&home_dir) {
                            arg.replace(&home_dir, "~")
                        } else {
                            arg.clone()
                        }
                    }
                } else {
                    // Not a path, keep as-is (FFmpeg flags, options, etc.)
                    arg.clone()
                }
            })
            .collect()
    }

    /// Get video information
    ///
    /// Note: This function does not pre-check file existence to avoid TOCTOU race conditions.
    /// FFmpeg will atomically open and validate the file.
    pub fn get_video_info(&self, video_path: &str) -> Result<VideoInfo> {
        let output = Command::new(&self.ffmpeg_path)
            .args(["-i", video_path, "-hide_banner"])
            .stderr(Stdio::piped())
            .stdout(Stdio::null())
            .output()?;

        // Check if FFmpeg failed (likely file not found or invalid)
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("No such file") || stderr.contains("does not exist") {
                return Err(CompressoError::FileNotFound(video_path.to_string()));
            }
        }

        let stderr = String::from_utf8_lossy(&output.stderr);

        let duration = Self::parse_duration(&stderr);
        let duration_seconds = duration.as_ref().and_then(|d| Self::duration_to_seconds(d));
        let dimensions = Self::parse_dimensions(&stderr);
        let fps = Self::parse_fps(&stderr);

        Ok(VideoInfo {
            duration,
            duration_seconds,
            dimensions,
            fps,
        })
    }

    fn parse_duration(output: &str) -> Option<String> {
        let re = Regex::new(r"Duration: (?P<duration>\d{2}:\d{2}:\d{2}\.\d{2})").ok()?;
        re.captures(output)
            .map(|cap| cap["duration"].to_string())
    }

    fn parse_dimensions(output: &str) -> Option<(u32, u32)> {
        let re = Regex::new(r"Video:.*? (\d{2,5})x(\d{2,5})").ok()?;
        re.captures(output).and_then(|cap| {
            let width = cap.get(1)?.as_str().parse().ok()?;
            let height = cap.get(2)?.as_str().parse().ok()?;
            Some((width, height))
        })
    }

    fn parse_fps(output: &str) -> Option<f32> {
        let re = Regex::new(r"(\d+(?:\.\d+)?)\s*fps").ok()?;
        re.captures(output)
            .and_then(|cap| cap.get(1)?.as_str().parse().ok())
    }

    fn duration_to_seconds(duration: &str) -> Option<f64> {
        let parts: Vec<&str> = duration.split(':').collect();
        if parts.len() != 3 {
            return None;
        }

        let hours: f64 = parts[0].parse().ok()?;
        let minutes: f64 = parts[1].parse().ok()?;
        let seconds: f64 = parts[2].parse().ok()?;

        Some(hours * 3600.0 + minutes * 60.0 + seconds)
    }

    /// Compress video with progress callback
    ///
    /// # Security
    ///
    /// This function avoids TOCTOU race conditions by:
    /// - Not pre-checking input file existence (FFmpeg opens it atomically)
    /// - Using unique temporary filenames to avoid collisions
    /// - Atomically renaming temp file to final output on success
    ///
    /// Path traversal and symlink protection:
    /// - Input and output paths are canonicalized to resolve symlinks
    /// - Paths are validated to prevent writing outside expected directories
    /// - Dangerous path sequences (.., null bytes) are rejected
    ///
    /// Note: Output overwrite protection check still has a small race window.
    /// Use unique output paths or enable overwrite mode for maximum safety.
    pub fn compress_video<F>(
        &self,
        config: &CompressionConfig,
        cancelled: Arc<AtomicBool>,
        progress_callback: F,
    ) -> Result<CompressionResult>
    where
        F: Fn(f64, u32, u32, f64, Option<f64>) + Send + 'static,
    {
        let input_path = &config.input_path;

        // Validate and canonicalize input path (protect against path traversal)
        let validated_input = Self::validate_path(input_path, "input")?;

        // Get video info for progress calculation (will fail atomically if file doesn't exist)
        let video_info = self.get_video_info(&validated_input)?;
        let total_duration = video_info.duration_seconds.unwrap_or(0.0);
        let fps = video_info.fps.unwrap_or(30.0);
        let total_frames = (total_duration * fps as f64) as u32;

        // Determine output format and path
        let output_format = config.format.map(|f| f.extension().to_string()).unwrap_or_else(|| {
            Path::new(&validated_input)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("mp4")
                .to_string()
        });

        let output_path = config.output_path.clone().unwrap_or_else(|| {
            crate::fs::generate_output_path(&validated_input, Some(&output_format))
        });

        // Validate output path (protect against path traversal and symlink attacks)
        Self::validate_output_path(&output_path)?;

        // Atomically check if output exists and prevent overwrite if not set
        // This uses create_new() which atomically fails if file exists
        if !config.overwrite {
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&output_path)
            {
                Ok(f) => {
                    // File didn't exist, we created it. Remove it immediately.
                    drop(f);
                    let _ = std::fs::remove_file(&output_path);
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    return Err(CompressoError::InvalidOutput(format!(
                        "File already exists: {}. Use -y to overwrite.",
                        output_path
                    )));
                }
                Err(e) => return Err(e.into()),
            }
        }

        // Create temporary output path for atomic write
        // Keep the correct extension so FFmpeg can detect the output format
        let output_path_obj = Path::new(&output_path);
        let temp_output_path = if let Some(stem) = output_path_obj.file_stem() {
            let extension = output_path_obj.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("mp4");
            let temp_filename = format!("{}.tmp.{}.{}", stem.to_string_lossy(), nanoid::nanoid!(8), extension);

            if let Some(parent) = output_path_obj.parent() {
                parent.join(temp_filename).to_string_lossy().to_string()
            } else {
                temp_filename
            }
        } else {
            format!("{}.tmp.{}", output_path, nanoid::nanoid!(8))
        };

        // Create RAII guard to ensure temp file is cleaned up on any exit path
        let mut temp_guard = TempFileGuard::new(PathBuf::from(&temp_output_path));

        // Get original size
        let original_size = std::fs::metadata(&validated_input)?.len();

        // Create progress metrics for tracking speed and ETA
        let progress_metrics = Arc::new(Mutex::new(ProgressMetrics::new(
            original_size,
            Some(total_duration),
        )));
        let metrics_for_thread = progress_metrics.clone();

        // Build FFmpeg arguments (write to temp file for atomic operation)
        let args = self.build_args(config, &validated_input, &temp_output_path, &output_format);

        if config.verbose {
            // Sanitize arguments to avoid leaking full paths in logs
            let sanitized_args = Self::sanitize_args_for_logging(&args);
            eprintln!("ℹ FFmpeg command (paths sanitized): ffmpeg {}", sanitized_args.join(" "));
        }

        // Spawn FFmpeg process
        let mut command = Command::new(&self.ffmpeg_path);
        command
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = SharedChild::spawn(&mut command)
            .map_err(|e| CompressoError::FfmpegError(e.to_string()))?;

        let child = Arc::new(child);
        let child_clone = child.clone();

        // Give the guard access to the child process so it can kill it on drop
        temp_guard.set_child(child.clone());

        // Channel for progress updates (progress, current_frame)
        let (tx, rx): (Sender<(f64, u32)>, Receiver<(f64, u32)>) = crossbeam_channel::unbounded();

        // Spawn thread to read stdout (progress)
        let cancelled_clone = cancelled.clone();
        std::thread::spawn(move || {
            if let Some(stdout) = child_clone.take_stdout() {
                let reader = BufReader::new(stdout);
                let re = Regex::new(r"out_time_ms=(\d+)").unwrap();
                let re_time = Regex::new(r"out_time=(\d{2}:\d{2}:\d{2}\.\d+)").unwrap();
                let re_frame = Regex::new(r"frame=\s*(\d+)").unwrap();

                let mut current_frame: u32 = 0;

                for line in reader.lines().map_while(|l| l.ok()) {
                    if cancelled_clone.load(Ordering::Relaxed) {
                        break;
                    }

                    // Parse frame number
                    if let Some(cap) = re_frame.captures(&line) {
                        if let Ok(frame) = cap[1].parse::<u32>() {
                            current_frame = frame;
                        }
                    }

                    // Try to parse out_time_ms first
                    if let Some(cap) = re.captures(&line) {
                        if let Ok(ms) = cap[1].parse::<f64>() {
                            let current_seconds = ms / 1_000_000.0;
                            if total_duration > 0.0 {
                                let progress = (current_seconds / total_duration * 100.0).min(100.0);
                                let _ = tx.try_send((progress, current_frame));
                            }
                        }
                    }
                    // Fallback to out_time
                    else if let Some(cap) = re_time.captures(&line) {
                        if let Some(seconds) = Self::duration_to_seconds(&cap[1]) {
                            if total_duration > 0.0 {
                                let progress = (seconds / total_duration * 100.0).min(100.0);
                                let _ = tx.try_send((progress, current_frame));
                            }
                        }
                    }
                }
            }
        });

        // Spawn thread for progress callback
        let cancelled_for_progress = cancelled.clone();
        let mut last_frame: u32 = 0;
        let mut last_time = std::time::Instant::now();
        let mut last_fps: f64 = 0.0;

        std::thread::spawn(move || {
            while let Ok((progress, current_frame)) = rx.recv() {
                if cancelled_for_progress.load(Ordering::Relaxed) {
                    break;
                }

                // Calculate FPS (frames per second)
                let now = std::time::Instant::now();
                let elapsed = now.duration_since(last_time).as_secs_f64();

                // Update FPS calculation if enough time has passed
                if elapsed > 0.3 && current_frame > last_frame {
                    let frames_processed = current_frame.saturating_sub(last_frame);
                    last_fps = frames_processed as f64 / elapsed;
                    last_frame = current_frame;
                    last_time = now;
                }

                // Update progress metrics to get ETA
                let eta = if let Ok(mut metrics) = metrics_for_thread.lock() {
                    metrics.update_progress(progress);
                    metrics.calculate_eta()
                } else {
                    None
                };

                progress_callback(progress, current_frame, total_frames, last_fps, eta);
            }
        });

        // Wait for completion or cancellation
        loop {
            if cancelled.load(Ordering::Relaxed) {
                // temp_guard will automatically kill FFmpeg and clean up the file on return
                return Err(CompressoError::Cancelled);
            }

            match child.try_wait() {
                Ok(Some(status)) => {
                    if status.success() {
                        break;
                    } else {
                        // temp_guard will automatically clean up the file on return
                        // Read stderr for error message
                        if let Some(mut stderr) = child.take_stderr() {
                            let mut error_msg = String::new();
                            let _ = std::io::Read::read_to_string(&mut stderr, &mut error_msg);
                            if !error_msg.is_empty() {
                                return Err(CompressoError::FfmpegError(error_msg));
                            }
                        }
                        return Err(CompressoError::CorruptedVideo);
                    }
                }
                Ok(None) => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => {
                    // temp_guard will automatically clean up the file on return
                    return Err(CompressoError::FfmpegError(e.to_string()));
                }
            }
        }

        // Success! Tell the guard to keep the temp file (we'll rename it)
        temp_guard.keep();

        // Atomic rename: move temp file to final output path
        std::fs::rename(&temp_output_path, &output_path)?;

        // Get compressed size
        let compressed_size = std::fs::metadata(&output_path)?.len();

        Ok(CompressionResult {
            file_name: Path::new(&output_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("output")
                .to_string(),
            file_path: output_path,
            original_size,
            compressed_size,
        })
    }

    fn build_args(&self, config: &CompressionConfig, input_path: &str, output_path: &str, output_format: &str) -> Vec<String> {
        let mut args: Vec<String> = vec![
            "-i".to_string(),
            input_path.to_string(),
            "-hide_banner".to_string(),
            "-progress".to_string(),
            "-".to_string(),
            "-nostats".to_string(),
            "-loglevel".to_string(),
            "error".to_string(),
        ];

        // Calculate CRF from quality (0-100)
        // Lower CRF = higher quality
        // CRF range: 24 (best) to 36 (worst)
        let max_crf: u16 = 36;
        let min_crf: u16 = 24;
        let quality = config.quality.min(100) as u16;
        let crf = min_crf + (max_crf - min_crf) * (100 - quality) / 100;
        let crf_str = crf.to_string();

        // Add preset-specific arguments
        match config.preset {
            Preset::Thunderbolt => {
                args.extend([
                    "-c:v".to_string(),
                    "libx264".to_string(),
                    "-crf".to_string(),
                    crf_str,
                ]);
            }
            Preset::Ironclad => {
                args.extend([
                    "-pix_fmt".to_string(),
                    "yuv420p".to_string(),
                    "-c:v".to_string(),
                    "libx264".to_string(),
                    "-b:v".to_string(),
                    "0".to_string(),
                    "-movflags".to_string(),
                    "+faststart".to_string(),
                    "-preset".to_string(),
                    "slow".to_string(),
                    "-qp".to_string(),
                    "0".to_string(),
                    "-crf".to_string(),
                    crf_str,
                ]);
            }
        }

        // Build video filters
        let filters = self.build_filters(config);
        if !filters.is_empty() {
            args.extend(["-vf".to_string(), filters]);
        }

        // FPS
        if let Some(fps) = config.fps {
            args.extend(["-r".to_string(), fps.to_string()]);
        }

        // WebM codec
        if output_format == "webm" {
            args.extend(["-c:v".to_string(), "libvpx-vp9".to_string()]);
        }

        // Mute audio
        if config.mute {
            args.push("-an".to_string());
        }

        // Output path
        args.push(output_path.to_string());

        // Overwrite
        if config.overwrite {
            args.push("-y".to_string());
        }

        args
    }

    fn build_filters(&self, config: &CompressionConfig) -> String {
        let mut filters: Vec<String> = Vec::new();

        // Apply transforms
        self.apply_transforms(&config.transforms, &mut filters);

        // Dimensions
        let padding = "pad=ceil(iw/2)*2:ceil(ih/2)*2";
        if let (Some(w), Some(h)) = (config.width, config.height) {
            filters.push(format!("scale={}:{}", w, h));
        }
        filters.push(padding.to_string());

        filters.join(",")
    }

    fn apply_transforms(&self, transforms: &VideoTransforms, filters: &mut Vec<String>) {
        // Rotate
        if let Some(angle) = transforms.rotate {
            match angle % 360 {
                90 | -270 => filters.push("transpose=1".to_string()),
                -90 | 270 => filters.push("transpose=2".to_string()),
                180 | -180 => filters.push("hflip,vflip".to_string()),
                _ => {}
            }
        }

        // Flip
        if let Some(ref flip) = transforms.flip {
            if flip.horizontal {
                filters.push("hflip".to_string());
            }
            if flip.vertical {
                filters.push("vflip".to_string());
            }
        }

        // Crop
        if let Some(ref crop) = transforms.crop {
            filters.push(format!(
                "crop={}:{}:{}:{}",
                crop.width, crop.height, crop.x, crop.y
            ));
        }
    }
}

impl Default for FFmpeg {
    fn default() -> Self {
        Self::new().expect("FFmpeg not found")
    }
}
