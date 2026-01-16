mod cli;
mod domain;
mod error;
mod ffmpeg;
mod fs;
mod output;

use clap::Parser;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use cli::Cli;
use error::CompressoError;
use ffmpeg::FFmpeg;
use output::*;

fn main() {
    let cli = Cli::parse();

    // Setup Ctrl+C handler
    let cancelled = Arc::new(AtomicBool::new(false));
    let cancelled_clone = cancelled.clone();

    ctrlc::set_handler(move || {
        cancelled_clone.store(true, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl+C handler");

    // Run the application
    if let Err(e) = run(cli, cancelled) {
        match e {
            CompressoError::Cancelled => {
                print_cancelled();
                std::process::exit(130); // Standard exit code for Ctrl+C
            }
            _ => {
                print_error(&e.to_string());
                std::process::exit(1);
            }
        }
    }
}

fn run(cli: Cli, cancelled: Arc<AtomicBool>) -> error::Result<()> {
    // Print header
    print_header();

    // Validate input file
    if !fs::file_exists(&cli.input) {
        return Err(CompressoError::FileNotFound(cli.input.clone()));
    }

    if !fs::is_video_file(&cli.input) {
        return Err(CompressoError::InvalidInput(format!(
            "{} is not a valid video file",
            cli.input
        )));
    }

    // Initialize FFmpeg
    let ffmpeg = FFmpeg::new()?;

    // Get video info
    let video_info = ffmpeg.get_video_info(&cli.input)?;
    let file_metadata = fs::get_file_metadata(&cli.input)?;

    // If --info flag, just show info and exit
    if cli.info {
        print_video_info(&cli.input, &video_info, file_metadata.size);
        return Ok(());
    }

    // Create config
    let config = cli.to_config();

    // Determine output path
    let output_path = config.output_path.clone().unwrap_or_else(|| {
        let format = config.format.map(|f| f.extension());
        fs::generate_output_path(&config.input_path, format)
    });

    // Print video info and config
    print_video_info(&cli.input, &video_info, file_metadata.size);
    print_config(&config, &output_path);

    // Check for overwrite
    if !config.overwrite && fs::file_exists(&output_path) {
        print_warning(&format!(
            "Output file already exists: {}",
            output_path
        ));
        print_info("Use -y flag to overwrite.");
        return Err(CompressoError::InvalidOutput(format!(
            "File already exists: {}",
            output_path
        )));
    }

    // Create progress bar
    let progress_bar = create_progress_bar();
    let progress_bar_clone = progress_bar.clone();

    // Start compression
    let start_time = std::time::Instant::now();

    let result = ffmpeg.compress_video(&config, cancelled.clone(), move |progress| {
        update_progress(&progress_bar_clone, progress);
    })?;

    let elapsed = start_time.elapsed();

    // Finish progress bar
    finish_progress(&progress_bar);

    // Print result
    print_result(&result, elapsed);

    Ok(())
}
