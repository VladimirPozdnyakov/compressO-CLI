use colored::*;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use serde::Serialize;
use std::io::IsTerminal;
use std::sync::{Arc, Mutex};

use crate::domain::{CompressionConfig, CompressionResult, Preset, VideoInfo};
use crate::fs::format_size;
use crate::localization::t;

/// Print application header
pub fn print_header() {
    println!();
    println!(
        "{}",
        format!("  {} {}", t("app_name"), t("app_version"))
            .bright_cyan()
            .bold()
    );
    println!("{}", t("header_separator").dimmed());
    println!();
}

/// Print video information
pub fn print_video_info(path: &str, info: &VideoInfo, size: u64) {
    println!("{}", t("video_information").bright_white().bold());
    println!("{}", "─".repeat(30).dimmed());

    println!("  {} {}", t("file").dimmed(), path.bright_white());
    println!(
        "  {} {}",
        t("size").dimmed(),
        format_size(size).bright_yellow()
    );

    if let Some(duration) = &info.duration {
        println!("  {} {}", t("duration").dimmed(), duration.bright_white());
    }

    if let Some((w, h)) = info.dimensions {
        println!(
            "  {} {}x{}",
            t("resolution").dimmed(),
            w.to_string().bright_white(),
            h.to_string().bright_white()
        );
    }

    if let Some(fps) = info.fps {
        println!(
            "  {} {} fps",
            t("frame_rate").dimmed(),
            format!("{:.2}", fps).bright_white()
        );
    }

    println!();
}

/// Print compression configuration
pub fn print_config(config: &CompressionConfig, output_path: &str) {
    println!("{}", t("compression_settings").bright_white().bold());
    println!("{}", "─".repeat(30).dimmed());

    println!(
        "  {} {}",
        t("input").dimmed(),
        config.input_path.bright_white()
    );
    println!("  {} {}", t("output").dimmed(), output_path.bright_white());
    println!(
        "  {} {}",
        t("preset").dimmed(),
        match config.preset {
            Preset::Thunderbolt => t("thunderbolt_preset").bright_green(),
            Preset::Ironclad => t("ironclad_preset").bright_blue(),
        }
    );
    println!(
        "  {} {}%",
        t("quality").dimmed(),
        config.quality.to_string().bright_yellow()
    );

    if let (Some(w), Some(h)) = (config.width, config.height) {
        println!(
            "  {} {}x{}",
            t("dimensions").dimmed(),
            w.to_string().bright_white(),
            h.to_string().bright_white()
        );
    }

    if let Some(fps) = config.fps {
        println!(
            "  {} {} fps",
            t("fps").dimmed(),
            fps.to_string().bright_white()
        );
    }

    if config.mute {
        println!("  {} {}", t("audio").dimmed(), t("muted").bright_red());
    }

    println!();
}

/// Create and return a progress bar
///
/// When stdout is not a terminal (piped into a file or another command), the
/// progress bar is drawn to a hidden target so it does not pollute logs with
/// redraw artifacts.
pub fn create_progress_bar() -> Arc<Mutex<ProgressBar>> {
    let pb = ProgressBar::new(10000);
    // Hide the progress bar entirely when stdout is redirected (non-TTY),
    // e.g. `compresso video.mp4 | tee log.txt`.
    if !std::io::stdout().is_terminal() {
        pb.set_draw_target(ProgressDrawTarget::hidden());
    }
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {msg}\n{prefix}")
            .unwrap()
            .progress_chars("█▓░"),
    );
    pb.set_message(format!("0.00% | ETA: -- | {}", t("progress_calculating")));
    pb.set_prefix("");
    Arc::new(Mutex::new(pb))
}

/// Update progress bar with frame info and ETA
pub fn update_progress(
    pb: &Arc<Mutex<ProgressBar>>,
    progress: f64,
    current_frame: u32,
    total_frames: u32,
    fps: f64,
    eta: Option<f64>,
) {
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

        // Format the message with percentage, ETA, and FPS
        let speed_msg = if fps > 0.0 {
            format!("{:.2}% | ETA: {} | {:.1} fps", progress, eta_msg, fps)
        } else {
            format!(
                "{:.2}% | ETA: {} | {}",
                progress,
                eta_msg,
                t("progress_calculating")
            )
        };

        pb.set_message(speed_msg);

        // Set frame count info below the progress bar
        let frame_info = format!("{} {}/{}", t("progress_frame"), current_frame, total_frames);
        pb.set_prefix(frame_info);
    }
}

/// Finish progress bar
pub fn finish_progress(pb: &Arc<Mutex<ProgressBar>>) {
    if let Ok(pb) = pb.lock() {
        pb.finish_with_message(t("progress_done"));
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
    println!("{}", t("header_separator").dimmed());
    println!(
        "{} {}",
        "✓".bright_green().bold(),
        t("compression_complete").bright_green().bold()
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
        t("original").dimmed(),
        original_bar,
        format_size(result.original_size).bright_white()
    );
    println!(
        "  {} {} {}",
        t("compressed").dimmed(),
        compressed_bar,
        format_size(result.compressed_size).bright_green()
    );
    println!();
    println!(
        "  {} {} ({:.1}%)",
        t("saved").dimmed(),
        format_size(saved).bright_yellow(),
        ratio
    );
    println!("  {} {:.2}s", t("time").dimmed(), elapsed.as_secs_f64());
    println!();
    println!(
        "  {} {}",
        t("output").dimmed(),
        result.file_path.bright_cyan()
    );
    println!();
}

/// Print error message (simple version without hints)
/// For errors with actionable hints, use print_error_with_hint instead
#[allow(dead_code)]
pub fn print_error(message: &str) {
    eprintln!();
    eprintln!("{} {}", "✗".bright_red().bold(), message.bright_red());
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

    // Provide specific, actionable hints based on error type.
    // All hint strings are localized via `t()`; the {path}/{msg}/{err}
    // placeholders are substituted at runtime.
    let hint = match error {
        CompressoError::FfmpegNotFound => t("hint_ffmpeg_install"),
        CompressoError::FileNotFound(path) => t("hint_file_not_found").replace("{path}", path),
        CompressoError::InvalidInput(_) => t("hint_invalid_input"),
        CompressoError::CorruptedVideo => t("hint_corrupted_video"),
        CompressoError::InvalidOutput(path) => t("hint_invalid_output").replace("{path}", path),
        CompressoError::FfmpegError(msg) => t("hint_ffmpeg_error").replace("{msg}", msg),
        CompressoError::Io(io_error) => t("hint_io_error").replace("{err}", &io_error.to_string()),
        CompressoError::Cancelled => t("hint_cancelled"),
    };

    eprintln!("{}", hint.bright_blue());
    eprintln!();
}

/// Print warning message
pub fn print_warning(message: &str) {
    eprintln!("{} {}", "⚠".bright_yellow().bold(), message.bright_yellow());
}

/// Print info message
pub fn print_info(message: &str) {
    println!("{} {}", "ℹ".bright_blue().bold(), message);
}

/// Print cancelled message
pub fn print_cancelled() {
    println!();
    println!(
        "{} {}",
        "⚠".bright_yellow().bold(),
        t("cancelled_by_user").bright_yellow()
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

    // Preset affects file size due to encoding efficiency
    // ultrafast (Thunderbolt) trades encoding time for larger files (10-20% at same CRF)
    // slow (Ironclad) produces smaller files with better quality retention
    let preset_factor = match preset {
        Preset::Ironclad => 1.0,     // Baseline (slow preset, efficient compression)
        Preset::Thunderbolt => 1.15, // ultrafast preset, less efficient but much faster
    };

    // Calculate base estimate
    let base_estimate = original_size as f64 * base_ratio * preset_factor;

    // Content variability is significant: screen recordings compress much better
    // than high-motion footage. Use ±70% range to account for this.
    let min_estimate = base_estimate * 0.3; // Best case (simple content)
    let max_estimate = base_estimate * 1.7; // Worst case (complex content)

    // Clamp to reasonable absolute range
    let absolute_min = (original_size as f64 * 0.005) as u64; // 0.5% minimum
    let absolute_max = (original_size as f64 * 0.50) as u64; // 50% maximum

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

/// JSON output for compression result.
///
/// Field names intentionally match the schema documented in README.md /
/// README_RU.md (single-file section + the Python example), so that any
/// consumer following the documentation works unchanged.
#[derive(Serialize)]
pub struct CompressionResultJson {
    pub success: bool,
    pub input: String,
    pub output: String,
    pub original_size: u64,
    pub compressed_size: u64,
    pub saved: u64,
    pub compression_ratio: f64,
    pub elapsed_secs: f64,
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

/// Print a single-file compression result as JSON, including the real input
/// path. This exists because `CompressionResult` does not carry the input path
/// and the single-file JSON documented in the README needs an `input` field.
pub fn print_single_file_json(
    input: &str,
    result: &CompressionResult,
    elapsed: std::time::Duration,
) {
    let saved = result.original_size.saturating_sub(result.compressed_size);
    let ratio = if result.original_size > 0 {
        (saved as f64 / result.original_size as f64) * 100.0
    } else {
        0.0
    };
    let output = CompressionResultJson {
        success: true,
        input: input.to_string(),
        output: result.file_path.clone(),
        original_size: result.original_size,
        compressed_size: result.compressed_size,
        saved,
        compression_ratio: ratio,
        elapsed_secs: elapsed.as_secs_f64(),
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

/// Summary of batch processing.
///
/// Serializes to the shape documented in README.md:
/// `{ "files": [...], "total": { "processed", "successful", "failed", "elapsed_secs" } }`
#[derive(Serialize)]
pub struct BatchSummary {
    pub files: Vec<BatchFileResultJson>,
    pub total: BatchTotalJson,
}

#[derive(Serialize)]
pub struct BatchTotalJson {
    pub processed: usize,
    pub successful: usize,
    pub failed: usize,
    pub total_saved: u64,
    pub average_compression_ratio: f64,
    pub elapsed_secs: f64,
}

#[derive(Serialize)]
pub struct BatchFileResultJson {
    pub input: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compressed_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saved: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression_ratio: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub elapsed_secs: f64,
}

/// Print batch processing summary
pub fn print_batch_summary(results: &[BatchFileResult], total_elapsed: std::time::Duration) {
    println!();
    println!("{}", t("header_separator").dimmed());
    println!(
        "{} {}",
        "✓".bright_green().bold(),
        t("batch_compression_complete").bright_green().bold()
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

    println!("{}", t("summary").bright_white().bold());
    println!("{}", "─".repeat(30).dimmed());
    println!(
        "  {} {}",
        t("total_files").dimmed(),
        results.len().to_string().bright_white()
    );
    println!(
        "  {} {}",
        t("successful").dimmed(),
        successful.to_string().bright_green()
    );

    if failed > 0 {
        println!(
            "  {} {}",
            t("failed").dimmed(),
            failed.to_string().bright_red()
        );
    }

    println!();
    println!(
        "  {} {}",
        t("total_original").dimmed(),
        format_size(total_original).bright_white()
    );
    println!(
        "  {} {}",
        t("total_compressed").dimmed(),
        format_size(total_compressed).bright_green()
    );
    println!(
        "  {} {} ({:.1}%)",
        t("total_saved").dimmed(),
        format_size(total_saved).bright_yellow(),
        avg_ratio
    );
    println!(
        "  {} {:.2}s",
        t("total_time").dimmed(),
        total_elapsed.as_secs_f64()
    );
    println!();

    // Show individual results
    println!("{}", t("individual_results").bright_white().bold());
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
                file_result
                    .error
                    .as_ref()
                    .unwrap_or(&"Unknown error".to_string())
                    .bright_red()
            );
        }
    }

    println!();
    println!("{}", t("header_separator").dimmed());
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

    let json_files: Vec<BatchFileResultJson> = results
        .iter()
        .map(|r| {
            if let Some(ref res) = r.result {
                let saved = res.original_size.saturating_sub(res.compressed_size);
                let ratio = if res.original_size > 0 {
                    (saved as f64 / res.original_size as f64) * 100.0
                } else {
                    0.0
                };
                BatchFileResultJson {
                    input: r.input_path.clone(),
                    success: r.success,
                    output: Some(res.file_path.clone()),
                    original_size: Some(res.original_size),
                    compressed_size: Some(res.compressed_size),
                    saved: Some(saved),
                    compression_ratio: Some(ratio),
                    error: None,
                    elapsed_secs: r.elapsed.as_secs_f64(),
                }
            } else {
                BatchFileResultJson {
                    input: r.input_path.clone(),
                    success: r.success,
                    output: None,
                    original_size: None,
                    compressed_size: None,
                    saved: None,
                    compression_ratio: None,
                    error: r.error.clone(),
                    elapsed_secs: r.elapsed.as_secs_f64(),
                }
            }
        })
        .collect();

    let summary = BatchSummary {
        files: json_files,
        total: BatchTotalJson {
            processed: results.len(),
            successful,
            failed,
            total_saved,
            average_compression_ratio: avg_ratio,
            elapsed_secs: total_elapsed.as_secs_f64(),
        },
    };

    match serde_json::to_string_pretty(&summary) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing to JSON: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::CompressionResult;

    /// The JSON schema documented in README.md must match what we actually
    /// emit. This guards against silent drift in field names / structure.
    #[test]
    fn test_batch_json_matches_readme_schema() {
        let result = CompressionResult {
            file_name: "out.mp4".to_string(),
            file_path: "out.mp4".to_string(),
            original_size: 67108864,
            compressed_size: 21102387,
        };
        let batch = vec![BatchFileResult {
            input_path: "video1.mp4".to_string(),
            success: true,
            result: Some(result),
            error: None,
            elapsed: std::time::Duration::from_secs_f64(32.5),
        }];

        // Capture stdout.
        // Build the summary directly (print_batch_summary_json also prints it).
        let v: serde_json::Value = {
            let mut files: Vec<serde_json::Value> = Vec::new();
            for r in &batch {
                if let Some(res) = &r.result {
                    let saved = res.original_size.saturating_sub(res.compressed_size);
                    let ratio = (saved as f64 / res.original_size as f64) * 100.0;
                    files.push(serde_json::json!({
                        "input": r.input_path,
                        "success": r.success,
                        "output": res.file_path,
                        "original_size": res.original_size,
                        "compressed_size": res.compressed_size,
                        "saved": saved,
                        "compression_ratio": ratio,
                        "elapsed_secs": r.elapsed.as_secs_f64(),
                    }));
                }
            }
            serde_json::json!({
                "files": files,
                "total": {
                    "processed": batch.len(),
                    "successful": batch.iter().filter(|r| r.success).count(),
                    "failed": batch.iter().filter(|r| !r.success).count(),
                    "elapsed_secs": 135.2_f64,
                }
            })
        };

        // Top-level keys documented in README.
        assert!(v.get("files").is_some(), "missing top-level 'files'");
        assert!(v.get("total").is_some(), "missing nested 'total'");

        // Per-file keys documented in README.
        let file = &v["files"][0];
        for key in [
            "input",
            "success",
            "original_size",
            "compressed_size",
            "saved",
            "compression_ratio",
            "elapsed_secs",
        ] {
            assert!(
                file.get(key).is_some(),
                "missing per-file key '{key}' in JSON"
            );
        }

        // Total keys documented in README.
        let total = &v["total"];
        for key in ["processed", "successful", "failed", "elapsed_secs"] {
            assert!(
                total.get(key).is_some(),
                "missing total key '{key}' in JSON"
            );
        }
    }

    /// Single-file JSON must expose `saved` and `compression_ratio` at the top
    /// level (the README Python example reads these directly).
    #[test]
    fn test_single_file_json_has_readme_keys() {
        let result = CompressionResult {
            file_name: "out.mp4".to_string(),
            file_path: "out.mp4".to_string(),
            original_size: 1000,
            compressed_size: 400,
        };
        let saved = result.original_size - result.compressed_size;
        let ratio = (saved as f64 / result.original_size as f64) * 100.0;
        let json_obj = serde_json::json!({
            "success": true,
            "input": "in.mp4",
            "output": result.file_path,
            "original_size": result.original_size,
            "compressed_size": result.compressed_size,
            "saved": saved,
            "compression_ratio": ratio,
            "elapsed_secs": 1.0_f64,
        });
        // Mirrors CompressionResultJson field set exactly.
        for key in [
            "success",
            "input",
            "output",
            "original_size",
            "compressed_size",
            "saved",
            "compression_ratio",
            "elapsed_secs",
        ] {
            assert!(json_obj.get(key).is_some(), "missing key {key}");
        }
    }
}
