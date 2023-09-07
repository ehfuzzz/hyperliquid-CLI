use std::sync::Arc;

use anyhow::Result;
use ethers::signers::{LocalWallet, Signer};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{
    types::{AssetCtx, ClearingHouseState, Ctx, UnfilledOrder, Universe},
    UserFill,
};

pub struct Info {
    pub wallet: Arc<LocalWallet>,
    pub client: reqwest::Client,
    pub base_url: String,
}

impl Info {
    pub async fn metadata(&self) -> Result<Universe, anyhow::Error> {
        Ok(self
            .post(json!({
                    "type": "meta",
            }))
            .await?)
    }

    pub async fn asset_ctxs(&self) -> Result<Vec<AssetCtx>, anyhow::Error> {
        Ok(self
            .post(json!({
                    "type": "metaAndAssetCtxs",
            }))
            .await?)
    }

    pub async fn asset_ctx(&self, asset: &str) -> Result<Option<Ctx>, anyhow::Error> {
        let asset_ctxs = self.asset_ctxs().await?;

        let universe = match asset_ctxs.get(0) {
            Some(AssetCtx::Universe(universe)) => universe,
            _ => return Ok(None),
        };

        let position = universe
            .universe
            .iter()
            .position(|a| a.name.to_uppercase() == asset.to_uppercase())
            .unwrap();

        let ctxs = match asset_ctxs.get(1) {
            Some(AssetCtx::Ctx(ctxs)) => ctxs,
            _ => return Ok(None),
        };

        Ok(Some(ctxs[position].clone()))
    }

    pub async fn clearing_house_state(&self) -> Result<ClearingHouseState, anyhow::Error> {
        let res = self
            .post(json!({
                    "type": "clearinghouseState",
                    "user": self.wallet.address(),
            }))
            .await?;

        Ok(res)
    }

    pub async fn open_orders(&self) -> Result<Vec<UnfilledOrder>, anyhow::Error> {
        let res = self
            .post(json!({
                    "type": "openOrders",
                    "user": self.wallet.address(),
            }))
            .await?;

        Ok(res)
    }

    pub async fn user_fills(&self) -> Result<Vec<UserFill>, anyhow::Error> {
        let res = self
            .post(json!({
                    "type": "userFills",
                    "user": self.wallet.address(),
            }))
            .await?;

        Ok(res)
    }

    async fn post<T: for<'de> Deserialize<'de>>(
        &self,
        body: impl Serialize,
    ) -> Result<T, anyhow::Error> {
        let res = self
            .client
            .post(format!("{}/info", self.base_url))
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
        Ok(res)
    }
}
