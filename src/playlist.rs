use std::path::PathBuf;

pub struct Playlist {
    tracks: Vec<PathBuf>,
    current_index: usize,
}

impl Playlist {
    pub fn new() -> Self {
        Playlist {
            tracks: Vec::new(),
            current_index: 0,
        }
    }

    pub fn add_track(&mut self, path: PathBuf) {
        self.tracks.push(path);
    }

    pub fn next(&mut self) -> Option<&PathBuf> {
        if !self.tracks.is_empty() {
            self.current_index = (self.current_index + 1) % self.tracks.len();
            Some(&self.tracks[self.current_index])
        } else {
            None
        }
    }

    pub fn previous(&mut self) -> Option<&PathBuf> {
        if !self.tracks.is_empty() {
            if self.current_index == 0 {
                self.current_index = self.tracks.len() - 1;
            } else {
                self.current_index -= 1;
            }
            Some(&self.tracks[self.current_index])
        } else {
            None
        }
    }

    pub fn current_track(&self) -> Option<&PathBuf> {
        self.tracks.get(self.current_index)
    }
}
