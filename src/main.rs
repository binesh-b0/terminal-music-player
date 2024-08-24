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
    // Load config
    let config = Config::load();

    // Setup playlist
    let mut playlist = Playlist::new();
    playlist.add_track(Path::new("sample.mp3").to_path_buf());

    // Setup audio stream
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Arc::new(Mutex::new(Sink::try_new(&stream_handle).unwrap()));
    sink.lock().unwrap().set_volume(config.default_volume);

    let sink_clone = Arc::clone(&sink);
    thread::spawn(move || {
        if let Some(track) = playlist.current_track() {
            let source = load_audio_file(&track).unwrap();
            sink_clone.lock().unwrap().append(source);
            sink_clone.lock().unwrap().play();
        }
    });

    let start_time = Instant::now();
    let track_duration = Duration::from_secs(300); // Example duration, replace with actual duration

    // Terminal UI loop
    loop {
        // Display UI, handle input, etc.
        show_help_menu();
        display_progress(start_time, track_duration);

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
    }
}
