mod cli;
mod handlers;
mod helpers;
mod settings;
pub mod hyperliquid {
    pub mod meta_info;
    mod open_positions;
    mod order;
    mod open_orders;
    mod order_responses;
    mod order_payload;
}

use settings::Settings;

#[tokio::main]
async fn main() {
    let _settings = Settings::new().expect("Failed to load config");
    cli::cli();
}
