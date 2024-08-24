use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub default_volume: f32,
    pub playlist_directory: String,
}

impl Config {
    pub fn load() -> Self {
        let config_str = std::fs::read_to_string("config.toml").unwrap();
        toml::from_str(&config_str).unwrap()
    }
}
