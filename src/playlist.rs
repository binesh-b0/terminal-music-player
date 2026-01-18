use std::path::PathBuf;
use std::path::Path;
use walkdir::WalkDir;

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

    pub fn add_tracks<I>(&mut self, tracks: I)
    where
        I: IntoIterator<Item = PathBuf>,
    {
        self.tracks.extend(tracks);
        if self.current_index >= self.tracks.len() {
            self.current_index = 0;
        }
    }

    pub fn from_dir(dir: &Path) -> std::io::Result<Self> {
        let mut playlist = Self::new();
        if !dir.exists() {
            return Ok(playlist);
        }

        let mut tracks: Vec<PathBuf> = WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter_map(|entry| {
                let path = entry.path();
                let ext = path.extension()?.to_str()?.to_ascii_lowercase();
                let is_audio = matches!(ext.as_str(), "mp3" | "wav" | "flac" | "ogg");
                is_audio.then(|| path.to_path_buf())
            })
            .collect();

        tracks.sort();
        playlist.add_tracks(tracks);
        Ok(playlist)
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_previous_wraps() {
        let mut playlist = Playlist::new();
        playlist.add_tracks([
            PathBuf::from("a.mp3"),
            PathBuf::from("b.mp3"),
            PathBuf::from("c.mp3"),
        ]);

        assert_eq!(playlist.current_track().unwrap(), &PathBuf::from("a.mp3"));
        assert_eq!(playlist.next().unwrap(), &PathBuf::from("b.mp3"));
        assert_eq!(playlist.next().unwrap(), &PathBuf::from("c.mp3"));
        assert_eq!(playlist.next().unwrap(), &PathBuf::from("a.mp3"));
        assert_eq!(playlist.previous().unwrap(), &PathBuf::from("c.mp3"));
    }
}
