mod app;
mod config;
mod player;
mod playlist;
mod ui;

use crate::app::App;
use crate::config::Config;
use crate::player::Player;
use crate::playlist::Playlist;
use anyhow::Context;
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, execute};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use rodio::OutputStream;
use std::env;
use std::io::{self, Stdout};
use std::path::{Path, PathBuf};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let config = Config::load();

    let mut playlist = build_playlist(&config);
    if playlist.is_empty() && Path::new("sample.mp3").exists() {
        playlist.add_track(Path::new("sample.mp3").to_path_buf());
    }
    if playlist.is_empty() {
        anyhow::bail!("No audio files found. Configure `playlist_directory`, pass a file/dir as an argument, or add `sample.mp3`.");
    }

    let (_stream, stream_handle) =
        OutputStream::try_default().context("initialize audio output")?;
    let mut player = Player::new(stream_handle, config.default_volume);
    let mut app = App::new(playlist);

    let mut terminal = setup_terminal().context("setup terminal")?;
    if let Err(err) = app.play_current(&mut player) {
        app.status = format!("Error: {err}");
    }
    let result = run_app(&mut terminal, &mut app, &mut player);
    restore_terminal(&mut terminal).ok();

    result
}

fn build_playlist(config: &Config) -> Playlist {
    let args: Vec<PathBuf> = env::args_os().skip(1).map(PathBuf::from).collect();
    if args.is_empty() {
        return Playlist::from_dir(Path::new(&config.playlist_directory))
            .unwrap_or_else(|_| Playlist::new());
    }

    let mut playlist = Playlist::new();
    for path in args {
        if path.is_dir() {
            if let Ok(dir_playlist) = Playlist::from_dir(&path) {
                playlist.add_tracks(dir_playlist.tracks().iter().cloned());
            }
        } else if path.is_file() {
            playlist.add_track(path);
        }
    }

    playlist
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    player: &mut Player,
) -> anyhow::Result<()> {
    let tick_rate = Duration::from_millis(100);

    loop {
        terminal.draw(|frame| ui::draw(frame, app, player))?;

        if let Err(err) = app.on_tick(player) {
            app.status = format!("Error: {err}");
        }

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match (key.code, key.modifiers) {
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
                    (KeyCode::Char('q'), _) => break,
                    (KeyCode::Esc, _) => app.show_help = false,
                    (KeyCode::Char('h'), _) | (KeyCode::Char('?'), _) => app.toggle_help(),
                    (KeyCode::Up, _) => app.move_selection_up(),
                    (KeyCode::Down, _) => app.move_selection_down(),
                    (KeyCode::Home, _) => app.selection = 0,
                    (KeyCode::End, _) => {
                        if !app.playlist.is_empty() {
                            app.selection = app.playlist.len() - 1;
                        }
                    }
                    (KeyCode::Enter, _) => {
                        if let Err(err) = app.play_selected(player) {
                            app.status = format!("Error: {err}");
                        }
                    }
                    (KeyCode::Char(' '), _) | (KeyCode::Char('p'), _) => {
                        if player.state() == crate::player::PlaybackState::Stopped {
                            if let Err(err) = app.play_selected(player) {
                                app.status = format!("Error: {err}");
                            }
                        } else {
                            app.toggle_pause(player);
                        }
                    }
                    (KeyCode::Char('s'), _) => {
                        player.stop();
                        app.status = "Stopped".to_string();
                    }
                    (KeyCode::Char('n'), _) => {
                        if let Err(err) = app.next_track(player) {
                            app.status = format!("Error: {err}");
                        }
                    }
                    (KeyCode::Char('b'), _) => {
                        if let Err(err) = app.previous_track(player) {
                            app.status = format!("Error: {err}");
                        }
                    }
                    (KeyCode::Left, KeyModifiers::SHIFT) => {
                        if let Err(err) = app.seek_backward(player, 30) {
                            app.status = format!("Error: {err}");
                        }
                    }
                    (KeyCode::Right, KeyModifiers::SHIFT) => {
                        if let Err(err) = app.seek_forward(player, 30) {
                            app.status = format!("Error: {err}");
                        }
                    }
                    (KeyCode::Left, _) => {
                        if let Err(err) = app.seek_backward(player, 5) {
                            app.status = format!("Error: {err}");
                        }
                    }
                    (KeyCode::Right, _) => {
                        if let Err(err) = app.seek_forward(player, 5) {
                            app.status = format!("Error: {err}");
                        }
                    }
                    (KeyCode::Char('+'), _) | (KeyCode::Char('='), _) => {
                        player.adjust_volume(0.05);
                        app.status = "Volume up".to_string();
                    }
                    (KeyCode::Char('-'), _) => {
                        player.adjust_volume(-0.05);
                        app.status = "Volume down".to_string();
                    }
                    (KeyCode::Char('m'), _) => {
                        player.toggle_mute();
                        app.status = if player.is_muted() {
                            "Muted".to_string()
                        } else {
                            "Unmuted".to_string()
                        };
                    }
                    (KeyCode::Char('z'), _) => {
                        app.toggle_shuffle();
                        app.status = if app.shuffle {
                            "Shuffle on".to_string()
                        } else {
                            "Shuffle off".to_string()
                        };
                    }
                    (KeyCode::Char('r'), _) => {
                        app.cycle_repeat();
                        app.status = format!("Repeat: {}", app.repeat.label());
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn setup_terminal() -> anyhow::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
