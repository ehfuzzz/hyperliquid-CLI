use hl::{settings::Settings, startup::startup};

#[tokio::main]
async fn main() {
    let settings = Settings::new().expect("Failed to load config");

    startup(&settings).await;
}
