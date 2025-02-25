use notify::EventKind::Create;
use notify::{RecursiveMode, Result, Watcher};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{channel, Sender};
use std::thread;

fn transcoder_thread(path: &str, outpath: &str) {
    let filename = Path::new(path).file_name().unwrap().to_str().unwrap();
    let filename = filename.split('.').next().unwrap();
    let outfile = format!("{}/{}_transcoded.wav", outpath, filename);
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
        println!("Transcoding successful, saved to {}", outfile);
    } else {
        println!("Transcoding failed.");
    }
}

fn consumer_thread(rx: std::sync::mpsc::Receiver<PathBuf>, outpath: &str) {
    loop {
        if let Ok(path) = rx.recv() {
            println!("Processing file: {:?}", path);
            transcoder_thread(path.to_str().unwrap(), outpath);
            println!("Done processing file: {:?}", path);
        } else {
            println!("Error receiving file path.");
        }
    }
}

fn handle_event(event: &notify::Event, tx: &Sender<PathBuf>) {
    match event {
        notify::Event {
            kind: Create(_),
            paths,
            ..
        } => {
            for path in paths {
                println!("File created: {:?}, adding to queue.", path);
                match tx.send(path.to_path_buf()) {
                    Ok(_) => {}
                    Err(e) => println!("Error sending path: {:?}", e),
                }
            }
        }
        _ => {}
    }
}

fn main() -> Result<()> {
    if std::env::args().len() != 3 {
        println!(
            "Usage: {} <in path> <out path>",
            std::env::args().nth(0).unwrap()
        );
        std::process::exit(1);
    }

    let outpath = std::env::args().nth(2).unwrap();
    let (tx, rx) = channel();
    let path = std::env::args().nth(1).unwrap();
    let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(event) => handle_event(&event, &tx),
        Err(e) => println!("Watch error: {:?}", e),
    })?;

    watcher.watch(Path::new(&path), RecursiveMode::Recursive)?;

    println!("Watching directory {}.", path);

    // Start consumer thread
    thread::spawn(move || {
        consumer_thread(rx, &outpath);
    });

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
