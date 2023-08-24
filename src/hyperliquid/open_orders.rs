use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::runtime;
use lazy_static::lazy_static;

#[derive(Deserialize, Debug)]
pub struct UnfilledResponse {
   pub coin: String,
   pub limitPx: String,
   pub oid: u64,
   pub side: String,
   pub sz: String,
   pub timestamp: u64,
}

#[derive(Serialize, Debug)]
struct RequestBody {
    #[serde(rename = "type")]
    request_type: String,
    user: String,
}

pub async fn get_user_open_orders() -> Result<HashMap<String, UnfilledResponse>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let request_body = RequestBody {
        request_type: "openOrders".to_string(),
        user: String::from("users onchain address"),
    };
    let json_body = serde_json::to_string(&request_body).expect("Failed to serialize the request body");
    let resp = client
        .post("https://api.hyperliquid.xyz/info")
        .body(json_body)
        .header("Content-Type", "application/json")
        .send()
        .await?
        .json::<Vec<UnfilledResponse>>()
        .await?;

    // Create a HashMap to store the response information
    let mut response_map: HashMap<String, UnfilledResponse> = HashMap::new();
    for unfilled_response in resp {
        // Use the `oid` field as the key for the HashMap
        response_map.insert(unfilled_response.oid.to_string(), unfilled_response);
    }

    Ok(response_map)
}

lazy_static! {
    pub static ref OPEN_ORDERS: HashMap<String, UnfilledResponse> = runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(get_user_open_orders())
        .unwrap();
}

pub fn get_side_from_oid(oid: &str) -> bool {
    let order = OPEN_ORDERS.get(oid).unwrap();
    if order.side == "B" {
        true
    } else {
        false
    }
}

pub fn get_asset_from_oid(oid: &str) -> String {
    let order = OPEN_ORDERS.get(oid).unwrap();
    order.coin.clone()
}

pub fn get_sz_from_oid(oid: &str) -> String {
    let order = OPEN_ORDERS.get(oid).unwrap();
    order.sz.clone()
}