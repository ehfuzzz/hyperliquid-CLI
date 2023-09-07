use config::{Config, ConfigError, File};
use secrecy::Secret;
use serde::Deserialize;

use crate::types::MarginType;

#[derive(Deserialize)]
pub struct AccountConfig {
    pub private_key: Secret<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SizeType {
    Risk,
    Notional,
}

#[derive(Deserialize)]
pub struct DefaultSizeConfig {
    #[serde(rename = "type")]
    pub type_name: SizeType,
    pub value: String,
}

#[derive(Deserialize)]
pub struct DefaultMarginConfig {
    pub value: MarginType,
}

#[derive(Deserialize)]
pub struct DefaultAssetConfig {
    pub value: String,
}

#[derive(Deserialize)]
pub struct NetworkConfig {
    pub api: String,
}

#[derive(Deserialize)]
pub struct Settings {
    pub account: AccountConfig,
    pub default_size: DefaultSizeConfig,
    pub default_margin: DefaultMarginConfig,
    pub default_asset: DefaultAssetConfig,
    pub network: NetworkConfig,
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
