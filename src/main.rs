mod playlist;
mod config;
mod controls;

use playlist::Playlist;
use config::Config;
use controls::{adjust_volume, display_progress, show_help_menu};

use rodio::{Decoder, OutputStream, Sink};
use crossterm::event;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::path::Path;
use std::io::BufReader;
use std::fs::File;
use symphonia::core::formats::FormatOptions;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;

fn load_audio_file(path: &Path) -> Result<(Decoder<BufReader<File>>, Duration), Box<dyn std::error::Error>> {
    // Use symphonia to extract metadata, especially duration
    let file = File::open(path).map_err(|e| {
        eprintln!("Failed to open file {}: {}", path.display(), e);
        e
    })?;
    
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let hint = Hint::new();
    let probed = get_probe().format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| {
            eprintln!("Failed to probe format for {}: {}", path.display(), e);
            e
        })?;
    
    let format = probed.format;
    let track = format.default_track().ok_or("No default track found")?;

    let duration = track.codec_params.n_frames.map(|frames| {
        let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
        Duration::from_secs(frames as u64 / sample_rate as u64)
    }).unwrap_or(Duration::from_secs(300)); // Fallback if duration isn't available

    // Use rodio for decoding and playback
    let source = Decoder::new(BufReader::new(File::open(path).map_err(|e| {
        eprintln!("Failed to open file for decoding {}: {}", path.display(), e);
        e
    })?)).map_err(|e| {
        eprintln!("Failed to decode file {}: {}", path.display(), e);
        e
    })?;
    
    Ok((source, duration))
}

fn main() {
    // Load config
    let config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            return;
        }
    };

    // Setup playlist
    let mut playlist = Playlist::new();
    playlist.add_track(Path::new("sample.mp3").to_path_buf());

    // Setup audio stream
    let (_stream, stream_handle) = match OutputStream::try_default() {
        Ok((stream, handle)) => (stream, handle),
        Err(e) => {
            eprintln!("Failed to initialize audio output: {}", e);
            return;
        }
    };
    
    let sink = Arc::new(Mutex::new(Sink::try_new(&stream_handle).unwrap()));
    sink.lock().unwrap().set_volume(config.default_volume);

    let (source, track_duration) = match playlist.current_track() {
        Some(track) => match load_audio_file(track) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Failed to load audio file: {}", e);
                return;
            }
        },
        None => {
            eprintln!("No track available in the playlist.");
            return;
        }
    };

    let start_time = Instant::now();
    sink.lock().unwrap().append(source);
    sink.lock().unwrap().play();

    // Terminal UI loop
    loop {
        // Display UI, handle input, etc.
        display_progress(start_time, track_duration);
        show_help_menu();

        if event::poll(Duration::from_millis(500)).unwrap() {
            if let event::Event::Key(key) = event::read().unwrap() {
                match key.code {
                    event::KeyCode::Char('q') => break,
                    event::KeyCode::Char('p') => {
                        let s = sink.lock().unwrap();
                        if s.is_paused() {
                            s.play();
                        } else {
                            s.pause();
                        }
                    }
                    event::KeyCode::Char('+') => adjust_volume(&sink.lock().unwrap(), true),
                    event::KeyCode::Char('-') => adjust_volume(&sink.lock().unwrap(), false),
                    _ => {}
                }
            }
        }

        // Exit loop if the track is finished
        if sink.lock().unwrap().empty() {
            break;
        }
    }
}
