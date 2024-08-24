# Terminal Music Player


The Terminal Music Player is a simple, terminal-based music player written in Rust. It allows you to play audio files directly from the terminal with a minimalistic and interactive user interface. The player supports various audio formats, including MP3, AAC, and FLAC, and provides basic features such as play/pause, volume control, and track navigation within a playlist. The UI includes a progress bar, metadata display, and key bindings for easy control.

## Features

- **Play Audio Files:** Supports multiple audio formats including MP3, AAC, and FLAC.
- **Playlist Management:** Add multiple tracks to a playlist and navigate through them.
- **Volume Control:** Increase or decrease the volume using keyboard shortcuts.
- **Progress Bar:** Visual progress indicator showing the elapsed time and total duration of the track.
- **Spinner Animation:** An animated spinner indicating ongoing playback.
- **Configurable Settings:** Default volume and playlist directory can be configured via a TOML file.

## Structure

- **`main.rs`:** The entry point of the application. Handles the main logic including loading audio files, initializing the UI, and managing playback controls.
- **`playlist.rs`:** Manages the playlist functionality, including adding tracks and navigating between them.
- **`controls.rs`:** Contains functions related to controlling playback, such as adjusting the volume.
- **`ui.rs`:** Handles the terminal-based UI, including displaying metadata, the progress bar, and key bindings.
- **`config.rs`:** Loads configuration settings from a `config.toml` file.

## Installation

1. **Clone the repository:**

   ```bash
   git clone https://github.com/binesh-b0/terminal-music-player.git
   cd terminal-music-player
   ```
2. **Build the project:**

   ```bash
   cargo build --release
   ```
3. **Run the application:**

   ```bash
   cargo run <path_to_audio_file>
   ```

Replace `<path_to_audio_file>` with the path to the audio file you want to play.

### Key Bindings

- **`q`**: Quit the player.
- **`p`**: Toggle play/pause.
- **`+`**: Increase volume.
- **`-`**: Decrease volume.

### Configuration

The player can be configured using a `config.toml` file. The configuration file allows you to set the default volume and playlist directory.

**Example `config.toml`:**

```toml
default_volume = 0.5
playlist_directory = "/path/to/your/music/directory"
```

### Adding Tracks to Playlist

Tracks can be added to the playlist by providing their file paths. The player will automatically play the first track added to the playlist.

### Dependencies

- **`rodio`**: For audio playback.
- **`crossterm`**: For terminal UI manipulation.
- **`serde`**: For parsing configuration files.
- **`toml`**: For reading and parsing the configuration file.
- **`symphonia`**: For audio format support.
- **`cpal`**: Low-level audio I/O library used by `rodio`.
- **`winapi`**: Windows API bindings.
- **`unicode-segmentation`**: Used for handling Unicode strings.

#### Not yet implemented

- next, previous track
- playlist directories
