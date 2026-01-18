use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub default_volume: f32,
    pub playlist_directory: String,
}

impl Config {
    pub fn load() -> Self {
        let defaults = Self::default();
        let config_str = match std::fs::read_to_string("config.toml") {
            Ok(contents) => contents,
            Err(_) => return defaults,
        };

        toml::from_str(&config_str).unwrap_or(defaults)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_volume: 0.5,
            playlist_directory: "music/".to_string(),
        }
    }
}
