//using version 2.33 not the latest one
use clap::{App, Arg};
use std::num::ParseFloatError;

mod handlers;
mod helpers;

use handlers::{
    handle_cross_margin, handle_isolated_margin, handle_notional_value, handle_risk_value,
};
use helpers::{validate_sl_price, validate_tp_price, validate_value_size};
#[tokio::main]
async fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("A CLI bot to interact with the hyperliquid exchange")
        .subcommand(
            App::new("set")
                .about("Handles all the specified set commands")
                .subcommand(
                    App::new("ds")
                        .about("Sets the default size")
                        .arg(
                            Arg::with_name("size_type")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .possible_values(&["risk", "notional"])
                                .help("Either risk or notional")
                        )
                        .arg(
                            Arg::with_name("value_size")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .validator(validate_value_size)
                                .help("Size in USDC or size in % of balance")
                        )
                )
                .subcommand(
                    App::new("dm")
                        .about("Sets the default margin")
                        .arg(
                            Arg::with_name("margin_type")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .possible_values(&["i", "c"])
                                .help("Default margin type Either Isolated(i) or cross margin(c)")
                        )
                )
        )
        .subcommand(
            App::new("tp")
                .about(" Handles Take profit command")
                .arg(
                    Arg::with_name("percentage_order")
                        .required(true)
                        .index(1)
                        .takes_value(true)
                        .help("% of order to TP")
                        .validator(validate_value_size)
                )
                .arg(
                    Arg::with_name("asset_symbol")
                        .required(true)
                        .index(2)
                        .takes_value(true)
                        .help(
                            "asset symbol e.g ETH, SOL, BTC .., optional if default asset is provided"
                        )
                )
                .arg(
                    Arg::with_name("tp_price")
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
                .about("Handles Stop Loss command")
                .arg(
                    Arg::with_name("percentage_order")
                        .required(true)
                        .index(1)
                        .takes_value(true)
                        .help("% of order to SL")
                        .validator(validate_value_size)
                )
                .arg(
                    Arg::with_name("asset_symbol")
                        .required(true)
                        .index(2)
                        .takes_value(true)
                        .help(
                            "asset symbol e.g ETH, SOL, BTC .., optional if default asset is provided"
                        )
                )
                .arg(
                    Arg::with_name("sl_price")
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
            App::new("scale")
                .about("Handles the scale buy and sell logic")
                .subcommand(
                    App::new("buy")
                        .about("scale buy")
                        .arg(
                            Arg::with_name("total_order_size")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("Total Order Size")
                                .validator(|v| {
                                    if v.parse::<f64>().is_ok() {
                                        Ok(())
                                    } else {
                                        Err(String::from("Invalid value"))
                                    }
                                })
                        )
                        .arg(
                            Arg::with_name("interval")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .help("Number of orders to place ie Total Order Size / interval")
                                .validator(|v| {
                                    if v.parse::<f64>().is_ok() {
                                        Ok(())
                                    } else {
                                        Err(String::from("Invalid value"))
                                    }
                                })
                        )

                        .arg(
                            Arg::with_name("asset_symbol")
                                .required(true)
                                .index(3)
                                .takes_value(true)
                                .help("asset symbol e.g ETH, SOL, BTC")
                        )

                        .arg(
                            Arg::with_name("lower_price_bracket")
                                .required(true)
                                .index(4)
                                .takes_value(true)
                                .help("Price to start buying from")
                                .validator(|v| {
                                    if v.parse::<f64>().is_ok() {
                                        Ok(())
                                    } else {
                                        Err(String::from("Invalid value"))
                                    }
                                })
                        )

                        .arg(
                            Arg::with_name("upper_price_bracket")
                                .required(true)
                                .index(5)
                                .takes_value(true)
                                .help("Price to stop buying at")
                                .validator(|v| {
                                    if v.parse::<f64>().is_ok() {
                                        Ok(())
                                    } else {
                                        Err(String::from("Invalid value"))
                                    }
                                })
                        )
                )
                .subcommand(
                    App::new("sell")
                        .about("scale sell")
                        .arg(
                            Arg::with_name("total_order_size")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("Total Order Size")
                                .validator(|v| {
                                    if v.parse::<f64>().is_ok() {
                                        Ok(())
                                    } else {
                                        Err(String::from("Invalid value"))
                                    }
                                })
                        )
                        .arg(
                            Arg::with_name("interval")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .help("Number of orders to place ie Total Order Size / interval")
                                .validator(|v| {
                                    if v.parse::<f64>().is_ok() {
                                        Ok(())
                                    } else {
                                        Err(String::from("Invalid value"))
                                    }
                                })
                        )

                        .arg(
                            Arg::with_name("asset_symbol")
                                .required(true)
                                .index(3)
                                .takes_value(true)
                                .help("asset symbol e.g ETH, SOL, BTC")
                        )

                        .arg(
                            Arg::with_name("lower_price_bracket")
                                .required(true)
                                .index(4)
                                .takes_value(true)
                                .help("Price to start selling from")
                                .validator(|v| {
                                    if v.parse::<f64>().is_ok() {
                                        Ok(())
                                    } else {
                                        Err(String::from("Invalid value"))
                                    }
                                })
                        )

                        .arg(
                            Arg::with_name("upper_price_bracket")
                                .required(true)
                                .index(5)
                                .takes_value(true)
                                .help("Price to stop selling at")
                                .validator(|v| {
                                    if v.parse::<f64>().is_ok() {
                                        Ok(())
                                    } else {
                                        Err(String::from("Invalid value"))
                                    }
                                })
                        )
                )
        ).get_matches();

    match matches.subcommand() {
        ("set", Some(set_matches)) => {
            if let Some(ds_matches) = set_matches.subcommand_matches("ds") {
                let size_type = ds_matches.value_of("size_type").unwrap();
                let value_size = ds_matches.value_of("value_size").unwrap();

                let converted_value: Result<f64, ParseFloatError> = if value_size.ends_with('%') {
                    value_size
                        .trim_end_matches('%')
                        .parse::<f64>()
                        .map(|percent| percent / 100.0)
                } else {
                    value_size.trim_start_matches('$').parse::<f64>()
                };

                match size_type {
                    "risk" => match converted_value {
                        Ok(value) => {
                            handle_risk_value(value);
                        }
                        Err(_) => {
                            println!("Invalid value format");
                        }
                    },
                    "notional" => match converted_value {
                        Ok(value) => {
                            handle_notional_value(value);
                        }
                        Err(_) => {
                            println!("Invalid value format");
                        }
                    },
                    _ => unreachable!(),
                }
            } else if let Some(ds_matches) = set_matches.subcommand_matches("dm") {
                let margin_type = ds_matches.value_of("margin_type").unwrap();
                println!("Margin type: {}", margin_type);

                match margin_type {
                    "i" => handle_isolated_margin(margin_type),
                    "c" => handle_cross_margin(margin_type),
                    _ => unreachable!(), // we should not get here because of the possible value checker
                }
            }
        }

        ("tp", Some(tp_matches)) => {
            let percentage_order = tp_matches.value_of("percentage_order").unwrap();
            let asset_symbol = tp_matches.value_of("asset_symbol").unwrap();
            let tp_price = tp_matches.value_of("tp_price").unwrap();

            let converted_percentage_order: Result<f64, ParseFloatError> = {
                percentage_order
                    .trim_end_matches("%")
                    .parse::<f64>()
                    .map(|percent| percent / 100.0)
            };
            println!(
                "converted percentage order: {:?}, asset_symbol: {}",
                converted_percentage_order, asset_symbol
            );

            match tp_price {
                tp_price if tp_price.trim_start_matches("+").ends_with("%") => {
                    let numeric_part = &tp_price[1..tp_price.len() - 1];
                    let converted_value = numeric_part.parse::<f64>().unwrap() / 100.0;
                    println!("Logic for handling +10% tp price: {}", converted_value);
                }
                tp_price if tp_price.starts_with("+$") => {
                    let numeric_part = &tp_price[2..];
                    let converted_value = numeric_part.parse::<u32>().unwrap();
                    println!("Logic for handling +$300: {}", converted_value);
                }
                tp_price if tp_price.trim_start_matches("+").ends_with("%pnl") => {
                    let numeric_part = &tp_price[1..tp_price.len() - 4];
                    let converted_value = numeric_part.parse::<f64>().unwrap() / 100.0;
                    println!("Logic for handling +10%pnl: {}", converted_value);
                }
                tp_price if tp_price.trim_start_matches("+").ends_with("pnl") => {
                    let numeric_part = &tp_price[1..tp_price.len() - 3];
                    let converted_value = numeric_part.parse::<u32>().unwrap();
                    println!("Logic for handling +300pnl: {}", converted_value);
                }

                _ => {
                    println!("No matching pattern");
                }
            }
        }
        ("sl", Some(sl_matches)) => {
            let percentage_order = sl_matches.value_of("percentage_order").unwrap();
            let asset_symbol = sl_matches.value_of("asset_symbol").unwrap();
            let sl_price = sl_matches.value_of("sl_price").unwrap();

            let converted_percentage_order: Result<f64, ParseFloatError> = {
                percentage_order
                    .trim_end_matches("%")
                    .parse::<f64>()
                    .map(|percent| percent / 100.0)
            };
            println!(
                "converted percentage order: {:?}, asset_symbol: {}",
                converted_percentage_order, asset_symbol
            );

            match sl_price {
                sl_price if sl_price.trim_start_matches("-").ends_with("%") => {
                    let numeric_part = &sl_price[1..sl_price.len() - 1];
                    let converted_value = numeric_part.parse::<f64>().unwrap() / 100.0;
                    println!("Logic for handling -10% sl price: {}", converted_value);
                }

                sl_price if sl_price.starts_with("-$") => {
                    let numeric_part = &sl_price[2..];
                    let converted_value = numeric_part.parse::<u32>().unwrap();
                    println!("Logic for handling -$300: {}", converted_value);
                }

                sl_price if sl_price.trim_start_matches("-").ends_with("%pnl") => {
                    let numeric_part = &sl_price[1..sl_price.len() - 4];
                    let converted_value = numeric_part.parse::<f64>().unwrap() / 100.0;
                    println!("Logic for handling -10%pnl: {}", converted_value);
                }

                sl_price if sl_price.trim_start_matches("-").ends_with("pnl") => {
                    let numeric_part = &sl_price[1..sl_price.len() - 3];
                    let converted_value = numeric_part.parse::<u32>().unwrap();
                    println!("Logic for handling -300pnl: {}", converted_value);
                }

                _ => {
                    println!("No matching pattern");
                }
            }

            //Handle Scale Buy  <total order size/number of intervals> <asset symbol> <lower price bracket> <upper price bracket>
        }
        ("buy", Some(scale_buy_matches)) => {
            let total_order_size = scale_buy_matches.value_of("total_order_size").unwrap();
            let asset_symbol = scale_buy_matches.value_of("asset_symbol").unwrap();
            let lower_price_bracket = scale_buy_matches.value_of("lower_price_bracket").unwrap();
            let upper_price_bracket = scale_buy_matches.value_of("upper_price_bracket").unwrap();
            let interval = scale_buy_matches.value_of("interval").unwrap();

            let converted_total_order_size =
                total_order_size.parse::<f64>().unwrap() / interval.parse::<f64>().unwrap();

            println!(
                "converted_total_order_size: {}, asset_symbol: {}, lower_price_bracket: {}, upper_price_bracket: {}, interval: {}",
                converted_total_order_size,
                asset_symbol,
                lower_price_bracket,
                upper_price_bracket,
                interval
            );

            //Handle Scale Sell  <total order size/number of intervals> <asset symbol> <lower price bracket> <upper price bracket>
        }
        ("sell", Some(scale_sell_matches)) => {
            let total_order_size = scale_sell_matches.value_of("total_order_size").unwrap();
            let interval = scale_sell_matches.value_of("interval").unwrap();
            let asset_symbol = scale_sell_matches.value_of("asset_symbol").unwrap();
            let lower_price_bracket = scale_sell_matches.value_of("lower_price_bracket").unwrap();
            let upper_price_bracket = scale_sell_matches.value_of("upper_price_bracket").unwrap();

            let converted_total_order_size =
                total_order_size.parse::<f64>().unwrap() / interval.parse::<f64>().unwrap();

            println!(
                "converted_total_order_size: {}, asset_symbol: {}, lower_price_bracket: {}, upper_price_bracket: {}, interval: {}",
                converted_total_order_size,
                asset_symbol,
                lower_price_bracket,
                upper_price_bracket,
                interval
            );
        }

        _ => {
            println!("No matching pattern");
        }
    }
}
