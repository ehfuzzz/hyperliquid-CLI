mod cli;
mod helpers;
mod settings;
pub mod hyperliquid {
    pub mod meta_info;
    pub mod open_orders;
    pub mod open_positions;
    pub mod order;
    pub mod order_payload;
    pub mod order_responses;
}

use settings::Settings;
pub mod hyperliquid {
    pub mod order;
    pub mod order_payload;
    pub mod order_responses;
}

#[tokio::main]
async fn main() {
    let _settings = Settings::new().expect("Failed to load config");
    cli::cli().await;
}
