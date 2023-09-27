use home::home_dir;

use crate::types::Config;

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

        // validate config
        if config.private_key.is_empty() {
            return Err("private_key is required".into());
        }

        if config.default_asset.is_empty() {
            return Err("default_asset is required".into());
        }

        if config.default_size.is_empty() {
            return Err("default_size is required".into());
        }

        Ok(config)
    }
}
