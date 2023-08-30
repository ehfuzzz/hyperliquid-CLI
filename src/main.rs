use std::collections::HashMap;

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

    let hyperliquid = HyperLiquid::new(wallet).await;

    // let metadata = hyperliquid
    //     .metadata()
    //     .await
    //     .expect("Failed to fetch metadata");

    // let assets = metadata
    //     .universe
    //     .into_iter()
    //     .map(|asset| (asset.name.to_uppercase(), asset.sz_decimals))
    //     .collect::<HashMap<String, u32>>();

    // println!("{:#?}", assets);

    let asset_ctx = hyperliquid
        .asset_ctx("BTC")
        .await
        .expect("Failed to fetch asset ctxs");

    println!("{:#?}", asset_ctx);

    // cli::cli(&settings, &hyperliquid).await;
}
