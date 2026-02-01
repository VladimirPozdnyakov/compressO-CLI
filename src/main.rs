mod cli;
mod domain;
mod error;
mod ffmpeg;
mod fs;
mod interactive;
mod localization;
mod output;
mod progress;

use clap::Parser;
use colored::Colorize;
use indicatif::ProgressBar;
use std::env;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use cli::Cli;
use domain::{CompressionConfig, CompressionResult};
use error::CompressoError;
use ffmpeg::FFmpeg;
use localization::{set_language, t};
use cli::LanguageArg;
use output::*;

fn main() {
    // Check if running without arguments - launch interactive mode
    let args: Vec<String> = env::args().collect();

    // Determine mode:
    // 1. No args -> interactive mode (prompt for file)
    // 2. Single arg that's a file path (not starting with -) -> interactive mode with file (drag & drop)
    // 3. Multiple file paths without flags -> interactive batch mode (drag & drop multiple files)
    // 4. Multiple args with flags -> CLI mode

    // Check if all args (except program name) are file paths (not flags)
    let all_files = args.len() > 1 && args[1..].iter().all(|arg| !arg.starts_with('-') && !arg.starts_with('/'));

    // Determine if we should run in interactive mode
    // Interactive if: no args, single non-flag arg, or multiple file args (not flags)
    // Also interactive if only language flag is provided without input files
    let has_language_flag = args.windows(2).any(|w| w[0] == "--language") ||
                           args.iter().any(|arg| arg.starts_with("--language="));

    // Check if there are any non-flag arguments that are not language values (potential input files)
    let has_non_flag_args = {
        let mut skip_next = false;
        let mut found_input = false;

        for i in 1..args.len() {
            if skip_next {
                skip_next = false;
                continue;
            }

            if args[i] == "--language" && i + 1 < args.len() {
                // Skip the next argument as it's the language value
                skip_next = true;
                continue;
            }

            if args[i].starts_with("--language=") {
                // This argument contains both flag and value
                continue;
            }

            if !args[i].starts_with('-') && !args[i].starts_with('/') {
                // Found a potential input file
                found_input = true;
                break;
            }
        }

        found_input
    };

    let is_interactive = args.len() == 1
        || (args.len() == 2 && !args[1].starts_with('-') && !args[1].starts_with('/'))
        || (args.len() > 2 && all_files)
        // Special case: if --language flag is provided without input files
        || (has_language_flag && !has_non_flag_args);

    // Determine language first by checking for --language flag in args
    // For interactive mode (no args), we need to handle parsing specially
    let config = if is_interactive {
        // Extract language from args if present
        let has_language_flag = args.windows(2).any(|w| w[0] == "--language") ||
                               args.iter().any(|arg| arg.starts_with("--language="));

        // Only set language from args if --language flag is provided
        if has_language_flag {
            let mut language_arg = LanguageArg::English; // default
            let mut i = 1;
            while i < args.len() {
                if args[i] == "--language" && i + 1 < args.len() {
                    if args[i + 1].to_lowercase() == "russian" {
                        language_arg = LanguageArg::Russian;
                    } else if args[i + 1].to_lowercase() == "english" {
                        language_arg = LanguageArg::English;
                    }
                    break;
                } else if args[i].starts_with("--language=") {
                    if args[i].contains("russian") {
                        language_arg = LanguageArg::Russian;
                    } else {
                        language_arg = LanguageArg::English;
                    }
                    break;
                }
                i += 1;
            }

            set_language(language_arg.into());
        }
        // If no language flag provided and this is initial launch (no args),
        // language will be set in interactive mode

        // Interactive mode
        if args.len() > 2 && all_files {
            // Multiple files drag & dropped -> batch interactive mode
            let files: Vec<String> = args[1..].iter().map(|s| s.clone()).collect();
            run_interactive_batch(files);
            return;
        }

        let provided_path = if args.len() == 2 {
            Some(args[1].clone())
        } else {
            None
        };

        match interactive::run_interactive(provided_path, args.len() == 1 && !has_language_flag) {
            Ok(Some(cfg)) => cfg,
            Ok(None) => {
                // User cancelled or empty input
                std::process::exit(0);
            }
            Err(e) => {
                print_error_with_hint(&e);
                interactive::wait_for_exit();
                std::process::exit(1);
            }
        }
    } else {
        // CLI mode - parse arguments
        let cli = Cli::parse();

        // Set language based on CLI argument
        set_language(cli.language.into());

        // Handle --info flag in CLI mode
        if cli.info {
            run_info_mode(&cli);
            return;
        }

        // Check if this is batch processing (multiple inputs or directory)
        let input_files = get_input_files(&cli);

        if input_files.is_empty() {
            print_error_with_hint(&CompressoError::FileNotFound("No input files specified".to_string()));
            std::process::exit(1);
        }

        // If multiple files, run batch processing
        if input_files.len() > 1 {
            run_batch_mode(&cli, input_files);
            return;
        }

        // Single file mode - use existing logic
        cli.to_config()
    };

    // Setup Ctrl+C handler
    let cancelled = Arc::new(AtomicBool::new(false));
    let cancelled_clone = cancelled.clone();

    ctrlc::set_handler(move || {
        cancelled_clone.store(true, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl+C handler");

    // Run the application
    if let Err(e) = run(config, cancelled) {
        match e {
            CompressoError::Cancelled => {
                print_cancelled();
                if is_interactive {
                    interactive::wait_for_exit();
                }
                std::process::exit(130); // Standard exit code for Ctrl+C
            }
            _ => {
                print_error_with_hint(&e);
                if is_interactive {
                    interactive::wait_for_exit();
                }
                std::process::exit(1);
            }
        }
    }

    // Wait for user input before closing in interactive mode
    if is_interactive {
        interactive::wait_for_exit();
    }
}

fn run_info_mode(cli: &Cli) {
    if !cli.json {
        print_header();
    }

    let input = cli.input.first().cloned().unwrap_or_default();

    if !fs::file_exists(&input) {
        if !cli.json {
            print_error_with_hint(&CompressoError::FileNotFound(input.clone()));
        } else {
            eprintln!("{{\"error\": \"File not found: {}\"}}", input);
        }
        std::process::exit(1);
    }

    let ffmpeg = match FFmpeg::new() {
        Ok(f) => f,
        Err(e) => {
            if !cli.json {
                print_error_with_hint(&e);
            } else {
                eprintln!("{{\"error\": \"{}\"}}", e);
            }
            std::process::exit(1);
        }
    };

    let video_info = match ffmpeg.get_video_info(&input) {
        Ok(info) => info,
        Err(e) => {
            if !cli.json {
                print_error_with_hint(&e);
            } else {
                eprintln!("{{\"error\": \"{}\"}}", e);
            }
            std::process::exit(1);
        }
    };

    let file_metadata = match fs::get_file_metadata(&input) {
        Ok(meta) => meta,
        Err(e) => {
            if !cli.json {
                print_error_with_hint(&e);
            } else {
                eprintln!("{{\"error\": \"{}\"}}", e);
            }
            std::process::exit(1);
        }
    };

    if cli.json {
        print_video_info_json(&input, &video_info, file_metadata.size);
    } else {
        print_video_info(&input, &video_info, file_metadata.size);
    }
}

fn run(config: CompressionConfig, cancelled: Arc<AtomicBool>) -> error::Result<CompressionResult> {
    // Print header (skip in JSON mode)
    if !config.json {
        print_header();
    }

    // Validate input file
    if !fs::file_exists(&config.input_path) {
        return Err(CompressoError::FileNotFound(config.input_path.clone()));
    }

    if !fs::is_video_file(&config.input_path) {
        return Err(CompressoError::InvalidInput(format!(
            "{} is not a valid video file",
            config.input_path
        )));
    }

    // Initialize FFmpeg
    let ffmpeg = FFmpeg::new()?;

    // Get video info
    let video_info = ffmpeg.get_video_info(&config.input_path)?;
    let file_metadata = fs::get_file_metadata(&config.input_path)?;

    // Determine output path
    let output_path = config.output_path.clone().unwrap_or_else(|| {
        let format = config.format.map(|f| f.extension());
        fs::generate_output_path(&config.input_path, format)
    });

    // Print video info and config (skip in JSON mode)
    if !config.json {
        print_video_info(&config.input_path, &video_info, file_metadata.size);
        print_config(&config, &output_path);
    }

    // Check for overwrite
    if !config.overwrite && fs::file_exists(&output_path) {
        if !config.json {
            print_warning(&format!(
                "Output file already exists: {}",
                output_path
            ));
            print_info("Use -y flag to overwrite.");
        }
        return Err(CompressoError::InvalidOutput(format!(
            "File already exists: {}",
            output_path
        )));
    }

    // Create progress bar (skip in JSON mode)
    let json_mode = config.json;
    let progress_bar = if !json_mode {
        create_progress_bar()
    } else {
        Arc::new(Mutex::new(ProgressBar::hidden()))
    };
    let progress_bar_clone = progress_bar.clone();

    // Start compression
    let start_time = std::time::Instant::now();

    let result = ffmpeg.compress_video(&config, Some(&video_info), cancelled.clone(), move |progress, current_frame, total_frames, fps, eta| {
        if !json_mode {
            update_progress(&progress_bar_clone, progress, current_frame, total_frames, fps, eta);
        }
    })?;

    let elapsed = start_time.elapsed();

    // Finish progress bar (skip in JSON mode)
    if !config.json {
        finish_progress(&progress_bar);
    }

    // Print result (only in non-batch mode - batch mode handles its own output)
    if !config.json {
        print_result(&result, elapsed);
    } else {
        print_result_json(&result, elapsed);
    }

    Ok(result)
}

/// Get list of input files from CLI arguments
fn get_input_files(cli: &Cli) -> Vec<String> {
    if let Some(ref dir) = cli.dir {
        // Process directory
        match fs::get_video_files_in_directory(dir) {
            Ok(files) => {
                if files.is_empty() {
                    eprintln!("No video files found in directory: {}", dir);
                }
                files
            }
            Err(e) => {
                print_error_with_hint(&e);
                Vec::new()
            }
        }
    } else {
        // Process individual files
        cli.input.clone()
    }
}

/// Run batch processing mode for multiple files
fn run_batch_mode(cli: &Cli, input_files: Vec<String>) {
    if !cli.json {
        print_header();
        println!("{}", format!("Processing {} files...", input_files.len()).bright_cyan().bold());
        println!();
    }

    let batch_start = std::time::Instant::now();
    let mut results = Vec::new();

    // Setup Ctrl+C handler
    let cancelled = Arc::new(AtomicBool::new(false));
    let cancelled_clone = cancelled.clone();

    ctrlc::set_handler(move || {
        cancelled_clone.store(true, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl+C handler");

    for (i, input_path) in input_files.iter().enumerate() {
        if !cli.json {
            println!(
                "{} Processing file {}/{}: {}",
                "→".bright_blue(),
                i + 1,
                input_files.len(),
                input_path.bright_white()
            );
        }

        let file_start = std::time::Instant::now();

        // Create config for this file
        let mut config = cli.to_config();
        config.input_path = input_path.clone();
        config.output_path = None; // Auto-generate for each file

        // Process the file
        let result = match run(config, cancelled.clone()) {
            Ok(compression_result) => {
                let elapsed = file_start.elapsed();
                output::BatchFileResult {
                    input_path: input_path.clone(),
                    success: true,
                    result: Some(compression_result),
                    error: None,
                    elapsed,
                }
            }
            Err(e) => {
                let elapsed = file_start.elapsed();
                if !cli.json {
                    eprintln!("  {} {}", "✗".bright_red(), e.to_string().bright_red());
                }
                output::BatchFileResult {
                    input_path: input_path.clone(),
                    success: false,
                    result: None,
                    error: Some(e.to_string()),
                    elapsed,
                }
            }
        };

        results.push(result);

        // Check if cancelled
        if cancelled.load(Ordering::Relaxed) {
            if !cli.json {
                print_cancelled();
            }
            break;
        }

        if !cli.json {
            println!();
        }
    }

    let batch_elapsed = batch_start.elapsed();

    // Print summary
    if cli.json {
        print_batch_summary_json(&results, batch_elapsed);
    } else {
        print_batch_summary(&results, batch_elapsed);
    }
}

/// Run interactive batch mode when multiple files are drag & dropped
fn run_interactive_batch(files: Vec<String>) {
    use dialoguer::{theme::ColorfulTheme, Input, Select};

    print_header();

    println!("{}", t("batch_compression_mode").bright_cyan().bold());
    println!("{}", "─".repeat(30).dimmed());
    println!();

    // Validate and filter video files
    let mut valid_files = Vec::new();
    let mut invalid_files = Vec::new();

    for file_path in files {
        // Clean up path (remove quotes)
        let cleaned = file_path.trim().trim_matches('"').trim_matches('\'').to_string();

        if !fs::file_exists(&cleaned) {
            invalid_files.push((cleaned, t("file_not_found")));
            continue;
        }

        if !fs::is_video_file(&cleaned) {
            invalid_files.push((cleaned, t("not_a_valid_video_file")));
            continue;
        }

        valid_files.push(cleaned);
    }

    // Show files to be processed
    println!("{} {}:", valid_files.len().to_string().bright_green(), t("video_files_found"));
    for (i, file) in valid_files.iter().enumerate() {
        println!("  {} {}", format!("[{}]", i + 1).dimmed(), file.bright_white());
    }

    if !invalid_files.is_empty() {
        println!();
        println!("{} {}:", invalid_files.len().to_string().bright_yellow(), t("files_will_be_skipped"));
        for (file, reason) in &invalid_files {
            println!("  {} {} - {}", "⚠".bright_yellow(), file.dimmed(), reason.bright_yellow());
        }
    }

    if valid_files.is_empty() {
        println!();
        println!("{}", t("no_valid_video_files").bright_red());
        interactive::wait_for_exit();
        return;
    }

    println!();

    let theme = ColorfulTheme::default();

    // Compression settings
    println!("{}", t("compression_settings").bright_white().bold());
    println!("{}", "─".repeat(30).dimmed());
    println!();

    // Preset
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
        1 => domain::Preset::Thunderbolt,
        _ => domain::Preset::Ironclad,
    };

    // Quality
    let quality: u8 = Input::with_theme(&theme)
        .with_prompt(t("quality_prompt"))
        .default(70)
        .interact()
        .unwrap_or(70)
        .clamp(0, 100);

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
    }

    // Confirm and start
    println!();
    println!("{}", t("header_separator").dimmed());

    let proceed_options = vec![t("no"), t("yes")];
    let proceed = Select::with_theme(&theme)
        .with_prompt(t("start_compression"))
        .items(&proceed_options)
        .default(1)
        .interact()
        .unwrap_or(1) == 1;

    if !proceed {
        println!("{}", t("compression_cancelled").bright_yellow());
        interactive::wait_for_exit();
        return;
    }

    println!();
    println!("{}", format!("{} {}...", t("processing"), valid_files.len()).bright_cyan().bold());
    println!();

    // Process files
    let batch_start = std::time::Instant::now();
    let mut results = Vec::new();

    // Setup Ctrl+C handler
    let cancelled = Arc::new(AtomicBool::new(false));
    let cancelled_clone = cancelled.clone();

    ctrlc::set_handler(move || {
        cancelled_clone.store(true, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl+C handler");

    for (i, input_path) in valid_files.iter().enumerate() {
        println!(
            "{} {} {}/{}: {}",
            "→".bright_blue(),
            t("processing"),
            i + 1,
            valid_files.len(),
            input_path.bright_white()
        );

        let file_start = std::time::Instant::now();

        // Create config for this file
        let config = CompressionConfig {
            input_path: input_path.clone(),
            output_path: None, // Auto-generate
            format: None,
            preset,
            quality,
            width,
            height,
            fps,
            mute,
            transforms: domain::VideoTransforms::default(),
            overwrite: true,
            verbose: false,
            json: false,
        };

        // Process the file
        let result = match run(config, cancelled.clone()) {
            Ok(compression_result) => {
                let elapsed = file_start.elapsed();
                output::BatchFileResult {
                    input_path: input_path.clone(),
                    success: true,
                    result: Some(compression_result),
                    error: None,
                    elapsed,
                }
            }
            Err(e) => {
                let elapsed = file_start.elapsed();
                eprintln!("  {} {}", "✗".bright_red(), e.to_string().bright_red());
                output::BatchFileResult {
                    input_path: input_path.clone(),
                    success: false,
                    result: None,
                    error: Some(e.to_string()),
                    elapsed,
                }
            }
        };

        results.push(result);

        // Check if cancelled
        if cancelled.load(Ordering::Relaxed) {
            print_cancelled();
            break;
        }

        println!();
    }

    let batch_elapsed = batch_start.elapsed();

    // Print summary
    print_batch_summary(&results, batch_elapsed);

    // Wait for exit
    interactive::wait_for_exit();
}
