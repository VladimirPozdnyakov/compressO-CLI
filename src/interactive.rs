use colored::*;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::io::{self, Write};

use crate::domain::{CompressionConfig, CropCoordinates, FlipOptions, OutputFormat, Preset, VideoTransforms};
use crate::error::Result;
use crate::fs;
use crate::localization::t;

/// Wait for user to press Enter before exiting
pub fn wait_for_exit() {
    println!();
    println!("{}", t("press_enter_to_exit").dimmed());
    let _ = io::stdout().flush();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
}

/// Run interactive mode - wizard for video compression
/// If input_path is provided, skip the file selection step
pub fn run_interactive(provided_path: Option<String>, should_ask_language: bool) -> Result<Option<CompressionConfig>> {
    // Ask for language if this is the initial launch (no arguments provided)
    if should_ask_language {
        ask_language_selection()?;
    }

    print_interactive_header();

    // Step 1: Get input file path (or use provided one)
    let input_path = if let Some(path) = provided_path {
        // Clean up path (remove quotes that Windows adds when dragging)
        let cleaned = path.trim().trim_matches('"').trim_matches('\'').to_string();

        println!("{} {}", t("file").dimmed(), cleaned.bright_cyan());
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
        println!("{}", t("file_not_found").bright_red());
        wait_for_exit();
        return Ok(None);
    }

    if !fs::is_video_file(&input_path) {
        println!("{}", t("not_a_valid_video_file").bright_red());
        wait_for_exit();
        return Ok(None);
    }

    println!("{} {}", t("selected").dimmed(), input_path.bright_green());
    println!();

    // Step 2: Compression settings
    let config = prompt_compression_settings(&input_path)?;

    Ok(Some(config))
}

/// Ask user to select language
pub fn ask_language_selection() -> Result<()> {
    use dialoguer::{theme::ColorfulTheme, Select};
    use crate::localization::{set_language, Language};

    let theme = ColorfulTheme::default();
    let language_options = vec![
        "English",
        "Русский",
    ];

    let language_idx = Select::with_theme(&theme)
        .with_prompt("Select language / Выберите язык")
        .items(&language_options)
        .default(0)
        .interact()
        .unwrap_or(0);

    let language = if language_idx == 1 {
        Language::Russian
    } else {
        Language::English
    };

    set_language(language);

    Ok(())
}

fn print_interactive_header() {
    println!();
    println!("{}", t("header_separator").dimmed());
    println!(
        "{}",
        format!("  {} {} - {}", t("app_name"), t("app_version"), t("interactive_mode")).bright_cyan().bold()
    );
    println!("{}", t("header_separator").dimmed());
    println!();
}

fn prompt_input_path() -> Result<String> {
    println!("{}", t("drag_drop_video").bright_white());
    println!("{}", t("press_enter_without_input").dimmed());
    println!();

    let input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(t("video_path"))
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
    println!("{}", t("compression_settings").bright_white().bold());
    println!("{}", "─".repeat(30).dimmed());
    println!();

    let presets = vec![
        t("ironclad_slow_best_quality"),
        t("thunderbolt_fast_good_quality"),
    ];

    let preset_idx = Select::with_theme(&theme)
        .with_prompt(t("select_preset"))
        .items(&presets)
        .default(0)
        .interact()
        .unwrap_or(0);

    let preset = match preset_idx {
        1 => Preset::Thunderbolt,
        _ => Preset::Ironclad,
    };

    // Quality
    let quality: u8 = Input::with_theme(&theme)
        .with_prompt(t("quality_prompt"))
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
        t("keep_original_format"),
        t("mp4_format"),
        t("webm_format"),
        t("mkv_format"),
        t("avi_format"),
        t("mov_format"),
    ];

    let format_idx = Select::with_theme(&theme)
        .with_prompt(t("output_format"))
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
    let advanced_options = vec![t("no"), t("yes")];
    let show_advanced = Select::with_theme(&theme)
        .with_prompt(t("configure_advanced_settings"))
        .items(&advanced_options)
        .default(0)
        .interact()
        .unwrap_or(0) == 1;

    let mut width: Option<u32> = None;
    let mut height: Option<u32> = None;
    let mut fps: Option<u32> = None;
    let mut mute = false;
    let mut rotate: Option<i32> = None;
    let mut flip_horizontal = false;
    let mut flip_vertical = false;
    let mut crop: Option<CropCoordinates> = None;

    if show_advanced {
        println!();
        println!("{}", t("advanced_settings").bright_white().bold());
        println!("{}", "─".repeat(30).dimmed());
        println!("{}", t("leave_empty_keep_original").dimmed());
        println!();

        // Resolution
        let width_input: String = Input::with_theme(&theme)
            .with_prompt(t("width_prompt"))
            .allow_empty(true)
            .interact_text()
            .unwrap_or_default();

        if !width_input.is_empty() {
            width = width_input.parse().ok();
        }

        let height_input: String = Input::with_theme(&theme)
            .with_prompt(t("height_prompt"))
            .allow_empty(true)
            .interact_text()
            .unwrap_or_default();

        if !height_input.is_empty() {
            height = height_input.parse().ok();
        }

        // FPS
        let fps_input: String = Input::with_theme(&theme)
            .with_prompt(t("fps_prompt"))
            .allow_empty(true)
            .interact_text()
            .unwrap_or_default();

        if !fps_input.is_empty() {
            fps = fps_input.parse().ok();
        }

        // Mute
        let mute_options = vec![t("no"), t("yes")];
        let mute_idx = Select::with_theme(&theme)
            .with_prompt(t("remove_audio"))
            .items(&mute_options)
            .default(0)
            .interact()
            .unwrap_or(0);
        mute = mute_idx == 1;

        println!();
        println!("{}", t("transform_options").bright_white().bold());
        println!("{}", "─".repeat(30).dimmed());
        println!();

        // Rotate
        let rotation_options = vec![
            t("none_keep_original"),
            t("ninety_clockwise"),
            t("one_eighty"),
            t("two_seventy_clockwise"),
        ];

        let rotation_idx = Select::with_theme(&theme)
            .with_prompt(t("rotate_video"))
            .items(&rotation_options)
            .default(0)
            .interact()
            .unwrap_or(0);

        rotate = match rotation_idx {
            1 => Some(90),
            2 => Some(180),
            3 => Some(270),
            _ => None,
        };

        // Flip
        let flip_h_options = vec![t("no"), t("yes")];
        let flip_h_idx = Select::with_theme(&theme)
            .with_prompt(t("flip_horizontally"))
            .items(&flip_h_options)
            .default(0)
            .interact()
            .unwrap_or(0);
        flip_horizontal = flip_h_idx == 1;

        let flip_v_options = vec![t("no"), t("yes")];
        let flip_v_idx = Select::with_theme(&theme)
            .with_prompt(t("flip_vertically"))
            .items(&flip_v_options)
            .default(0)
            .interact()
            .unwrap_or(0);
        flip_vertical = flip_v_idx == 1;

        // Crop
        println!();
        println!("{}", t("crop_video").dimmed());
        println!("{}", t("crop_example").dimmed());

        let crop_input: String = Input::with_theme(&theme)
            .with_prompt("Crop")
            .allow_empty(true)
            .interact_text()
            .unwrap_or_default();

        if !crop_input.is_empty() {
            // Parse crop format: WxH:X:Y
            let parts: Vec<&str> = crop_input.split(':').collect();
            if parts.len() == 2 {
                let size_parts: Vec<&str> = parts[0].split('x').collect();
                let pos_parts: Vec<&str> = parts[1].split(':').collect();

                if size_parts.len() == 2 {
                    let crop_width = size_parts[0].parse().ok();
                    let crop_height = size_parts[1].parse().ok();
                    let crop_x = parts[1].split(':').next().and_then(|s| s.parse().ok()).unwrap_or(0);
                    let crop_y = if pos_parts.len() > 1 {
                        pos_parts[1].parse().ok().unwrap_or(0)
                    } else {
                        0
                    };

                    if let (Some(w), Some(h)) = (crop_width, crop_height) {
                        crop = Some(CropCoordinates {
                            width: w,
                            height: h,
                            x: crop_x,
                            y: crop_y,
                        });
                    }
                }
            }
        }
    }

    // Generate output path
    let output_path = fs::generate_output_path(input_path, format.map(|f| f.extension()));

    // Get file size for estimate
    let file_metadata = fs::get_file_metadata(input_path)?;
    let original_size = file_metadata.size;
    let (estimated_min, estimated_max) = crate::output::estimate_output_size_range(original_size, quality, preset);

    // Summary and confirmation
    println!();
    println!("{}", t("header_separator").dimmed());
    println!("{}", t("summary").bright_white().bold());
    println!("{}", "─".repeat(30).dimmed());
    println!("  {} {}", t("input").dimmed(), input_path.bright_white());
    println!("  {} {}", t("output").dimmed(), output_path.bright_cyan());
    println!(
        "  {} {}",
        t("preset").dimmed(),
        match preset {
            Preset::Thunderbolt => t("thunderbolt_preset").bright_green(),
            Preset::Ironclad => t("ironclad_preset").bright_blue(),
        }
    );
    println!("  {} {}%", t("quality").dimmed(), quality.to_string().bright_yellow());

    // Show size estimate range
    println!();
    println!(
        "  {} {}",
        t("original_size").dimmed(),
        fs::format_size(original_size).bright_white()
    );
    println!(
        "  {} {} - {}",
        t("est_output").dimmed(),
        fs::format_size(estimated_min).bright_cyan(),
        fs::format_size(estimated_max).bright_cyan()
    );
    let avg_estimated = (estimated_min + estimated_max) / 2;
    let savings_pct = ((original_size.saturating_sub(avg_estimated)) as f64 / original_size as f64) * 100.0;
    println!(
        "  {} ~{:.0}%",
        t("est_savings").dimmed(),
        savings_pct.to_string().bright_green()
    );

    if let Some(f) = format {
        println!("  {} {}", t("format").dimmed(), f.extension().bright_white());
    }

    if let (Some(w), Some(h)) = (width, height) {
        println!("  {} {}x{}", t("dimensions").dimmed(), w, h);
    }

    if let Some(f) = fps {
        println!("  {} {} fps", t("fps").dimmed(), f);
    }

    if mute {
        println!("  {} {}", t("audio").dimmed(), t("muted").bright_red());
    }

    // Display transforms if any
    if rotate.is_some() || flip_horizontal || flip_vertical || crop.is_some() {
        println!();
        if let Some(r) = rotate {
            println!("  {} {}°", t("rotate").dimmed(), r.to_string().bright_cyan());
        }

        if flip_horizontal || flip_vertical {
            let flip_desc = match (flip_horizontal, flip_vertical) {
                (true, true) => "horizontal + vertical",
                (true, false) => "horizontal",
                (false, true) => "vertical",
                _ => "",
            };
            println!("  {} {}", t("flip").dimmed(), flip_desc.bright_cyan());
        }

        if let Some(c) = &crop {
            println!(
                "  {} {}x{} at ({}, {})",
                t("crop").dimmed(),
                c.width.to_string().bright_cyan(),
                c.height.to_string().bright_cyan(),
                c.x,
                c.y
            );
        }
    }

    println!("{}", t("header_separator").dimmed());
    println!();

    let proceed_options = vec![t("no"), t("yes")];
    let proceed = Select::with_theme(&theme)
        .with_prompt(t("start_compression"))
        .items(&proceed_options)
        .default(1)
        .interact()
        .unwrap_or(1) == 1;

    if !proceed {
        println!("{}", t("compression_cancelled").bright_yellow());
        std::process::exit(0);
    }

    println!();

    // Build transforms from user input
    let flip = if flip_horizontal || flip_vertical {
        Some(FlipOptions {
            horizontal: flip_horizontal,
            vertical: flip_vertical,
        })
    } else {
        None
    };

    let transforms = VideoTransforms {
        crop,
        rotate,
        flip,
    };

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
        transforms,
        overwrite: true,
        verbose: false,
        json: false,
    })
}
