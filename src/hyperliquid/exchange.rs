use std::{fmt::Debug, sync::Arc, time::SystemTime};

use ethers::{
    abi::AbiEncode,
    signers::{LocalWallet, Signer},
    types::{Address, Signature, H256},
    utils::keccak256,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{float_to_int_for_hashing, l1, ExchangeResponse, OrderRequest};

pub struct Exchange {
    pub wallet: Arc<LocalWallet>,
    pub client: reqwest::Client,
    pub base_url: String,
}

impl Exchange {
    async fn signature(&self, connection_id: H256) -> Signature {
        let payload = l1::Agent {
            source: "b".to_string(),
            connection_id,
        };

        self.wallet
            .sign_typed_data(&payload)
            .await
            .expect("Failed to sign payload")
    }

    pub async fn place_order(
        &self,
        order: OrderRequest,
    ) -> Result<ExchangeResponse, anyhow::Error> {
        let nonce = self.timestamp();
        let orders = vec![order];

        let connection_id = {
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
        };

        let res = self
            .post(json!({
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

    pub async fn update_leverage(
        &self,
        leverage: u32,
        asset: u32,
        is_cross: bool,
    ) -> Result<ExchangeResponse, anyhow::Error> {
        let nonce = self.timestamp();

        let vault_address = Address::zero();
        let connection_id =
            keccak256((asset, is_cross, leverage, vault_address, nonce).encode()).into();

        let res = self
            .post(json!({
                "action": {
                    "type": "updateLeverage",
                    "asset": asset,
                    "isCross": is_cross,
                    "leverage": leverage,
                },
                "nonce": nonce,
                "signature": self.signature(connection_id).await,
            }))
            .await?;

        Ok(res)
    }

    async fn post<T: for<'de> Deserialize<'de>>(
        &self,
        body: impl Serialize + Debug,
    ) -> Result<T, anyhow::Error> {
        let res = self
            .client
            .post(format!("{}/exchange", self.base_url))
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        Ok(res)
    }

    fn timestamp(&self) -> u128 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }
}
