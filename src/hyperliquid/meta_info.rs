use lazy_static::lazy_static;
use reqwest::Client; // Renaming the imported Response type
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::runtime;

#[derive(Deserialize, Debug)]
struct UniverseResponse {
    universe: Vec<Universe>,
}

#[derive(Deserialize, Debug)]
struct Universe {
    pub name: String,
    pub sz_decimals: u32,
}

#[derive(Serialize, Debug)]
struct RequestBody {
    #[serde(rename = "type")]
    request_type: String,
}

pub async fn initialize_universe_data() -> Result<HashMap<String, u32>, Box<dyn std::error::Error>>
{
    let client = Client::new();
    let request_body = RequestBody {
        request_type: "meta".to_string(),
    };

    let json_body =
        serde_json::to_string(&request_body).expect("Failed to serialize the request body");
    let resp = client
        .post("https://api.hyperliquid.com/v1/universe")
        .body(json_body)
        .header("Content-Type", "application/json")
        .send()
        .await?
        .json::<UniverseResponse>()
        .await?;

    let mut map = HashMap::new();
    for asset in resp.universe {
        map.insert(asset.name, asset.sz_decimals);
    }

    Ok(map)
}

lazy_static! {
    pub static ref UNIVERSE: HashMap<String, u32> = runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(initialize_universe_data())
        .unwrap();
}

pub fn calculate_asset_to_id(asset: &str) -> u32 {
    let sz_decimals = UNIVERSE.get(asset).unwrap();
    *sz_decimals
}
