use bevy::prelude::*;

use std::path::Path;
use crate::config_read::{keybind_list, keybind};
use serde::Deserialize;

///The user keybinds and other personal settings
#[derive(Deserialize)]
pub struct Config {
    pub sens: f32,
    pub zoom_sens: f32,

    ///up, left, down, right
    #[serde(deserialize_with = "keybind_list")]
    pub movement: [KeyCode; 4],
    #[serde(deserialize_with = "keybind")]
    pub jump: KeyCode,
}

const DEFAULT_CONFIG: &str = include_str!("./defaultConfig.yaml");

impl Config {
    pub fn load_or_create<P: AsRef<Path>>(file: &P) -> Self {
        let config = std::fs::read_to_string(file).unwrap_or_else(move |_| {
            let mut f = std::fs::File::create(file).expect("couldn't open new config for writing");
            use std::io::Write;
            f.write_all(DEFAULT_CONFIG.as_bytes()).expect("Couldn't write new config ??");
            DEFAULT_CONFIG.into()
        });

        Config::load_from_string(&config)
    }

    pub fn load_from_string(config: &str) -> Self {
        serde_yaml::from_str(&config).expect("Couldn't read config")
    }

    pub fn load_or_create_default() -> Self {
        let file = "./config.yaml";
        Config::load_or_create(&file)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::load_from_string(DEFAULT_CONFIG)
    }
}

#[test]
fn default_config_valid() {
    Config::default();
}
