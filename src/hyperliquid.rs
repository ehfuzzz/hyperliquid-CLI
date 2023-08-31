use std::time::SystemTime;

use anyhow::Result;
use config::Value;
use ethers::{
    abi::AbiEncode,
    contract::{Eip712, EthAbiType},
    signers::{LocalWallet, Signer},
    types::{Address, Signature, H256},
    utils::keccak256,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    helpers::float_to_int_for_hashing,
    model::{
        AssetCtx, ClearingHouseState, Ctx, ExchangeResponse, OrderRequest, UnfilledOrder, Universe,
    },
};

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

        let nonce = self.timestamp();

        let connection_id = self.connection_id(&orders, nonce);

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

    fn cancel_order(&self, order_id: String) {
        println!(
            "Cancelling order {} for {}",
            order_id,
            self.wallet.address()
        );
        todo!("Implement cancel_order");
    }

    pub async fn metadata(&self) -> Result<Universe, anyhow::Error> {
        Ok(self
            .info(json!({
                    "type": "meta",
            }))
            .await?)
    }

    pub async fn asset_ctx(&self, asset: &str) -> Result<Option<Ctx>, anyhow::Error> {
        let res = &self
            .info::<Vec<AssetCtx>>(json!({
                    "type": "metaAndAssetCtxs",
            }))
            .await?;

        let universe = match res.get(0) {
            Some(AssetCtx::Universe(universe)) => universe,
            _ => return Ok(None),
        };

        let position = universe
            .universe
            .iter()
            .position(|a| a.name.to_uppercase() == asset.to_uppercase())
            .unwrap();

        let ctxs = match res.get(1) {
            Some(AssetCtx::Ctx(ctxs)) => ctxs,
            _ => return Ok(None),
        };

        println!("Position: {}", position);

        Ok(Some(ctxs[position].clone()))
    }

    pub async fn clearing_house_state(&self) -> Result<ClearingHouseState, anyhow::Error> {
        let res = self
            .info(json!({
                    "type": "clearinghouseState",
                    "user": self.wallet.address(),
            }))
            .await?;

        Ok(res)
    }

    pub async fn open_orders(&self) -> Result<Vec<UnfilledOrder>, anyhow::Error> {
        let res = self
            .info(json!({
                    "type": "openOrders",
                    "user": self.wallet.address(),
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

    async fn info<T: for<'de> Deserialize<'de>>(
        &self,
        body: impl Serialize,
    ) -> Result<T, anyhow::Error> {
        let res = self
            .client
            .post("https://api.hyperliquid-testnet.xyz/info")
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
