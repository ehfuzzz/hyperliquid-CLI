mod agent;
mod exchange;
mod helpers;
mod info;
mod types;
use std::sync::Arc;

pub use agent::*;
use ethers::signers::LocalWallet;
pub use helpers::float_to_int_for_hashing;
pub use types::*;

pub trait HyperLiquid {
    fn new(wallet: Arc<LocalWallet>, base_url: String) -> Self;
}

pub use exchange::Exchange;
pub use info::Info;

impl HyperLiquid for Exchange {
    fn new(wallet: Arc<LocalWallet>, base_url: String) -> Self {
        let client = reqwest::Client::new();

        Self {
            wallet,
            client,
            base_url,
        }
    }
}

impl HyperLiquid for Info {
    fn new(wallet: Arc<LocalWallet>, base_url: String) -> Self {
        let client = reqwest::Client::new();

        Self {
            wallet,
            client,
            base_url,
        }
    }
}
