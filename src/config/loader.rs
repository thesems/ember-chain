use std::fs;

use super::models::{Config, WalletConfig};

pub fn load_toml(path: &str) -> Config {
    let contents = fs::read_to_string(path).unwrap();
    log::info!("Loaded configuration file: {}.", path);
    toml::from_str(&contents).unwrap()
}

pub fn load_toml_wallet(path: &str) -> WalletConfig {
    let contents = fs::read_to_string(path).unwrap();
    log::info!("Loaded wallet configuration file: {}.", path);
    toml::from_str(&contents).unwrap()
}
