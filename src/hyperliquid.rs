pub struct HyperLiquid {
    account: String,
    client: reqwest::Client,
}

impl HyperLiquid {
    pub fn new(account: String) -> Self {
        let client = reqwest::Client::new();

        Self { account, client }
    }

    fn sign(&self, data: String) -> String {
        println!("Signing message {} for {}", data, self.account);
        "signature".to_string()
    }

    async fn place_order(&self, amount: f64, price: f64) {
        println!("Placing order for {}", self.account);
        self.client
            .post("https://api.hyperliquid.com/v1/orders")
            .json(&json!({
                "amount": amount,
                "price": price,
                "signature": self.sign("message".to_string()),
            }))
            .send()
            .await?;
    }

    fn cancel_order(&self, order_id: String) {
        println!("Cancelling order {} for {}", order_id, self.account);
    }

    pub fn handle_risk_value(&self, value: f64) {
        // Logic for handling risk value type
        println!("Handling risk value: {}", value);
        self.place_order(1.0, 1.0);
    }

    pub fn handle_set_margin(&self, value: f64) {
        // Logic for handling notional value type
        println!("Handling notional value: {}", value);
    }

    pub fn handle_cross_margin(&self, margin_type: &str) {
        //logic handling cross margin type
        println!("Handling cross margin type: {}", margin_type);
    }
}
