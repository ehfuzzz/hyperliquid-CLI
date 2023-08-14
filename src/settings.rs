use config::{Config, ConfigError, File};
use secrecy::Secret;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AccountConfig {
    pub private_key: Secret<String>,
}

#[derive(Deserialize)]
pub struct Settings {
    pub account: AccountConfig,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_env = std::env::var("RUN_ENV").unwrap_or_else(|_| "testnet".into());

        let config = std::env::current_dir()
            .expect("Failed to get current directory")
            .join("config");

        let cfg = Config::builder()
            .add_source(File::from(config.join("default.toml")))
            .add_source(File::from(config.join(format!("{}.toml", run_env))).required(false))
            .build()
            .unwrap();

        cfg.try_deserialize()
    }
}
