use std::path::PathBuf;

/// A struct representing a playlist of audio tracks.
pub struct Playlist {
    // A vector of paths to the audio files in the playlist.
    tracks: Vec<PathBuf>,
    // The index of the currently playing track.
    current_index: usize,
}

impl Playlist {
    /// Creates a new, empty playlist.
    ///
    /// # Returns
    ///
    /// A new instance of `Playlist`.
    pub fn new() -> Self {
        Playlist {
            tracks: Vec::new(),    // Initialize with an empty list of tracks.
            current_index: 0,      // Start with the first track in the playlist.
        }
    }

    /// Adds a track to the playlist.
    ///
    /// # Arguments
    ///
    /// * `path` - A `PathBuf` representing the path to the audio file to be added.
    pub fn add_track(&mut self, path: PathBuf) {
        self.tracks.push(path);    // Add the track to the end of the list.
    }

    /// Moves to the next track in the playlist.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the `PathBuf` of the next track.
    /// Returns `None` if the playlist is empty.
    pub fn next(&mut self) -> Option<&PathBuf> {
        if !self.tracks.is_empty() {
            // Increment the current index, wrapping around to 0 if it exceeds the number of tracks.
            self.current_index = (self.current_index + 1) % self.tracks.len();
            Some(&self.tracks[self.current_index])    // Return the next track.
        } else {
            None    // Return `None` if the playlist is empty.
        }
    }

    /// Moves to the previous track in the playlist.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the `PathBuf` of the previous track.
    /// Returns `None` if the playlist is empty.
    pub fn previous(&mut self) -> Option<&PathBuf> {
        if !self.tracks.is_empty() {
            if self.current_index == 0 {
                // If at the first track, wrap around to the last track.
                self.current_index = self.tracks.len() - 1;
            } else {
                // Otherwise, just decrement the current index.
                self.current_index -= 1;
            }
            Some(&self.tracks[self.current_index])    // Return the previous track.
        } else {
            None    // Return `None` if the playlist is empty.
        }
    }

    /// Retrieves the currently selected track in the playlist.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the `PathBuf` of the current track.
    /// Returns `None` if the playlist is empty.
    pub fn current_track(&self) -> Option<&PathBuf> {
        self.tracks.get(self.current_index)    // Return the current track, if it exists.
    }
}
