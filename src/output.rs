use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
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
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}% {msg}")
            .unwrap()
            .progress_chars("█▓░"),
    );
    pb.set_message("Compressing...");
    Arc::new(Mutex::new(pb))
}

/// Update progress bar
pub fn update_progress(pb: &Arc<Mutex<ProgressBar>>, progress: f64) {
    if let Ok(pb) = pb.lock() {
        pb.set_position(progress as u64);
    }
}

/// Finish progress bar
pub fn finish_progress(pb: &Arc<Mutex<ProgressBar>>) {
    if let Ok(pb) = pb.lock() {
        pb.finish_with_message("Done!");
    }
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

    let saved = if result.original_size > result.compressed_size {
        result.original_size - result.compressed_size
    } else {
        0
    };

    let ratio = if result.original_size > 0 {
        (saved as f64 / result.original_size as f64) * 100.0
    } else {
        0.0
    };

    println!(
        "  {} {}",
        "Original:".dimmed(),
        format_size(result.original_size).bright_white()
    );
    println!(
        "  {} {}",
        "Compressed:".dimmed(),
        format_size(result.compressed_size).bright_green()
    );
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

/// Print error message
pub fn print_error(message: &str) {
    eprintln!();
    eprintln!(
        "{} {}",
        "✗".bright_red().bold(),
        message.bright_red()
    );
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
