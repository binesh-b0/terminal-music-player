mod config;
mod controls;
mod playlist;
mod ui;

use config::Config;
use controls::adjust_volume;
use playlist::Playlist;
use ui::{
    clear_screen, display_key_bindings, display_metadata, display_progress_bar, display_spinner,
};

use crossterm::event::{self, KeyCode};
use rodio::{Decoder, OutputStream, Sink};
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use symphonia::core::codecs::CODEC_TYPE_NULL;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;

/// Loads an audio file and extracts metadata like duration and title.
///
/// # Arguments
///
/// * `path` - A reference to the path of the audio file.
///
/// # Returns
///
/// A tuple containing the audio source, track duration, and title.
/// On failure, returns an error.
fn load_audio_file(
    path: &Path,
) -> Result<(Decoder<BufReader<File>>, Duration, String), Box<dyn std::error::Error>> {
    // Open the audio file
    let file = File::open(path)?;

    // Create a media source stream from the file
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    // Hints and options for probing the format and metadata
    let hint = Hint::new();
    let format_options = FormatOptions::default();
    let metadata_options = MetadataOptions::default();

    // Probe the file format
    let mut probed = get_probe().format(&hint, mss, &format_options, &metadata_options)?;

    let format = probed.format;

    // Find the track with valid codec parameters
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or("No valid track found")?;

    // Initialize the decoder for the found track
    // let mut decoder = symphonia::default::get_codecs()
    //     .make(&track.codec_params, &DecoderOptions::default())?;

    let mut title = "Unknown Title".to_string();

    // Extract metadata such as the title
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
    print!("{}", title.to_string());

    // Calculate the duration of the track
    let duration = track.codec_params.n_frames.map_or_else(
        || Duration::from_secs(300),
        |frames| {
            let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
            Duration::from_secs(frames as u64 / sample_rate as u64)
        },
    );

    // Create a `Decoder` for `rodio` playback
    let source = Decoder::new(BufReader::new(File::open(path)?))?;

    Ok((source, duration, title))
}

fn main() {
    // Read the input file path from command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_audio_file>", args[0]);
        return;
    }

    // Get the file path and extract the file name
    let file_path = Path::new(&args[1]);
    let file_name = match file_path.file_name().and_then(OsStr::to_str) {
        Some(name) => name,
        None => {
            eprintln!("Failed to extract file name from path");
            "unknown"
        }
    };

    // Check if the provided path is a valid file
    if !file_path.is_file() {
        eprintln!(
            "The provided path is not a valid file: {}",
            file_path.display()
        );
        return;
    }

    // Load the configuration settings
    let config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            return;
        }
    };

    // Initialize the playlist and add the audio file to it
    let mut playlist = Playlist::new();
    playlist.add_track(file_path.to_path_buf());

    // Initialize the audio output stream
    let (_stream, stream_handle) = match OutputStream::try_default() {
        Ok((stream, handle)) => (stream, handle),
        Err(e) => {
            eprintln!("Failed to initialize audio output: {}", e);
            return;
        }
    };

    // Create a shared Sink for audio playback
    let sink = Arc::new(Mutex::new(Sink::try_new(&stream_handle).unwrap()));
    sink.lock().unwrap().set_volume(config.default_volume);

    // Load the audio file and extract its duration and title
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

    // Start playing the audio
    let start_time = Instant::now();
    sink.lock().unwrap().append(source);
    sink.lock().unwrap().play();

    let mut spinner_pos = 0;

    // Clear the terminal screen and display the initial UI components
    clear_screen();
    display_metadata(&title, track_duration, file_name);
    display_key_bindings();
    let mut is_playing = true;
    let mut last_press_time = Instant::now();
    // Main loop for updating the UI and handling user input
    loop {
        // Update the progress bar and spinner if playing
        if is_playing {
            display_progress_bar(start_time.elapsed(), track_duration);
            display_spinner(spinner_pos);
            spinner_pos += 1;
        }

        // Check for user input (e.g., play/pause, volume control)
        if event::poll(Duration::from_millis(500)).unwrap() {
            if let event::Event::Key(key) = event::read().unwrap() {
                // Debounce: check if enough time has passed since the last press
                if last_press_time.elapsed() < Duration::from_millis(300) {
                    continue;
                }
                last_press_time = Instant::now();

                match key.code {
                    KeyCode::Char('q') => {
                        clear_screen();
                        break;
                    }
                    KeyCode::Char('p') => {
                        let s = sink.lock().unwrap();
                        if !is_playing {
                            s.play();
                        } else {
                            s.pause();
                        }
                        is_playing = !is_playing;
                    }
                    event::KeyCode::Char('+') => adjust_volume(&sink.lock().unwrap(), true),
                    event::KeyCode::Char('-') => adjust_volume(&sink.lock().unwrap(), false),
                    _ => {}
                }
            }
        }

        // Exit the loop if the track has finished playing
        if sink.lock().unwrap().empty() {
            break;
        }
    }
}
