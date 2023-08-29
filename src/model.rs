use serde::{Deserialize, Serialize};

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
