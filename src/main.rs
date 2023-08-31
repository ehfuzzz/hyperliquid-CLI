use hyperliquid::{cli, settings::Settings};

#[tokio::main]
async fn main() {
    let settings = Settings::new().expect("Failed to load config");

    cli::cli(&settings).await;
}
