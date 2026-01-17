use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::io::{self, Write};

use crate::domain::{CompressionConfig, OutputFormat, Preset, VideoTransforms};
use crate::error::Result;
use crate::fs;

/// Wait for user to press Enter before exiting
pub fn wait_for_exit() {
    println!();
    println!("{}", "Press Enter to exit...".dimmed());
    let _ = io::stdout().flush();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
}

/// Run interactive mode - wizard for video compression
/// If input_path is provided, skip the file selection step
pub fn run_interactive(provided_path: Option<String>) -> Result<Option<CompressionConfig>> {
    print_interactive_header();

    // Step 1: Get input file path (or use provided one)
    let input_path = if let Some(path) = provided_path {
        // Clean up path (remove quotes that Windows adds when dragging)
        let cleaned = path.trim().trim_matches('"').trim_matches('\'').to_string();

        println!("{} {}", "File:".dimmed(), cleaned.bright_cyan());
        println!();

        cleaned
    } else {
        let path = prompt_input_path()?;

        if path.is_empty() {
            return Ok(None);
        }

        path
    };

    // Validate input file
    if !fs::file_exists(&input_path) {
        println!("{}", "File not found!".bright_red());
        wait_for_exit();
        return Ok(None);
    }

    if !fs::is_video_file(&input_path) {
        println!("{}", "This is not a valid video file!".bright_red());
        wait_for_exit();
        return Ok(None);
    }

    println!("{} {}", "Selected:".dimmed(), input_path.bright_green());
    println!();

    // Step 2: Compression settings
    let config = prompt_compression_settings(&input_path)?;

    Ok(Some(config))
}

fn print_interactive_header() {
    println!();
    println!("{}", "━".repeat(50).dimmed());
    println!(
        "{}",
        "  CompressO CLI v1.0.0 - Interactive Mode".bright_cyan().bold()
    );
    println!("{}", "━".repeat(50).dimmed());
    println!();
}

fn prompt_input_path() -> Result<String> {
    println!("{}", "Drag & drop video file here or enter path:".bright_white());
    println!("{}", "(Press Enter without input to exit)".dimmed());
    println!();

    let input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Video path")
        .allow_empty(true)
        .interact_text()
        .unwrap_or_default();

    // Clean up path (remove quotes that Windows adds when dragging)
    let cleaned = input
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string();

    Ok(cleaned)
}

fn prompt_compression_settings(input_path: &str) -> Result<CompressionConfig> {
    let theme = ColorfulTheme::default();

    // Preset selection
    println!("{}", "Compression Settings".bright_white().bold());
    println!("{}", "─".repeat(30).dimmed());
    println!();

    let presets = vec![
        "Thunderbolt (fast, good quality) [default]",
        "Ironclad (slow, best quality)",
    ];

    let preset_idx = Select::with_theme(&theme)
        .with_prompt("Select preset")
        .items(&presets)
        .default(0)
        .interact()
        .unwrap_or(0);

    let preset = match preset_idx {
        1 => Preset::Ironclad,
        _ => Preset::Thunderbolt,
    };

    // Quality
    let quality: u8 = Input::with_theme(&theme)
        .with_prompt("Quality (0-100, higher = better)")
        .default(70)
        .validate_with(|input: &u8| {
            if *input <= 100 {
                Ok(())
            } else {
                Err("Quality must be between 0 and 100")
            }
        })
        .interact()
        .unwrap_or(70);

    // Output format
    let formats = vec![
        "Keep original format [default]",
        "MP4",
        "WebM",
        "MKV",
        "AVI",
        "MOV",
    ];

    let format_idx = Select::with_theme(&theme)
        .with_prompt("Output format")
        .items(&formats)
        .default(0)
        .interact()
        .unwrap_or(0);

    let format = match format_idx {
        1 => Some(OutputFormat::Mp4),
        2 => Some(OutputFormat::Webm),
        3 => Some(OutputFormat::Mkv),
        4 => Some(OutputFormat::Avi),
        5 => Some(OutputFormat::Mov),
        _ => None,
    };

    // Advanced settings
    let show_advanced = Confirm::with_theme(&theme)
        .with_prompt("Configure advanced settings?")
        .default(false)
        .interact()
        .unwrap_or(false);

    let mut width: Option<u32> = None;
    let mut height: Option<u32> = None;
    let mut fps: Option<u32> = None;
    let mut mute = false;

    if show_advanced {
        println!();
        println!("{}", "Advanced Settings".bright_white().bold());
        println!("{}", "─".repeat(30).dimmed());
        println!("{}", "(Leave empty to keep original)".dimmed());
        println!();

        // Resolution
        let width_input: String = Input::with_theme(&theme)
            .with_prompt("Width (e.g., 1920)")
            .allow_empty(true)
            .interact_text()
            .unwrap_or_default();

        if !width_input.is_empty() {
            width = width_input.parse().ok();
        }

        let height_input: String = Input::with_theme(&theme)
            .with_prompt("Height (e.g., 1080)")
            .allow_empty(true)
            .interact_text()
            .unwrap_or_default();

        if !height_input.is_empty() {
            height = height_input.parse().ok();
        }

        // FPS
        let fps_input: String = Input::with_theme(&theme)
            .with_prompt("FPS (e.g., 30)")
            .allow_empty(true)
            .interact_text()
            .unwrap_or_default();

        if !fps_input.is_empty() {
            fps = fps_input.parse().ok();
        }

        // Mute
        mute = Confirm::with_theme(&theme)
            .with_prompt("Remove audio?")
            .default(false)
            .interact()
            .unwrap_or(false);
    }

    // Generate output path
    let output_path = fs::generate_output_path(input_path, format.map(|f| f.extension()));

    // Summary and confirmation
    println!();
    println!("{}", "━".repeat(50).dimmed());
    println!("{}", "Summary".bright_white().bold());
    println!("{}", "─".repeat(30).dimmed());
    println!("  {} {}", "Input:".dimmed(), input_path.bright_white());
    println!("  {} {}", "Output:".dimmed(), output_path.bright_cyan());
    println!(
        "  {} {}",
        "Preset:".dimmed(),
        match preset {
            Preset::Thunderbolt => "Thunderbolt".bright_green(),
            Preset::Ironclad => "Ironclad".bright_blue(),
        }
    );
    println!("  {} {}%", "Quality:".dimmed(), quality.to_string().bright_yellow());

    if let Some(f) = format {
        println!("  {} {}", "Format:".dimmed(), f.extension().bright_white());
    }

    if let (Some(w), Some(h)) = (width, height) {
        println!("  {} {}x{}", "Resolution:".dimmed(), w, h);
    }

    if let Some(f) = fps {
        println!("  {} {} fps", "FPS:".dimmed(), f);
    }

    if mute {
        println!("  {} {}", "Audio:".dimmed(), "muted".bright_red());
    }

    println!("{}", "━".repeat(50).dimmed());
    println!();

    let proceed = Confirm::with_theme(&theme)
        .with_prompt("Start compression?")
        .default(true)
        .interact()
        .unwrap_or(false);

    if !proceed {
        println!("{}", "Compression cancelled.".bright_yellow());
        std::process::exit(0);
    }

    println!();

    Ok(CompressionConfig {
        input_path: input_path.to_string(),
        output_path: Some(output_path),
        format,
        preset,
        quality,
        width,
        height,
        fps,
        mute,
        transforms: VideoTransforms::default(),
        overwrite: true,
        verbose: false,
    })
}
