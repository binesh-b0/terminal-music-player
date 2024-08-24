use serde::Deserialize;
use std::fs;
use std::error::Error;

#[derive(Deserialize)]
pub struct Config {
    pub default_volume: f32,
    pub playlist_directory: String,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let config_str = fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
}
