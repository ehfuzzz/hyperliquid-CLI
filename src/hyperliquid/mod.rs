mod exchange;
mod info;
mod types;

use std::sync::Arc;

use ethers::signers::LocalWallet;
pub use types::*;

pub trait HyperLiquid {
    fn new(wallet: Arc<LocalWallet>) -> Self;
}

pub use exchange::Exchange;
pub use info::Info;

impl HyperLiquid for Exchange {
    fn new(wallet: Arc<LocalWallet>) -> Self {
        let client = reqwest::Client::new();

        Self { wallet, client }
    }
}

impl HyperLiquid for Info {
    fn new(wallet: Arc<LocalWallet>) -> Self {
        let client = reqwest::Client::new();

        Self { wallet, client }
    }
}