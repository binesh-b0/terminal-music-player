use serde::Deserialize;
use std::fs;
use std::error::Error;

/// A struct representing the configuration settings for the music player.
///
/// The `Config` struct is designed to be deserialized from a TOML file, providing
/// settings such as the default volume and the directory where the playlist is stored.
#[derive(Deserialize)]
pub struct Config {
    // The default volume level for the music player (between 0.0 and 1.0).
    pub default_volume: f32,
    // The directory path where the playlist files are located.
    // uncomment whlile implementing
    // pub playlist_directory: String,
}

impl Config {
    /// Loads the configuration settings from a `config.toml` file.
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded `Config` on success, or an error if the file
    /// could not be read or the contents could not be parsed.
    pub fn load() -> Result<Self, Box<dyn Error>> {
        // Read the contents of the `config.toml` file into a string.
        let config_str = fs::read_to_string("config.toml")?;
        
        // Parse the string as TOML and deserialize it into a `Config` struct.
        let config: Config = toml::from_str(&config_str)?;
        
        // Return the deserialized `Config`.
        Ok(config)
    }
}
