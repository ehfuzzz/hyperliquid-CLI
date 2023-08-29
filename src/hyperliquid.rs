use std::time::SystemTime;

use anyhow::Result;
use ethers::{
    abi::AbiEncode,
    contract::{Eip712, EthAbiType},
    signers::{LocalWallet, Signer, Wallet},
    types::{transaction::eip712::Eip712, Signature, H256},
    utils::keccak256,
};
use serde_json::json;

use crate::model::{OrderRequest, OrderResponse};

// <https://eips.ethereum.org/EIPS/eip-712>
// <https://eips.ethereum.org/EIPS/eip-2612>
#[derive(Eip712, EthAbiType, Clone)]
#[eip712(
    name = "Exchange",
    version = "1",
    chain_id = 421613,
    verifying_contract = "0x0000000000000000000000000000000000000000"
)]
pub struct Permit {
    pub source: String,
    pub connection_id: H256,
}

pub struct HyperLiquid {
    wallet: LocalWallet,
    client: reqwest::Client,
}

impl HyperLiquid {
    pub fn new(wallet: LocalWallet) -> Self {
        let client = reqwest::Client::new();

        Self { wallet, client }
    }

    async fn signature(&self, timestamp: u128) -> Signature {
        let connection_id = keccak256((self.wallet.address(), timestamp).encode()).into();

        let payload = Permit {
            source: "a".to_string(),

            connection_id,
        };

        self.wallet
            .sign_typed_data(&payload)
            .await
            .expect("Failed to sign payload")
    }

    pub async fn place_order(&self, orders: Vec<OrderRequest>) -> Result<(), anyhow::Error> {
        println!("Placing order for {}", self.wallet.address());

        let now = SystemTime::now();
        let nonce = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();

        println!(
            "Body: {:#?}",
            json!({
                "action": {
                    "type": "order",
                    "grouping": "na",
                    "orders": orders,
                },
                "nonce": nonce,
                "signature": self.signature(nonce).await
            })
        );

        let res = self
            .client
            .post("https://api.hyperliquid-testnet.xyz/exchange")
            .header("Content-Type", "application/json")
            .json(&json!({
                "action": {
                    "type": "order",
                    "grouping": "na",
                    "orders": orders,
                },
                "nonce": nonce,
                "signature": self.signature(nonce).await
            }))
            .send()
            .await?
            .text()
            .await?;
        // .json::<OrderResponse>()
        // .await?;

        println!("{:#?}", res);

        // Ok(res)
        Ok(())
    }

    fn cancel_order(&self, order_id: String) {
        println!(
            "Cancelling order {} for {}",
            order_id,
            self.wallet.address()
        );
        todo!("Implement cancel_order");
    }
}
