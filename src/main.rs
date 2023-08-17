mod settings;
mod cli;
mod helpers;
mod handlers; 
use settings::Settings;



#[tokio::main]
async fn main() {
    let _settings = Settings::new().expect("Failed to load config");
    cli::cli();

    }
