use std::env;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use log::{error, info};

use crate::compress::CompressType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub token: String,
    pub compression_level: i32,
    pub compress_type: CompressType,
    pub mode: Mode,
    pub storage: Vec<Storage>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            token: "".to_owned(),
            compression_level: 9,
            compress_type: CompressType::LZMA,
            mode: Mode::Store,
            storage: Vec::<Storage>::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Mode {
    Store,
    Retrieve,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storage {
    pub name: String,
    pub files: Vec<String>,
}

fn get_config_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        info!("Detected Windows system");
        let app_data = env::var("APPDATA")
            .unwrap_or_else(|_| "C:\\Users\\{Username}\\AppData\\Roaming".to_string());
        PathBuf::from(app_data).join("discordstorage")
    } else if cfg!(target_os = "macos") {
        info!("Detected MacOS system");
        PathBuf::from(env::var("HOME").unwrap()).join("Library/Application Support/discordstorage")
    } else {
        info!("Detected Unix-like system");
        PathBuf::from(
            env::var("XDG_CONFIG_HOME")
                .unwrap_or_else(|_| env::var("HOME").unwrap().to_string() + "/.config"),
        )
        .join("discordstorage")
    }
}

pub fn get_config() -> Config {
    let config_path = PathBuf::from(get_config_dir()).join("config.json");

    if !config_path.exists() {
        error!("Config file not found at {:?}", config_path);
        info!("Creating default config file at {:?}", config_path);
        let default_config = Config::default();
        let config = serde_json::to_string(&default_config).unwrap();
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(config_path, config).unwrap();

        default_config
    } else {
        let config = std::fs::read_to_string(&config_path).unwrap();
        let config: Config = serde_json::from_str(&config).unwrap_or_else(|_| {
            error!("Failed to parse config file at {:?}", config_path);
            info!("Creating default config file at {:?}", config_path);
            std::fs::write(
                &config_path,
                serde_json::to_string(&Config::default()).unwrap(),
            )
            .unwrap();
            Config::default()
        });
        info!("Config file loaded from {:?}", config_path);
        config
    }
}

pub fn set_config(config: Config) {
    let config_path = PathBuf::from(get_config_dir()).join("config.json");

    if !config_path.exists() {
        error!("Config file not found at {:?}", config_path);
        info!("Creating default config file at {:?}", config_path);
        let config = serde_json::to_string(&config).unwrap();
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(config_path, config).unwrap();
        return;
    }

    let config = serde_json::to_string(&config).unwrap();
    std::fs::write(&config_path, config).unwrap();
    info!("Config file updated at {:?}", config_path);
}
