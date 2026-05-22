use std::{fs, path::PathBuf};

use serde::Deserialize;
use tracing::{error, info, warn};

#[derive(Deserialize, Debug, Clone)]
pub struct KeyboardConfig {
    pub layout: String,
    pub variant: Option<String>,
    pub options: Option<String>,
    pub model: Option<String>,
    pub numlock: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub keyboard: Option<KeyboardConfig>,
}

impl Config {
    pub fn get_file() -> PathBuf {
        PathBuf::from(std::env::var("HOME").expect("HOME variable not found !"))
            .join(".config")
            .join("river")
            .join("narisha.toml")
    }

    pub fn parse() -> Self {
        let path = Self::get_file();
        if let Ok(parsed) = fs::read_to_string(&path) {
            match toml::from_str::<Config>(&parsed) {
                Ok(config) => {
                    info!("Narisha has successfully loaded your configuration file !");
                    return config;
                }
                Err(e) => {
                    error!(
                        "Parsing the configuration file has failed with error {}, Narisha falling back to default settings...",
                        e
                    );
                }
            }
        } else {
            warn!(
                "Couldn't find your configuration file at {:?}, Narisha falling back to default settings...",
                path
            )
        }
        Config { keyboard: None }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::parse()
    }
}
