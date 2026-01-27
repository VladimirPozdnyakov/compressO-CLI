mod cli;
mod domain;
mod error;
mod ffmpeg;
mod fs;
mod interactive;
mod output;
mod progress;

use clap::Parser;
use indicatif::ProgressBar;
use std::env;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use cli::Cli;
use domain::CompressionConfig;
use error::CompressoError;
use ffmpeg::FFmpeg;
use output::*;

fn main() {
    // Check if running without arguments - launch interactive mode
    let args: Vec<String> = env::args().collect();

    // Determine mode:
    // 1. No args -> interactive mode (prompt for file)
    // 2. Single arg that's a file path (not starting with -) -> interactive mode with file (drag & drop)
    // 3. Multiple args or flags -> CLI mode
    let is_interactive = args.len() == 1
        || (args.len() == 2 && !args[1].starts_with('-') && !args[1].starts_with('/'));

    let config = if is_interactive {
        // Interactive mode
        let provided_path = if args.len() == 2 {
            Some(args[1].clone())
        } else {
            None
        };

        match interactive::run_interactive(provided_path) {
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

        // Handle --info flag in CLI mode
        if cli.info {
            run_info_mode(&cli);
            return;
        }

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

    if !fs::file_exists(&cli.input) {
        if !cli.json {
            print_error_with_hint(&CompressoError::FileNotFound(cli.input.clone()));
        } else {
            eprintln!("{{\"error\": \"File not found: {}\"}}", cli.input);
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

    let video_info = match ffmpeg.get_video_info(&cli.input) {
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

    let file_metadata = match fs::get_file_metadata(&cli.input) {
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
        print_video_info_json(&cli.input, &video_info, file_metadata.size);
    } else {
        print_video_info(&cli.input, &video_info, file_metadata.size);
    }
}

fn run(config: CompressionConfig, cancelled: Arc<AtomicBool>) -> error::Result<()> {
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

    let result = ffmpeg.compress_video(&config, cancelled.clone(), move |progress, speed, eta| {
        if !json_mode {
            update_progress(&progress_bar_clone, progress, speed, eta);
        }
    })?;

    let elapsed = start_time.elapsed();

    // Finish progress bar (skip in JSON mode)
    if !config.json {
        finish_progress(&progress_bar);
    }

    // Print result
    if config.json {
        print_result_json(&result, elapsed);
    } else {
        print_result(&result, elapsed);
    }

    Ok(())
}
