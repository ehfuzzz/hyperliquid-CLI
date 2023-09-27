use hl::{startup::startup, types::Config};

#[tokio::main]
async fn main() {
    let mut config = match Config::new() {
        Ok(config) => config,
        Err(err) => {
            println!("---\nFailed to load config: {}\n---", err);
            return;
        }
    };

    startup(&mut config).await;
}
