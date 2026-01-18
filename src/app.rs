use crate::player::{PlaybackState, Player};
use crate::playlist::Playlist;
use rand::seq::SliceRandom;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    Off,
    All,
    One,
}

impl RepeatMode {
    pub fn next(self) -> Self {
        match self {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            RepeatMode::Off => "Off",
            RepeatMode::All => "All",
            RepeatMode::One => "One",
        }
    }
}

pub struct App {
    pub playlist: Playlist,
    pub selection: usize,
    pub shuffle: bool,
    pub repeat: RepeatMode,
    pub show_help: bool,
    pub status: String,
}

impl App {
    pub fn new(playlist: Playlist) -> Self {
        Self {
            playlist,
            selection: 0,
            shuffle: false,
            repeat: RepeatMode::All,
            show_help: false,
            status: String::new(),
        }
    }

    pub fn selected_track(&self) -> Option<&PathBuf> {
        self.playlist.track_at(self.selection)
    }

    pub fn move_selection_up(&mut self) {
        if self.playlist.is_empty() {
            self.selection = 0;
            return;
        }
        self.selection = self.selection.saturating_sub(1);
    }

    pub fn move_selection_down(&mut self) {
        if self.playlist.is_empty() {
            self.selection = 0;
            return;
        }
        self.selection = (self.selection + 1).min(self.playlist.len().saturating_sub(1));
    }

    pub fn play_selected(&mut self, player: &mut Player) -> anyhow::Result<()> {
        let Some(track) = self.selected_track().cloned() else {
            self.status = "Playlist is empty".to_string();
            return Ok(());
        };

        self.playlist.set_current_index(self.selection);
        player.play_track(track)?;
        self.status = "Playing".to_string();
        Ok(())
    }

    pub fn play_current(&mut self, player: &mut Player) -> anyhow::Result<()> {
        let Some(track) = self.playlist.current_track().cloned() else {
            return Ok(());
        };
        player.play_track(track)?;
        Ok(())
    }

    pub fn next_track(&mut self, player: &mut Player) -> anyhow::Result<()> {
        if self.playlist.is_empty() {
            return Ok(());
        }

        if self.shuffle && self.playlist.len() > 1 {
            let mut indices: Vec<usize> = (0..self.playlist.len()).collect();
            indices.retain(|i| *i != self.playlist.current_index());
            let mut rng = rand::thread_rng();
            if let Some(next_index) = indices.choose(&mut rng).copied() {
                self.playlist.set_current_index(next_index);
            }
        } else {
            self.playlist.next();
        }

        self.selection = self.playlist.current_index();
        self.play_current(player)?;
        self.status = "Next track".to_string();
        Ok(())
    }

    pub fn previous_track(&mut self, player: &mut Player) -> anyhow::Result<()> {
        if self.playlist.is_empty() {
            return Ok(());
        }
        self.playlist.previous();
        self.selection = self.playlist.current_index();
        self.play_current(player)?;
        self.status = "Previous track".to_string();
        Ok(())
    }

    pub fn on_tick(&mut self, player: &mut Player) -> anyhow::Result<()> {
        if !player.is_finished() {
            return Ok(());
        }

        match self.repeat {
            RepeatMode::One => {
                if let Some(track) = player.current_track().cloned() {
                    player.play_track(track)?;
                }
            }
            RepeatMode::All => {
                self.next_track(player)?;
            }
            RepeatMode::Off => {
                player.stop();
                self.status = "Finished".to_string();
            }
        }

        Ok(())
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
    }

    pub fn cycle_repeat(&mut self) {
        self.repeat = self.repeat.next();
    }

    pub fn seek_forward(&mut self, player: &mut Player, seconds: u64) -> anyhow::Result<()> {
        player.seek_relative(Duration::from_secs(seconds), true)?;
        Ok(())
    }

    pub fn seek_backward(&mut self, player: &mut Player, seconds: u64) -> anyhow::Result<()> {
        player.seek_relative(Duration::from_secs(seconds), false)?;
        Ok(())
    }

    pub fn toggle_pause(&mut self, player: &mut Player) {
        if player.state() != PlaybackState::Stopped {
            player.toggle_pause();
        }
    }
}

