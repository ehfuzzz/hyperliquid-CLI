use std::{sync::Arc, time::SystemTime};

use ethers::{
    abi::AbiEncode,
    contract::{Eip712, EthAbiType},
    signers::{LocalWallet, Signer},
    types::{Address, Signature, H256},
    utils::keccak256,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::helpers::float_to_int_for_hashing;

use super::{ExchangeResponse, OrderRequest};

// <https://eips.ethereum.org/EIPS/eip-712>
// <https://eips.ethereum.org/EIPS/eip-2612>
#[derive(Eip712, EthAbiType, Clone, Debug)]
#[eip712(
    name = "Exchange",
    version = "1",
    chain_id = 1337,
    verifying_contract = "0x0000000000000000000000000000000000000000"
)]
pub struct Agent {
    pub source: String,
    pub connection_id: H256,
}

pub struct Exchange {
    pub wallet: Arc<LocalWallet>,
    pub client: reqwest::Client,
}

impl Exchange {
    async fn signature(&self, connection_id: H256) -> Signature {
        let payload = Agent {
            source: "a".to_string(),
            connection_id,
        };

        self.wallet
            .sign_typed_data(&payload)
            .await
            .expect("Failed to sign payload")
    }

    pub async fn place_order(
        &self,
        orders: Vec<OrderRequest>,
    ) -> Result<ExchangeResponse, anyhow::Error> {
        println!("Placing order for {}", self.wallet.address());

        let nonce = self.timestamp();

        let connection_id = self.connection_id(&orders, nonce);

        println!(
            "{:#?}",
            json!({
                "action": {
                    "type": "order",
                    "grouping": "na",
                    "orders": orders,
                },
                "nonce": nonce,
                "signature": self.signature(connection_id).await,
            })
        );

        let res = self
            .exchange(json!({
                "action": {
                    "type": "order",
                    "grouping": "na",
                    "orders": orders,
                },
                "nonce": nonce,
                "signature": self.signature(connection_id).await,
            }))
            .await?;

        Ok(res)
    }

    async fn exchange<T: for<'de> Deserialize<'de>>(
        &self,
        body: impl Serialize,
    ) -> Result<T, anyhow::Error> {
        let res = self
            .client
            .post("https://api.hyperliquid-testnet.xyz/exchange")
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
        Ok(res)
    }
    fn connection_id(&self, orders: &Vec<OrderRequest>, nonce: u128) -> H256 {
        let hashable_tuples = orders
            .iter()
            .map(|order| {
                let order_type = order.get_type();

                (
                    order.asset,
                    order.is_buy,
                    float_to_int_for_hashing(
                        order.limit_px.parse().expect("Failed to parse limit_px"),
                    ),
                    float_to_int_for_hashing(order.sz.parse().expect("Failed to parse sz")),
                    order.reduce_only,
                    order_type.0,
                    order_type.1,
                )
            })
            .collect::<Vec<_>>();

        let grouping: i32 = 0;
        let vault_address = Address::zero();
        keccak256((hashable_tuples, grouping, vault_address, nonce).encode()).into()
    }

    fn timestamp(&self) -> u128 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }
}
