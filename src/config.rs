use home::home_dir;
use hyperliquid::types::Chain;

use crate::types::{Config, MarginType};

impl Config {
    pub fn new() -> Result<Self, String> {
        let home_dir = match home_dir() {
            Some(path) => path,
            None => return Err("Impossible to get your home dir!".into()),
        };

        // create .hyperliquid if it doesn't exist
        let config_path = home_dir.join(".hyperliquid");
        std::fs::create_dir_all(&config_path).expect("Failed to create config directory");

        // create .hyperliquid/config if it doesn't exist
        let config_file_path = config_path.join("config");
        if !config_file_path.exists() {
            std::fs::write(
                &config_file_path,
                serde_json::to_string_pretty(&Config::default())
                    .expect("Failed to serialize config"),
            )
            .expect("Failed to create config file");
        }

        let config =
            std::fs::read_to_string(&config_file_path).expect("Failed to read config file");

        let config: Self = serde_json::from_str(&config).expect("Failed to parse config file");

        Ok(config)
    }
}

impl Config {
    pub fn save(&self) -> Result<(), String> {
        let home_dir = match home_dir() {
            Some(path) => path,
            None => return Err("Impossible to get your home dir!".into()),
        };

        // create .hyperliquid if it doesn't exist
        let config_path = home_dir.join(".hyperliquid");
        std::fs::create_dir_all(&config_path).expect("Failed to create config directory");

        // create .hyperliquid/config if it doesn't exist
        let config_file_path = config_path.join("config");
        if !config_file_path.exists() {
            std::fs::write(
                &config_file_path,
                serde_json::to_string_pretty(&Config::default())
                    .expect("Failed to serialize config"),
            )
            .expect("Failed to create config file");
        }

        std::fs::write(
            &config_file_path,
            serde_json::to_string_pretty(&self).expect("Failed to serialize config"),
        )
        .expect("Failed to create/update config file");

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            private_key: String::new(),
            default_margin: MarginType::Isolated,
            default_asset: String::new(),
            default_size: String::new(),
            chain: Chain::ArbitrumTestnet,
        }
    }
}
