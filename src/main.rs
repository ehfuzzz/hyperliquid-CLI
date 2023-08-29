mod cli;
mod handlers;
mod helpers;
mod settings;
use settings::Settings;
pub mod hyperliquid {
    pub mod order;
    pub mod order_payload;
    pub mod order_responses;
}

#[tokio::main]
async fn main() {
    let _settings = Settings::new().expect("Failed to load config");
    cli::cli();
}
