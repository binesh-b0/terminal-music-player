mod playlist;
mod config;
mod controls;

use playlist::Playlist;
use config::Config;
use controls::{adjust_volume, display_progress, show_help_menu};

use rodio::{Decoder, OutputStream, Sink};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::path::Path;
use std::io::BufReader;
use std::fs::File;
use crossterm::event;

fn load_audio_file(path: &Path) -> Result<Decoder<BufReader<File>>, rodio::decoder::DecoderError> {
    let file = File::open(path).map_err(|e| rodio::decoder::DecoderError::IoError(e.to_string()))?;
    let source = Decoder::new(BufReader::new(file))?;
    Ok(source)
}


fn main() {
    let config = Config::load();

    let mut playlist = match Playlist::from_dir(Path::new(&config.playlist_directory)) {
        Ok(playlist) => playlist,
        Err(err) => {
            eprintln!(
                "Failed to read playlist directory '{}': {err}",
                config.playlist_directory
            );
            Playlist::new()
        }
    };

    if playlist.is_empty() && Path::new("sample.mp3").exists() {
        playlist.add_track(Path::new("sample.mp3").to_path_buf());
    }

    let (_stream, stream_handle) = match OutputStream::try_default() {
        Ok(parts) => parts,
        Err(err) => {
            eprintln!("Failed to initialize audio output: {err}");
            return;
        }
    };

    let sink = match Sink::try_new(&stream_handle) {
        Ok(sink) => Arc::new(Mutex::new(sink)),
        Err(err) => {
            eprintln!("Failed to create audio sink: {err}");
            return;
        }
    };

    if let Ok(locked) = sink.lock() {
        locked.set_volume(config.default_volume);
    }

    let sink_clone = Arc::clone(&sink);
    thread::spawn(move || {
        if let Some(track) = playlist.current_track() {
            match load_audio_file(&track) {
                Ok(source) => {
                    if let Ok(locked) = sink_clone.lock() {
                        locked.append(source);
                        locked.play();
                    }
                }
                Err(err) => {
                    eprintln!("Failed to load audio file '{}': {err}", track.display());
                }
            }
        }
    });

    let start_time = Instant::now();
    let track_duration = Duration::from_secs(300); // Example duration, replace with actual duration

    // Terminal UI loop
    loop {
        // Display UI, handle input, etc.
        show_help_menu();
        display_progress(start_time, track_duration);

        if event::poll(Duration::from_millis(500)).unwrap_or(false) {
            if let Ok(event::Event::Key(key)) = event::read() {
                match key.code {
                    event::KeyCode::Char('q') => break,
                    event::KeyCode::Char('p') => {
                        if let Ok(s) = sink.lock() {
                            if s.is_paused() {
                                s.play();
                            } else {
                                s.pause();
                            }
                        }
                    }
                    event::KeyCode::Char('+') => {
                        if let Ok(s) = sink.lock() {
                            adjust_volume(&s, true);
                        }
                    }
                    event::KeyCode::Char('-') => {
                        if let Ok(s) = sink.lock() {
                            adjust_volume(&s, false);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
