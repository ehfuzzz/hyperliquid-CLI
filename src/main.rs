use ethers::signers::LocalWallet;
use hyperliquid::{cli, hyperliquid::HyperLiquid, settings::Settings};
use secrecy::ExposeSecret;

#[tokio::main]
async fn main() {
    let settings = Settings::new().expect("Failed to load config");

    let wallet = settings
        .account
        .private_key
        .expose_secret()
        .parse::<LocalWallet>()
        .expect("Failed to parse private key");

    let hyperliquid = HyperLiquid::new(wallet);
    cli::cli(&settings, &hyperliquid).await;
}
