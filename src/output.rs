use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;
use std::sync::{Arc, Mutex};

use crate::domain::{CompressionConfig, CompressionResult, Preset, VideoInfo};
use crate::fs::format_size;

/// Print application header
pub fn print_header() {
    println!();
    println!(
        "{}",
        "  CompressO CLI v1.0.0".bright_cyan().bold()
    );
    println!("{}", "━".repeat(50).dimmed());
    println!();
}

/// Print video information
pub fn print_video_info(path: &str, info: &VideoInfo, size: u64) {
    println!("{}", "Video Information".bright_white().bold());
    println!("{}", "─".repeat(30).dimmed());

    println!(
        "  {} {}",
        "File:".dimmed(),
        path.bright_white()
    );
    println!(
        "  {} {}",
        "Size:".dimmed(),
        format_size(size).bright_yellow()
    );

    if let Some(duration) = &info.duration {
        println!(
            "  {} {}",
            "Duration:".dimmed(),
            duration.bright_white()
        );
    }

    if let Some((w, h)) = info.dimensions {
        println!(
            "  {} {}x{}",
            "Resolution:".dimmed(),
            w.to_string().bright_white(),
            h.to_string().bright_white()
        );
    }

    if let Some(fps) = info.fps {
        println!(
            "  {} {} fps",
            "Frame rate:".dimmed(),
            format!("{:.2}", fps).bright_white()
        );
    }

    println!();
}

/// Print compression configuration
pub fn print_config(config: &CompressionConfig, output_path: &str) {
    println!("{}", "Compression Settings".bright_white().bold());
    println!("{}", "─".repeat(30).dimmed());

    println!(
        "  {} {}",
        "Input:".dimmed(),
        config.input_path.bright_white()
    );
    println!(
        "  {} {}",
        "Output:".dimmed(),
        output_path.bright_white()
    );
    println!(
        "  {} {}",
        "Preset:".dimmed(),
        match config.preset {
            Preset::Thunderbolt => "thunderbolt (fast)".bright_green(),
            Preset::Ironclad => "ironclad (quality)".bright_blue(),
        }
    );
    println!(
        "  {} {}%",
        "Quality:".dimmed(),
        config.quality.to_string().bright_yellow()
    );

    if let (Some(w), Some(h)) = (config.width, config.height) {
        println!(
            "  {} {}x{}",
            "Dimensions:".dimmed(),
            w.to_string().bright_white(),
            h.to_string().bright_white()
        );
    }

    if let Some(fps) = config.fps {
        println!(
            "  {} {} fps",
            "FPS:".dimmed(),
            fps.to_string().bright_white()
        );
    }

    if config.mute {
        println!(
            "  {} {}",
            "Audio:".dimmed(),
            "muted".bright_red()
        );
    }

    println!();
}

/// Create and return a progress bar
pub fn create_progress_bar() -> Arc<Mutex<ProgressBar>> {
    let pb = ProgressBar::new(10000);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {msg}")
            .unwrap()
            .progress_chars("█▓░"),
    );
    pb.set_message("0.00% | ETA: -- | Calculating...");
    Arc::new(Mutex::new(pb))
}

/// Update progress bar with speed and ETA
pub fn update_progress(pb: &Arc<Mutex<ProgressBar>>, progress: f64, speed: f64, eta: Option<f64>) {
    if let Ok(pb) = pb.lock() {
        // Convert progress from 0-100 range to 0-10000 range for precision
        pb.set_position((progress * 100.0) as u64);

        // Format ETA
        let eta_msg = if let Some(eta_secs) = eta {
            let eta_mins = (eta_secs / 60.0) as u64;
            let eta_secs_rem = (eta_secs % 60.0) as u64;
            format!("{:02}:{:02}", eta_mins, eta_secs_rem)
        } else {
            "--:--".to_string()
        };

        // Format the message with percentage, ETA, and speed
        let speed_msg = if speed > 0.0 {
            format!("{:.2}% | ETA: {} | {}/s", progress, eta_msg, format_size(speed as u64))
        } else {
            format!("{:.2}% | ETA: {} | Calculating...", progress, eta_msg)
        };

        pb.set_message(speed_msg);
    }
}

/// Finish progress bar
pub fn finish_progress(pb: &Arc<Mutex<ProgressBar>>) {
    if let Ok(pb) = pb.lock() {
        pb.finish_with_message("Done!");
    }
}

/// Generate a visual size comparison bar
fn create_size_bar(size: u64, max_size: u64, bar_width: usize) -> String {
    if max_size == 0 {
        return "░".repeat(bar_width);
    }

    let filled_width = ((size as f64 / max_size as f64) * bar_width as f64) as usize;
    let filled_width = filled_width.min(bar_width);
    let empty_width = bar_width.saturating_sub(filled_width);

    format!(
        "{}{}",
        "█".repeat(filled_width).bright_cyan(),
        "░".repeat(empty_width).dimmed()
    )
}

/// Print compression result
pub fn print_result(result: &CompressionResult, elapsed: std::time::Duration) {
    println!();
    println!("{}", "━".repeat(50).dimmed());
    println!(
        "{} {}",
        "✓".bright_green().bold(),
        "Compression complete!".bright_green().bold()
    );
    println!();

    let saved = result.original_size.saturating_sub(result.compressed_size);

    let ratio = if result.original_size > 0 {
        (saved as f64 / result.original_size as f64) * 100.0
    } else {
        0.0
    };

    // Visual size comparison
    let bar_width = 40;
    let original_bar = create_size_bar(result.original_size, result.original_size, bar_width);
    let compressed_bar = create_size_bar(result.compressed_size, result.original_size, bar_width);

    println!(
        "  {} {} {}",
        "Original:".dimmed(),
        original_bar,
        format_size(result.original_size).bright_white()
    );
    println!(
        "  {} {} {}",
        "Compressed:".dimmed(),
        compressed_bar,
        format_size(result.compressed_size).bright_green()
    );
    println!();
    println!(
        "  {} {} ({:.1}%)",
        "Saved:".dimmed(),
        format_size(saved).bright_yellow(),
        ratio
    );
    println!(
        "  {} {:.2}s",
        "Time:".dimmed(),
        elapsed.as_secs_f64()
    );
    println!();
    println!(
        "  {} {}",
        "Output:".dimmed(),
        result.file_path.bright_cyan()
    );
    println!();
}

/// Print error message (simple version without hints)
/// For errors with actionable hints, use print_error_with_hint instead
#[allow(dead_code)]
pub fn print_error(message: &str) {
    eprintln!();
    eprintln!(
        "{} {}",
        "✗".bright_red().bold(),
        message.bright_red()
    );
    eprintln!();
}

/// Print error message with actionable hints based on error type
pub fn print_error_with_hint(error: &crate::error::CompressoError) {
    use crate::error::CompressoError;

    eprintln!();
    eprintln!(
        "{} {}",
        "✗".bright_red().bold(),
        error.to_string().bright_red()
    );
    eprintln!();

    // Provide specific, actionable hints based on error type
    let hint = match error {
        CompressoError::FfmpegNotFound => {
            "💡 How to install FFmpeg:\n\
             \n\
             Windows:\n\
               • winget install Gyan.FFmpeg\n\
               • or download from https://ffmpeg.org/download.html\n\
             \n\
             macOS:\n\
               • brew install ffmpeg\n\
             \n\
             Linux:\n\
               • sudo apt install ffmpeg  (Debian/Ubuntu)\n\
               • sudo dnf install ffmpeg  (Fedora)\n\
               • sudo pacman -S ffmpeg    (Arch)"
        }
        CompressoError::FileNotFound(path) => {
            &format!(
                "💡 Suggestions:\n\
                 \n\
                   • Check if the file path is correct: {}\n\
                   • Make sure you have permission to access the file\n\
                   • Try using an absolute path instead of a relative path\n\
                   • On Windows, use quotes around paths with spaces",
                path
            )
        }
        CompressoError::InvalidInput(_) => {
            "💡 Supported video formats:\n\
             \n\
               • MP4 (.mp4)\n\
               • MOV (.mov)\n\
               • WebM (.webm)\n\
               • AVI (.avi)\n\
               • MKV (.mkv)\n\
               • FLV (.flv)\n\
               • WMV (.wmv)\n\
             \n\
             Check that your file has a valid video extension and is not corrupted."
        }
        CompressoError::CorruptedVideo => {
            "💡 Possible solutions:\n\
             \n\
               • Try playing the video in a media player to verify it works\n\
               • The file might be incomplete or corrupted during download\n\
               • Try re-encoding the video with a different tool first\n\
               • Check if the file is actually a video (not renamed from another format)"
        }
        CompressoError::InvalidOutput(path) => {
            &format!(
                "💡 Suggestions:\n\
                 \n\
                   • Check if the output directory exists: {}\n\
                   • Make sure you have write permissions to the directory\n\
                   • Ensure the filename doesn't contain invalid characters: < > : \" / \\ | ? *\n\
                   • Try using a different output location",
                path
            )
        }
        CompressoError::FfmpegError(msg) => {
            &format!(
                "💡 FFmpeg encountered an error:\n\
                 \n\
                   Error: {}\n\
                 \n\
                   Possible solutions:\n\
                   • Try reducing quality or changing preset\n\
                   • Check if there's enough disk space\n\
                   • Verify the input video is not corrupted\n\
                   • Try updating FFmpeg to the latest version",
                msg
            )
        }
        CompressoError::Io(io_error) => {
            &format!(
                "💡 File system error:\n\
                 \n\
                   {}\n\
                 \n\
                   Common solutions:\n\
                   • Check available disk space\n\
                   • Verify you have read/write permissions\n\
                   • Close other programs that might be using the file\n\
                   • Try running with administrator/sudo privileges if needed",
                io_error
            )
        }
        CompressoError::Cancelled => {
            "💡 Compression was cancelled.\n\
             \n\
             You can start a new compression anytime."
        }
    };

    eprintln!("{}", hint.bright_blue());
    eprintln!();
}

/// Print warning message
pub fn print_warning(message: &str) {
    eprintln!(
        "{} {}",
        "⚠".bright_yellow().bold(),
        message.bright_yellow()
    );
}

/// Print info message
pub fn print_info(message: &str) {
    println!(
        "{} {}",
        "ℹ".bright_blue().bold(),
        message
    );
}

/// Print cancelled message
pub fn print_cancelled() {
    println!();
    println!(
        "{} {}",
        "⚠".bright_yellow().bold(),
        "Compression cancelled by user.".bright_yellow()
    );
    println!();
}

/// Estimate output file size range based on quality and preset
/// Returns (min_size, max_size) as a rough approximation for user guidance
/// Based on empirical data: Quality 70% typically produces ~1.5-3% of original size
pub fn estimate_output_size_range(original_size: u64, quality: u8, preset: Preset) -> (u64, u64) {
    // Modern video codecs (AV1/VP9) are extremely efficient
    // Base compression ratio formula derived from real-world data:
    // Quality 70% -> ~2-3% of original
    // Quality 50% -> ~4-5% of original
    // Quality 90% -> ~1.5-2.5% of original

    let quality_inv = (100 - quality) as f64 / 100.0;

    // Base ratio: higher quality = larger file (less compression)
    // Formula: 2% base + quality_inverse * 5%
    // Examples: Q70=3.5%, Q50=4.5%, Q90=2.5%
    let base_ratio = 0.02 + quality_inv * 0.05;

    // Preset has minimal impact on file size (mostly affects encoding speed)
    // Based on real data: Ironclad vs Thunderbolt differ by ~1-2%
    let preset_factor = match preset {
        Preset::Ironclad => 1.1,    // Slightly larger for better quality retention
        Preset::Thunderbolt => 0.95, // Slightly smaller, more aggressive
    };

    // Calculate base estimate
    let base_estimate = original_size as f64 * base_ratio * preset_factor;

    // Content variability is significant: screen recordings compress much better
    // than high-motion footage. Use ±70% range to account for this.
    let min_estimate = base_estimate * 0.3;  // Best case (simple content)
    let max_estimate = base_estimate * 1.7;  // Worst case (complex content)

    // Clamp to reasonable absolute range
    let absolute_min = (original_size as f64 * 0.005) as u64; // 0.5% minimum
    let absolute_max = (original_size as f64 * 0.50) as u64;  // 50% maximum

    let min_size = (min_estimate as u64).clamp(absolute_min, absolute_max);
    let max_size = (max_estimate as u64).clamp(absolute_min, absolute_max);

    (min_size, max_size)
}

// ============================================================================
// JSON Output
// ============================================================================

/// JSON output for video information
#[derive(Serialize)]
pub struct VideoInfoJson {
    pub path: String,
    pub size: u64,
    pub size_formatted: String,
    #[serde(flatten)]
    pub info: VideoInfo,
}

/// JSON output for compression result
#[derive(Serialize)]
pub struct CompressionResultJson {
    pub success: bool,
    pub elapsed_seconds: f64,
    #[serde(flatten)]
    pub result: CompressionResult,
    pub saved_bytes: u64,
    pub compression_ratio: f64,
}

/// Print video information as JSON
pub fn print_video_info_json(path: &str, info: &VideoInfo, size: u64) {
    let output = VideoInfoJson {
        path: path.to_string(),
        size,
        size_formatted: format_size(size),
        info: info.clone(),
    };

    match serde_json::to_string_pretty(&output) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing to JSON: {}", e),
    }
}

/// Print compression result as JSON
pub fn print_result_json(result: &CompressionResult, elapsed: std::time::Duration) {
    let saved = result.original_size.saturating_sub(result.compressed_size);
    let ratio = if result.original_size > 0 {
        (saved as f64 / result.original_size as f64) * 100.0
    } else {
        0.0
    };

    let output = CompressionResultJson {
        success: true,
        elapsed_seconds: elapsed.as_secs_f64(),
        result: result.clone(),
        saved_bytes: saved,
        compression_ratio: ratio,
    };

    match serde_json::to_string_pretty(&output) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing to JSON: {}", e),
    }
}

// ============================================================================
// Batch Processing Output
// ============================================================================

/// Result of processing a single file in batch
#[derive(Debug, Clone)]
pub struct BatchFileResult {
    pub input_path: String,
    pub success: bool,
    pub result: Option<CompressionResult>,
    pub error: Option<String>,
    pub elapsed: std::time::Duration,
}

/// Summary of batch processing
#[derive(Serialize)]
pub struct BatchSummary {
    pub total_files: usize,
    pub successful: usize,
    pub failed: usize,
    pub total_original_size: u64,
    pub total_compressed_size: u64,
    pub total_saved: u64,
    pub average_compression_ratio: f64,
    pub total_elapsed_seconds: f64,
    pub results: Vec<BatchFileResultJson>,
}

#[derive(Serialize)]
pub struct BatchFileResultJson {
    pub input_path: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<CompressionResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub elapsed_seconds: f64,
}

/// Print batch processing summary
pub fn print_batch_summary(results: &[BatchFileResult], total_elapsed: std::time::Duration) {
    println!();
    println!("{}", "━".repeat(50).dimmed());
    println!(
        "{} {}",
        "✓".bright_green().bold(),
        "Batch compression complete!".bright_green().bold()
    );
    println!();

    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;

    let mut total_original: u64 = 0;
    let mut total_compressed: u64 = 0;

    for result in results {
        if let Some(ref res) = result.result {
            total_original += res.original_size;
            total_compressed += res.compressed_size;
        }
    }

    let total_saved = total_original.saturating_sub(total_compressed);
    let avg_ratio = if total_original > 0 {
        (total_saved as f64 / total_original as f64) * 100.0
    } else {
        0.0
    };

    println!("{}", "Summary".bright_white().bold());
    println!("{}", "─".repeat(30).dimmed());
    println!(
        "  {} {}",
        "Total files:".dimmed(),
        results.len().to_string().bright_white()
    );
    println!(
        "  {} {}",
        "Successful:".dimmed(),
        successful.to_string().bright_green()
    );

    if failed > 0 {
        println!(
            "  {} {}",
            "Failed:".dimmed(),
            failed.to_string().bright_red()
        );
    }

    println!();
    println!(
        "  {} {}",
        "Total original:".dimmed(),
        format_size(total_original).bright_white()
    );
    println!(
        "  {} {}",
        "Total compressed:".dimmed(),
        format_size(total_compressed).bright_green()
    );
    println!(
        "  {} {} ({:.1}%)",
        "Total saved:".dimmed(),
        format_size(total_saved).bright_yellow(),
        avg_ratio
    );
    println!(
        "  {} {:.2}s",
        "Total time:".dimmed(),
        total_elapsed.as_secs_f64()
    );
    println!();

    // Show individual results
    println!("{}", "Individual Results".bright_white().bold());
    println!("{}", "─".repeat(30).dimmed());

    for (i, file_result) in results.iter().enumerate() {
        if file_result.success {
            if let Some(ref res) = file_result.result {
                let saved = res.original_size.saturating_sub(res.compressed_size);
                let ratio = if res.original_size > 0 {
                    (saved as f64 / res.original_size as f64) * 100.0
                } else {
                    0.0
                };

                println!(
                    "  {} {} → {} ({:.1}% saved)",
                    format!("[{}]", i + 1).dimmed(),
                    file_result.input_path.bright_cyan(),
                    format_size(res.compressed_size).bright_green(),
                    ratio
                );
            }
        } else {
            println!(
                "  {} {} - {}",
                format!("[{}]", i + 1).dimmed(),
                file_result.input_path.bright_red(),
                file_result.error.as_ref().unwrap_or(&"Unknown error".to_string()).bright_red()
            );
        }
    }

    println!();
    println!("{}", "━".repeat(50).dimmed());
    println!();
}

/// Print batch processing summary as JSON
pub fn print_batch_summary_json(results: &[BatchFileResult], total_elapsed: std::time::Duration) {
    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;

    let mut total_original: u64 = 0;
    let mut total_compressed: u64 = 0;

    for result in results {
        if let Some(ref res) = result.result {
            total_original += res.original_size;
            total_compressed += res.compressed_size;
        }
    }

    let total_saved = total_original.saturating_sub(total_compressed);
    let avg_ratio = if total_original > 0 {
        (total_saved as f64 / total_original as f64) * 100.0
    } else {
        0.0
    };

    let json_results: Vec<BatchFileResultJson> = results
        .iter()
        .map(|r| BatchFileResultJson {
            input_path: r.input_path.clone(),
            success: r.success,
            result: r.result.clone(),
            error: r.error.clone(),
            elapsed_seconds: r.elapsed.as_secs_f64(),
        })
        .collect();

    let summary = BatchSummary {
        total_files: results.len(),
        successful,
        failed,
        total_original_size: total_original,
        total_compressed_size: total_compressed,
        total_saved,
        average_compression_ratio: avg_ratio,
        total_elapsed_seconds: total_elapsed.as_secs_f64(),
        results: json_results,
    };

    match serde_json::to_string_pretty(&summary) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing to JSON: {}", e),
    }
}
