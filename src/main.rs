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
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{event, execute};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use rodio::OutputStream;
use std::io::{self, Stdout};
use std::path::Path;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let config = Config::load();

    let mut playlist = Playlist::from_dir(Path::new(&config.playlist_directory)).unwrap_or_else(|_| Playlist::new());
    if playlist.is_empty() && Path::new("sample.mp3").exists() {
        playlist.add_track(Path::new("sample.mp3").to_path_buf());
    }
    if playlist.is_empty() {
        anyhow::bail!(
            "No audio files found. Put music in '{}' or add 'sample.mp3'.",
            config.playlist_directory
        );
    }

    let (_stream, stream_handle) = OutputStream::try_default().context("initialize audio output")?;
    let mut player = Player::new(stream_handle, config.default_volume);
    let mut app = App::new(playlist);

    let mut terminal = setup_terminal().context("setup terminal")?;
    let result = run_app(&mut terminal, &mut app, &mut player);
    restore_terminal(&mut terminal).ok();

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    player: &mut Player,
) -> anyhow::Result<()> {
    let tick_rate = Duration::from_millis(100);

    loop {
        terminal.draw(|frame| ui::draw(frame, app, player))?;

        app.on_tick(player)?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), _) => break,
                    (KeyCode::Char('h'), _) | (KeyCode::Char('?'), _) => app.toggle_help(),
                    (KeyCode::Up, _) => app.move_selection_up(),
                    (KeyCode::Down, _) => app.move_selection_down(),
                    (KeyCode::Enter, _) => app.play_selected(player)?,
                    (KeyCode::Char(' '), _) | (KeyCode::Char('p'), _) => app.toggle_pause(player),
                    (KeyCode::Char('s'), _) => {
                        player.stop();
                        app.status = "Stopped".to_string();
                    }
                    (KeyCode::Char('n'), _) => app.next_track(player)?,
                    (KeyCode::Char('b'), _) => app.previous_track(player)?,
                    (KeyCode::Left, KeyModifiers::SHIFT) => app.seek_backward(player, 30)?,
                    (KeyCode::Right, KeyModifiers::SHIFT) => app.seek_forward(player, 30)?,
                    (KeyCode::Left, _) => app.seek_backward(player, 5)?,
                    (KeyCode::Right, _) => app.seek_forward(player, 5)?,
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
