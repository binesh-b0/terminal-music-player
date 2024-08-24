mod playlist;
mod config;
mod controls;
mod ui;

use playlist::Playlist;
use config::Config;
use controls::{adjust_volume, display_progress};
use ui::{clear_screen, display_key_bindings, display_metadata, display_progress_bar, display_spinner};

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
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::default::get_probe;

fn load_audio_file(path: &Path) -> Result<(Decoder<BufReader<File>>, Duration, String), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let hint = Hint::new();
    let format_options = FormatOptions::default();
    let metadata_options = MetadataOptions::default();

    let mut probed = get_probe()
        .format(&hint, mss, &format_options, &metadata_options)?;

    let format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or("No valid track found")?;

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())?;

    let track_id = track.id;
    let mut title = "Unknown Title".to_string();

    // Extract metadata, such as the title
    if let Some(metadata) = probed.metadata.get() {
        for rev in metadata.current().iter() {
            for tag in rev.tags().iter() {
                if let Some(key) = tag.std_key {
                    if key == symphonia::core::meta::StandardTagKey::TrackTitle {
                        title = tag.value.to_string();
                        break;
                    }
                }
            }
        }
    }

    let duration = track.codec_params.n_frames.map_or_else(
        || Duration::from_secs(300),
        |frames| {
            let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
            Duration::from_secs(frames as u64 / sample_rate as u64)
        },
    );

    // This assumes you're using `rodio` for playback and creating a `Decoder`.
    let source = Decoder::new(BufReader::new(File::open(path)?))?;

    Ok((source, duration, title))
}

fn main() {
    let config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            return;
        }
    };

    let mut playlist = Playlist::new();
    playlist.add_track(Path::new("sample.mp3").to_path_buf());

    let (_stream, stream_handle) = match OutputStream::try_default() {
        Ok((stream, handle)) => (stream, handle),
        Err(e) => {
            eprintln!("Failed to initialize audio output: {}", e);
            return;
        }
    };

    let sink = Arc::new(Mutex::new(Sink::try_new(&stream_handle).unwrap()));
    sink.lock().unwrap().set_volume(config.default_volume);

    let (source, track_duration, title) = match playlist.current_track() {
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

    let mut spinner_pos = 0;

    clear_screen();
    display_metadata(&title, track_duration);
    display_key_bindings();

    loop {
        display_progress_bar(start_time.elapsed(), track_duration);
        display_spinner(spinner_pos);
        spinner_pos += 1;

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

        if sink.lock().unwrap().empty() {
            break;
        }
    }
}
