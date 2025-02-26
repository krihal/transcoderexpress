//! Transcode audio files to 16kHz mono WAV format.
//!
//! This program watches a directory for new audio files and transcodes them
//! to 16kHz mono WAV format.
//!
//! Run the program with the input and output directories as arguments:
//!
//! ```sh
//! cargo run -- -i input_dir -o output_dir
//! ```
//!
//! The program uses ffmpeg for transcoding, so make sure it is installed.
//!
use clap::Parser;
use log::{error, info};
use notify::{recommended_watcher, Event, EventKind::Create, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

/// Command line arguments.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "INPUT_DIR", required = true)]
    input_dir: Option<String>,
    #[arg(short, long, value_name = "OUTPUT_DIR", required = true)]
    output_dir: Option<String>,
}

/// Launches ffmpeg on a file and transcode it to 16kHz mono WAV format.
fn transcoder(path: &str, outpath: &str) {
    let filename = Path::new(path).file_name().unwrap().to_str().unwrap();
    let filename = filename.split('.').next().unwrap();
    let outfile = format!("{}/{}_transcoded.wav", outpath, filename);

    // Transcode the file to 16kHz mono WAV format
    let output = Command::new("ffmpeg")
        .args([
            "-i",
            path,
            "-ac",
            "1",
            "-ar",
            "16000",
            "-sample_fmt",
            "s16",
            &outfile,
        ])
        .output()
        .expect("Failed to execute ffmpeg");

    if output.status.success() {
        info!("Transcoding successful, saved to {}", outfile);
    } else {
        error!(
            "Transcoding failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// Consumer thread that processes files from the queue.
fn consumer_thread(rx: &Receiver<PathBuf>, outpath: &str) {
    loop {
        if let Ok(path) = rx.recv() {
            info!("Processing file: {:?}", path);
            transcoder(path.to_str().unwrap(), outpath);
            info!("Done processing file: {:?}", path);
        } else {
            error!("Error receiving file path.");
        }
    }
}

/// Handle file creation events.
fn handle_event(event: &Event, tx: &Sender<PathBuf>) {
    if let notify::Event {
        kind: Create(_),
        paths,
        ..
    } = event
    {
        for path in paths {
            info!("File created, adding to queue: {:?}", path);
            if let Err(e) = tx.send(path.to_path_buf()) {
                error!("Error sending path: {}", e);
            }
        }
    }
}

/// Main function.
fn main() -> std::io::Result<()> {
    let args = Cli::parse();
    let input_dir = args.input_dir.unwrap();
    let output_dir = args.output_dir.unwrap();

    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_target(false)
        .format_timestamp(Some(env_logger::TimestampPrecision::Millis))
        .init();

    // Create a channel for the consumer thread
    let (tx, rx) = channel();

    let mut watcher = recommended_watcher(move |res| match res {
        Ok(event) => handle_event(&event, &tx),
        Err(e) => println!("Watch error: {:?}", e),
    })
    .expect("Failed to create watcher");

    let _ = watcher.watch(Path::new(&input_dir), RecursiveMode::Recursive);

    info!("Watching directory: {}", input_dir);

    // Start consumer thread
    thread::spawn(move || {
        consumer_thread(&rx, &output_dir);
    });

    loop {
        thread::sleep(std::time::Duration::from_secs(1));
    }
}
