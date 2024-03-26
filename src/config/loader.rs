use std::fs;

use super::models::Config;

pub fn load_toml(path: &str) -> Config {
    let contents = fs::read_to_string(path).unwrap();
    log::info!("Loaded configuration file: {}.", path);
    toml::from_str(&contents).unwrap()
}

