//using version 2.33 not the latest one
use clap::{App, Arg};

use crate::helpers::{
    validate_limit_price, validate_sl_price, validate_tp_price, validate_value, validate_value_size,
};
use crate::hyperliquid::{Exchange, Info};
use crate::settings::Settings;

pub fn command() -> App<'static, 'static> {
    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("A CLI bot to interact with the hyperliquid exchange")
        .subcommand(
            App::new("tp")
                .about("Takes profit on open order as a market order")
                .arg(
                    Arg::with_name("size")
                        .required(true)
                        .index(1)
                        .takes_value(true)
                        .help("% of order to tp")
                        .validator(validate_value_size)
                )
                .arg(
                    Arg::with_name("asset")
                        .required(true)
                        .index(2)
                        .takes_value(true)
                        .help(
                            "Asset symbol e.g ETH, SOL, BTC ... optional if default asset is provided"
                        )
                )
                .arg(
                    Arg::with_name("tp")
                        .required(true)
                        .index(3)
                        .takes_value(true)
                        .help(
                            "Take profit price or %/$ gain in asset before tp or %$ gain in pnl before tp"
                        )
                        .validator(validate_tp_price)
                )
        )
        .subcommand(
            App::new("sl")
                .about("Stop loss on an open order as market order")
                .arg(
                    Arg::with_name("size")
                        .required(true)
                        .index(1)
                        .takes_value(true)
                        .help("% of order to sl")
                        .validator(validate_value_size)
                )
                .arg(
                    Arg::with_name("asset")
                        .required(true)
                        .index(2)
                        .takes_value(true)
                        .help(
                            "asset symbol e.g ETH, SOL, BTC .., optional if default asset is provided"
                        )
                )
                .arg(
                    Arg::with_name("sl")
                        .required(true)
                        .index(3)
                        .takes_value(true)
                        .help(
                            "Stop loss price or %/$ loss in asset before sl or %$ loss in pnl before sl"
                        )
                        .validator(validate_sl_price)
                )
        )
        .subcommand(
            App::new("buy")
                .about("Handles the Buy command")
                .help(
                    " The way we call it is: buy --size '$100' --asset eth --price @1900 --sl 1920 --tp 1865"
                )
                .arg(
                    Arg::with_name("size")
                        .help("size of the order e.g ., $100 ")
                        .long("size")
                        .takes_value(true)
                        .validator(validate_value_size)
                )
                .arg(
                    Arg::with_name("asset")
                        .help(
                            "Asset symbol e.g ETH, SOL, BTC, optional if default asset is defined"
                        )
                        .long("asset")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("price")
                        .help("Limit price e.g ., @1900")
                        .long("price")
                        .takes_value(true)
                        .validator(validate_limit_price)
                )
                .arg(
                    Arg::with_name("tp")
                        .help("Take profit value")
                        .long("tp")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("sl").help("Stop loss value").long("sl").takes_value(true)
                )
        )
        .subcommand(
            App::new("sell")
                .about(" Handles the Sell command")
                .help(
                    " The way we call it is: sell --size '$100' --asset eth --price @1900 --sl 1920 --tp 1865"
                )
                .arg(
                    Arg::with_name("size")
                        .help("size of the order e.g ., $100 ")
                        .long("size")
                        .takes_value(true)
                        .validator(validate_value_size)
                )
                .arg(
                    Arg::with_name("asset")
                        .help(
                            "Asset symbol e.g ETH, SOL, BTC, optional if default asset is defined"
                        )
                        .long("asset")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("price")
                        .help("Limit price e.g ,. @1900")
                        .long("price")
                        .takes_value(true)
                        .validator(validate_limit_price)
                )
                .arg(
                    Arg::with_name("tp")
                        .help("Take profit value")
                        .long("tp")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("sl")
                        .help("Stop loss value")
                        .long("sl")
                        .takes_value(true)
                )
        )
        .subcommand(
            App::new("twap")
                .about("Divides the total order size by the number of intervals. After the time between intervals, each piece of the divided order will be bought at market")
                .subcommand(
                    App::new("buy")
                        .about("twap buy")
                        .arg(
                            Arg::with_name("size")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("Total order size")
                                .validator(validate_value_size)
                        )
                        .arg(
                            Arg::with_name("asset")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .help("asset to be traded")
                        )
                        .arg(
                            Arg::with_name("interval")
                                .required(true)
                                .index(3)
                                .takes_value(true)
                                .help(
                                    "Time between intervals in minutes, number of intervals e.g 5,10"
                                )
                        )
                )
                .subcommand(
                    App::new("sell")
                        .about("twap sell")
                        .arg(
                            Arg::with_name("size")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("Total order size")
                                .validator(validate_value_size)
                        )
                        .arg(
                            Arg::with_name("asset")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .help("asset to be traded")
                        )
                        .arg(
                            Arg::with_name("interval")
                                .required(true)
                                .index(3)
                                .takes_value(true)
                                .help(
                                    "Time between intervals in minutes, number of intervals e.g 5,10"
                                )
                        )
                )
        )
        .subcommand(
            App::new("view")
                .about("Handles the view commands")
                .subcommand(
                    App::new("pnl").about("view pnl").help("Use to display the account's PNL")
                )
                .subcommand(
                    App::new("wallet")
                        .about("view wallet balance")
                        .arg(
                            Arg::with_name("balance")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("argument to complete the view wallet balance command")
                                .possible_values(&["balance"])
                        )
                )
                .subcommand(
                    App::new("unfilled")
                        .about("view unfilled orders")
                        .arg(
                            Arg::with_name("orders")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("argument to complete the view unfilled orders command")
                                .possible_values(&["orders"])
                        )
                )
                .subcommand(
                    App::new("open")
                        .about("view open positions")
                        .arg(
                            Arg::with_name("positions")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("argument to complete the view open positions command")
                                .possible_values(&["positions"])
                        )
                )
        )
        .subcommand(
            App::new("pair")
                .about("Takes 50% of order size and longs Asset X and takes another 50% of order size and shorts Asset Y.")
                .subcommand(
                    App::new("buy")
                        .about("pair to buy")
                        .arg(
                            Arg::with_name("size")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("Order Size")
                                .validator(validate_value_size)
                        )
                        .arg(
                            Arg::with_name("pair")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .help("Asset X/Asset Y e.g BTC/SOL")
                        )
                        .arg(
                            Arg::with_name("price")
                                .required(false)
                                .long("price")
                                .takes_value(true)
                                .help("Limit price if applicable")
                                .validator(validate_limit_price)
                        )
                        .arg(
                            Arg::with_name("sl")
                                .required(false)
                                .long("sl")
                                .takes_value(true)
                                .help("Stop loss")
                                .validator(validate_value)
                        )
                        .arg(
                            Arg::with_name("tp")
                                .required(false)
                                .long("tp")
                                .takes_value(true)
                                .help("Take profit")
                                .validator(validate_value)
                        )
                )
                .subcommand(
                    App::new("sell")
                        .about("pair to sell")
                        .arg(
                            Arg::with_name("size")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("Order Size")
                                .validator(validate_value)
                        )
                        .arg(
                            Arg::with_name("pair")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .help("Asset X/Asset Y e.g BTC/SOL")
                        )
                        .arg(
                            Arg::with_name("price")
                                .long("price")
                                .required(false)
                                .takes_value(true)
                                .help("Limit price if applicable")
                                .validator(validate_limit_price)
                        )
                        .arg(
                            Arg::with_name("sl")
                                .required(false)
                                .long("sl")
                                .takes_value(true)
                                .help("stop loss")
                                .validator(validate_value)
                        )
                        .arg(
                            Arg::with_name("tp")
                                .required(false)
                                .long("tp")
                                .takes_value(true)
                                .help("Take profit")
                                .validator(validate_value)
                        )
                )
        )
        .subcommand(
            App::new("scale")
                .about("Divides the total order size by the number of intervals. After the time between intervals, each piece of the divided order will be bought at market")
                .subcommand(
                    App::new("buy")
                        .about("scale buy")
                        .arg(
                            Arg::with_name("size_per_interval")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help(
                                    "total order size/number of intervals"
                                )
                        )
                        .arg(
                            Arg::with_name("asset")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .help("asset e.g ETH, SOL, BTC")
                        )
                        .arg(
                            Arg::with_name("lower")
                                .required(true)
                                .index(3)
                                .takes_value(true)
                                .help("lower price bracket")
                                .validator(validate_value)
                        )
                        .arg(
                            Arg::with_name("upper")
                                .required(true)
                                .index(4)
                                .takes_value(true)
                                .help("upper price bracket")
                                .validator(validate_value)
                        )
                )
                .subcommand(
                    App::new("sell")
                        .about("Divides the total order size by the number of intervals. After the time between intervals, each piece of the divided order will be bought at market")
                        .arg(
                            Arg::with_name("size_per_interval")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help(
                                    "total order size/number of intervals"
                                )
                        )
                        .arg(
                            Arg::with_name("asset")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .help("asset e.g ETH, SOL, BTC")
                        )
                        .arg(
                            Arg::with_name("lower")
                                .required(true)
                                .index(3)
                                .takes_value(true)
                                .help("Lower price bracket")
                                .validator(validate_value)
                        )
                        .arg(
                            Arg::with_name("upper")
                                .required(true)
                                .index(4)
                                .takes_value(true)
                                .help("Upper price bracket")
                                .validator(validate_value)
                        )
                )
        )
}

pub async fn order_checks(info: &Info, _exchange: &Exchange, config: &Settings, asset: &str) {
    let state = info
        .clearing_house_state()
        .await
        .expect("Failed to fetch clearing house state");

    let asset_position = state
        .asset_positions
        .iter()
        .find(|ap| ap.position.coin.to_uppercase() == asset.to_uppercase());

    let _update_leverage = match asset_position {
        Some(ap) => {
            let leverage = &ap.position.leverage;

            leverage.value != config.default_leverage.value() as u32
        }
        None => {
            println!("No open position for {}", asset);
            true
        }
    };

    let _update_margin = match asset_position {
        Some(ap) => {
            let margin_type = &ap.position.leverage.type_;

            margin_type.to_lowercase() != config.default_margin.value.to_string().to_lowercase()
        }
        None => {
            println!("No open position for {}", asset);
            true
        }
    };

    todo!("Update leverage and margin");

    // if update_leverage {
    //     println!(
    //         "Adjusting leverage for {} from {} to {}",
    //         asset,
    //         asset_position.unwrap().position.leverage.value,
    //         config.default_leverage.value()
    //     );
    //     let res = exchange
    //         .update_leverage(asset, is_cross, leverage)
    //         .await
    //         .expect("Failed to update leverage");
    // }

    // if update_margin {
    //     println!(
    //         "Adjusting margin type for {} from {} to {}",
    //         asset,
    //         asset_position.unwrap().position.leverage.type_,
    //         config.default_margin.value
    //     );
    //     todo!("Update margin type");
    // let res = exchange
    //     .update_margin(asset, is_cross, margin)
    //     .await
    //     .expect("Failed to update margin");
    // }
}
