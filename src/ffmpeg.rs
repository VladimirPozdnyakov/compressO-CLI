use crossbeam_channel::{Receiver, Sender};
use regex::Regex;
use shared_child::SharedChild;
use std::{
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use crate::domain::{CompressionConfig, CompressionResult, Preset, VideoInfo, VideoTransforms};
use crate::error::{CompressoError, Result};
use crate::progress::ProgressMetrics;

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

    /// Find FFmpeg binary
    fn find_ffmpeg() -> Result<String> {
        // First, check if ffmpeg is in PATH
        if let Ok(path) = which::which("ffmpeg") {
            return Ok(path.to_string_lossy().to_string());
        }

        // Check for bundled ffmpeg
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
                return Ok(bundled.to_string_lossy().to_string());
            }
        }

        Err(CompressoError::FfmpegNotFound)
    }

    /// Get video information
    pub fn get_video_info(&self, video_path: &str) -> Result<VideoInfo> {
        if !Path::new(video_path).exists() {
            return Err(CompressoError::FileNotFound(video_path.to_string()));
        }

        let output = Command::new(&self.ffmpeg_path)
            .args(["-i", video_path, "-hide_banner"])
            .stderr(Stdio::piped())
            .stdout(Stdio::null())
            .output()?;

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
    pub fn compress_video<F>(
        &self,
        config: &CompressionConfig,
        cancelled: Arc<AtomicBool>,
        progress_callback: F,
    ) -> Result<CompressionResult>
    where
        F: Fn(f64, f64, Option<f64>) + Send + 'static,
    {
        let input_path = &config.input_path;

        if !Path::new(input_path).exists() {
            return Err(CompressoError::FileNotFound(input_path.clone()));
        }

        // Get video info for progress calculation
        let video_info = self.get_video_info(input_path)?;
        let total_duration = video_info.duration_seconds.unwrap_or(0.0);

        // Determine output format and path
        let output_format = config.format.map(|f| f.extension().to_string()).unwrap_or_else(|| {
            Path::new(input_path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("mp4")
                .to_string()
        });

        let output_path = config.output_path.clone().unwrap_or_else(|| {
            crate::fs::generate_output_path(input_path, Some(&output_format))
        });

        // Check if output exists and overwrite is not set
        if !config.overwrite && Path::new(&output_path).exists() {
            return Err(CompressoError::InvalidOutput(format!(
                "File already exists: {}. Use -y to overwrite.",
                output_path
            )));
        }

        // Get original size
        let original_size = std::fs::metadata(input_path)?.len();

        // Create progress metrics for tracking speed and ETA
        let progress_metrics = Arc::new(Mutex::new(ProgressMetrics::new(
            original_size,
            Some(total_duration),
        )));
        let metrics_for_thread = progress_metrics.clone();

        // Build FFmpeg arguments
        let args = self.build_args(config, &output_path, &output_format);

        if config.verbose {
            eprintln!("FFmpeg command: {} {}", self.ffmpeg_path, args.join(" "));
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
        let child_for_cancel = child.clone();

        // Channel for progress updates
        let (tx, rx): (Sender<f64>, Receiver<f64>) = crossbeam_channel::unbounded();

        // Spawn thread to read stdout (progress)
        let cancelled_clone = cancelled.clone();
        std::thread::spawn(move || {
            if let Some(stdout) = child_clone.take_stdout() {
                let reader = BufReader::new(stdout);
                let re = Regex::new(r"out_time_ms=(\d+)").unwrap();
                let re_time = Regex::new(r"out_time=(\d{2}:\d{2}:\d{2}\.\d+)").unwrap();

                for line in reader.lines().map_while(|l| l.ok()) {
                    if cancelled_clone.load(Ordering::Relaxed) {
                        break;
                    }

                    // Try to parse out_time_ms first
                    if let Some(cap) = re.captures(&line) {
                        if let Ok(ms) = cap[1].parse::<f64>() {
                            let current_seconds = ms / 1_000_000.0;
                            if total_duration > 0.0 {
                                let progress = (current_seconds / total_duration * 100.0).min(100.0);
                                let _ = tx.try_send(progress);
                            }
                        }
                    }
                    // Fallback to out_time
                    else if let Some(cap) = re_time.captures(&line) {
                        if let Some(seconds) = Self::duration_to_seconds(&cap[1]) {
                            if total_duration > 0.0 {
                                let progress = (seconds / total_duration * 100.0).min(100.0);
                                let _ = tx.try_send(progress);
                            }
                        }
                    }
                }
            }
        });

        // Spawn thread for progress callback
        let cancelled_for_progress = cancelled.clone();
        std::thread::spawn(move || {
            while let Ok(progress) = rx.recv() {
                if cancelled_for_progress.load(Ordering::Relaxed) {
                    break;
                }

                // Update progress metrics with current progress and get speed/ETA
                let (speed, eta) = if let Ok(mut metrics) = metrics_for_thread.lock() {
                    metrics.update_progress(progress);
                    let speed = metrics.calculate_speed();
                    let eta = metrics.calculate_eta();
                    (speed, eta)
                } else {
                    (0.0, None)
                };

                progress_callback(progress, speed, eta);
            }
        });

        // Wait for completion or cancellation
        loop {
            if cancelled.load(Ordering::Relaxed) {
                let _ = child_for_cancel.kill();
                // Clean up partial output
                let _ = std::fs::remove_file(&output_path);
                return Err(CompressoError::Cancelled);
            }

            match child.try_wait() {
                Ok(Some(status)) => {
                    if status.success() {
                        break;
                    } else {
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
                    return Err(CompressoError::FfmpegError(e.to_string()));
                }
            }
        }

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

    fn build_args(&self, config: &CompressionConfig, output_path: &str, output_format: &str) -> Vec<String> {
        let mut args: Vec<String> = vec![
            "-i".to_string(),
            config.input_path.clone(),
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
