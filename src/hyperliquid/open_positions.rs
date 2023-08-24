use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
struct PositionsResponse {
    asset_positions: Vec<AssetPositions>,
}

#[derive(Deserialize, Debug)]

struct Position {
    pub coin: String,
    pub entry_px: Option<f64>,
    pub leverage: Leverage,
    pub liquidation_px: String,
    pub margin_used: String,
    pub max_trade_szs: Vec<String>,
    pub return_on_equity: String,
    pub szi: String,
    pub unrealized_pnl: String,
}

#[derive(Deserialize, Debug)]
struct Leverage {
    pub type_: String,
    pub value: u32,
}

#[derive(Deserialize, Debug)]
struct AssetPositions {
    pub position: Position,
    pub type_: String,
}

#[derive(Serialize, Debug)]
struct RequestBody {
    #[serde(rename = "type")]
    request_type: String,
    user: String,
}

async fn get_user_state() -> Result<Vec<AssetPositions>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let request_body = RequestBody {
        request_type: "clearinghouseState".to_string(),
        user: String::from("users onchain address"),
    };
    let json_body = serde_json::to_string(&request_body).expect("Failed to serialize the request body");
    let resp = client
        .post("https://api.hyperliquid.xyz/info")
        .body(json_body)
        .header("Content-Type", "application/json")
        .send()
        .await?
        .json::<PositionsResponse>()
        .await?;
    Ok(resp.asset_positions)
}
