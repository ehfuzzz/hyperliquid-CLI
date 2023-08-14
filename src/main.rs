mod settings;

use clap::{Arg, ArgAction, Command};
use secrecy::ExposeSecret;
use settings::Settings;

#[tokio::main]
async fn main() {
    let command = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("A CLI bot to interact with the hyperliquid exchange")
        .subcommand(
            Command::new("ds").about("Set default size").arg(
                Arg::new("type")
                    .short('t')
                    .long("type")
                    .help("The type of default size to set")
                    .required(true)
                    .action(ArgAction::Set),
                //     Arg::new("amount")
                //         .short('a')
                //         .long("amount")
                //         .help("The amount of default size to set")
                //         .required(true)
                //         .action(ArgAction::Set),
            ),
        );

    let settings = Settings::new().expect("Failed to load configuration");

    println!("{}", settings.account.private_key.expose_secret());

    let matches = command.get_matches();

    match matches.subcommand() {
        Some(("ds", _)) => {
            println!("Setting default size");
        }
        Some(("dl", matches)) => {
            let leverage = matches
                .get_one::<String>("leverage")
                .expect("Invalid leverage")
                .parse::<f64>()
                .expect("Invalid leverage");

            println!("Leverage: {}", leverage);

            // TODO: Validate leverage
            // TODO: set leverage on the exchange
        }

        _ => {}
    }
}

// set ds risk $100
// set ds --type risk --amount $100
// set ds -t risk -a $100
