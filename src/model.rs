use serde::{Deserialize, Serialize};

use crate::helpers::float_to_int_for_hashing;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum Tif {
    Gtc,
    Alo,
    Ioc,
}

#[derive(Serialize)]
pub struct Limit {
    pub tif: Tif,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TriggerType {
    Tp,
    Sl,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Trigger {
    pub triger_px: f64,
    pub is_market: bool,
    pub tpsl: TriggerType,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Limit(Limit),
    Trigger(Trigger),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderRequest {
    pub asset: u32,
    pub is_buy: bool,
    pub limit_px: String,
    pub sz: String,
    pub reduce_only: bool,
    pub order_type: OrderType,
}

impl OrderRequest {
    pub fn get_type(&self) -> (u8, u64) {
        match &self.order_type {
            OrderType::Limit(limit) => match limit.tif {
                Tif::Gtc => (2, 0),
                Tif::Alo => (1, 0),
                Tif::Ioc => (3, 0),
            },
            OrderType::Trigger(trigger) => match (trigger.is_market, &trigger.tpsl) {
                (true, TriggerType::Tp) => (4, float_to_int_for_hashing(trigger.triger_px)),
                (false, TriggerType::Tp) => (5, float_to_int_for_hashing(trigger.triger_px)),
                (true, TriggerType::Sl) => (6, float_to_int_for_hashing(trigger.triger_px)),
                (false, TriggerType::Sl) => (7, float_to_int_for_hashing(trigger.triger_px)),
            },
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FilledOrder {
    pub total_sz: String,
    pub avg_px: String,
    pub oid: u128,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestingOrder {
    pub oid: u128,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Filled(FilledOrder),
    Resting(RestingOrder),
    Error(String),
}

#[derive(Deserialize, Debug)]
pub struct OrderResponseData {
    pub statuses: Vec<OrderStatus>,
}

#[derive(Deserialize, Debug)]
pub struct OrderResponse {
    #[serde(rename = "type")]
    pub type_name: String,
    pub data: OrderResponseData,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase", tag = "status", content = "response")]
pub enum ExchangeResponse {
    Ok(OrderResponse),
    Err(String),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub name: String,
    pub sz_decimals: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Universe {
    pub universe: Vec<Asset>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Ctx {
    pub mark_px: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum AssetCtx {
    Universe(Universe),
    Ctx(Vec<Ctx>),
}
