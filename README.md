# Terminal Music Player

A sleek terminal-based music player built with Rust. It scans a local music folder (configurable via `config.toml`) and provides a modern TUI with playlist navigation and playback controls.

## Features

- Modern TUI (Ratatui + Crossterm)
- Playlist browser (loads audio files from a directory)
- Play/pause/stop, next/previous
- Seek (±5s, ±30s)
- Volume + mute
- Shuffle + repeat (Off/All/One)

## Installation

1. **Clone the repository:**

   ```bash
   git clone https://github.com/binesh-b0/terminal-music-player.git
   cd terminal-music-player
   ```

2. **Build the project:**

   Ensure you have Rust installed. Then, run:

   ```bash
   cargo build --release
   ```

3. **Run the application:**

   ```bash
   cargo run
   ```

## Usage

The player loads audio files from `playlist_directory` in `config.toml`. If the folder is empty, it falls back to `sample.mp3` if present.

You can also pass files/directories as arguments:

- `cargo run -- music/`
- `cargo run -- path/to/song.mp3`

- **↑/↓**: Move selection
- **Enter**: Play selected track
- **Space / P**: Play/Pause
- **S**: Stop
- **N / B**: Next / Previous
- **← / →**: Seek -/+ 5s
- **Shift + ← / →**: Seek -/+ 30s
- **+ / -**: Volume up/down
- **M**: Mute
- **Z**: Toggle shuffle
- **R**: Cycle repeat (Off/All/One)
- **H / ?**: Help
- **Q**: Quit

## Learning Goals

- Understanding Rust's memory management and concurrency model
- Working with external crates and libraries
- Building a terminal UI (TUI) application

## Next Steps

- Add in-app search/filter
- Add metadata (artist/album) + duration list
- Improve queue management and persistence

## Contributions

This project is primarily for learning purposes, but contributions and suggestions are welcome!

## License

This project is licensed under the MIT License.
