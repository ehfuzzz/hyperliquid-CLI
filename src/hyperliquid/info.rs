use std::sync::Arc;

use anyhow::Result;
use ethers::signers::{LocalWallet, Signer};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::types::{AssetCtx, ClearingHouseState, Ctx, UnfilledOrder, Universe};

pub struct Info {
    pub wallet: Arc<LocalWallet>,
    pub client: reqwest::Client,
}

impl Info {
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
}
