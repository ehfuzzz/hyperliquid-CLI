use std::{collections::HashMap, time::SystemTime};

use anyhow::Result;
use ethers::{
    abi::{AbiEncode, Hash},
    contract::{Eip712, EthAbiType},
    signers::{LocalWallet, Signer, Wallet},
    types::{transaction::eip712::Eip712, Address, Signature, H256},
    utils::keccak256,
};
use serde::Deserialize;
use serde_json::json;

use crate::model::{AssetCtx, ExchangeResponse, OrderRequest, OrderResponse, Universe};

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

pub struct HyperLiquid {
    wallet: LocalWallet,
    client: reqwest::Client,
}

impl HyperLiquid {
    pub async fn new(wallet: LocalWallet) -> Self {
        let client = reqwest::Client::new();

        Self { wallet, client }
    }

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

        let now = SystemTime::now();
        let timestamp = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect(
                "Time wen
            t backwards",
            )
            .as_millis();

        let connection_id = {
            let asset: u32 = 4;
            let is_buy: bool = true;
            let limit_px: u64 = 180000000000;
            let sz: u64 = 1000000;
            let reduce_only: bool = false;
            let order_type_0: u8 = 2;
            let order_type_1: u64 = 0;

            let order = vec![(
                asset,
                is_buy,
                limit_px,
                sz,
                reduce_only,
                order_type_0,
                order_type_1,
            )];

            let mut hashable_tuples = Vec::new();

            // for order in orders.iter() {
            // let order_type = match order.order_type {
            //     ethers::contract::OrderType::Limit(_) => 2,
            //     ethers::contract::OrderType::Trigger(_) => 3,
            // };

            // let order_type_0 = order_type as u8;
            // let order_type_1 = 0;

            // let order = (
            //     order.asset,
            //     order.is_buy,
            //     order.limit_px,
            //     order.sz,
            //     order.reduce_only,
            //     order_type_0,
            //     order_type_1,
            // );

            hashable_tuples.push(order);
            // }

            let grouping: i32 = 0;
            let vault_address = Address::zero();
            keccak256((hashable_tuples, grouping, vault_address, timestamp).encode()).into()
        };

        let signature = self.signature(connection_id).await;

        let res = self
            .client
            .post("https://api.hyperliquid-testnet.xyz/exchange")
            .json(&json!({
                "action": {
                    "type": "order",
                    "grouping": "na",
                    "orders": orders,
                },
                "nonce": timestamp,
                "signature": signature,
            }))
            .send()
            .await?
            .json()
            .await?;

        Ok(res)
    }

    fn cancel_order(&self, order_id: String) {
        println!(
            "Cancelling order {} for {}",
            order_id,
            self.wallet.address()
        );
        todo!("Implement cancel_order");
    }

    pub async fn metadata(&self) -> Result<Universe, anyhow::Error> {
        Ok(self.info("meta").await?)
    }

    pub async fn asset_ctx(&self, asset: &str) -> Result<u32, anyhow::Error> {
        let res = self.info::<Vec<AssetCtx>>("metaAndAssetCtxs").await?;

        // filter out the asset we want
        // let asset_ctx = res
        //     .into_iter()
        //     .filter(|asset_ctx| asset_ctx.name == asset)
        //     .map(|asset_ctx| asset_ctx.ctx)
        //     .collect::<Vec<u32>>()
        //     .pop()
        //     .expect("Failed to find asset");
        Ok(0)
    }

    async fn info<R: for<'de> Deserialize<'de>>(&self, value: &str) -> Result<R, anyhow::Error> {
        let res = self
            .client
            .post("https://api.hyperliquid.xyz/info")
            .json(&json!({
                    "type": value,
            }))
            .send()
            .await?
            .text()
            .await?;
        // .json()
        // .await?;

        println!("{:#?}", res);
        todo!("Implement info");

        // Ok(res)
    }
}
