//using version 2.33 not the latest one
use clap::{App, Arg};
use std::num::ParseFloatError;

mod handlers;
mod helpers;
mod settings;

use handlers::{
    handle_cross_margin, handle_isolated_margin, handle_notional_value, handle_risk_value,
};
use helpers::{
    validate_limit_price, validate_sl_price, validate_tp_price, validate_value, validate_value_size,
};
use settings::Settings;

#[tokio::main]
async fn main() {
    let _settings = Settings::new().expect("Failed to load config");

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
                                .help("Either risk or notional"),
                        )
                        .arg(
                            Arg::with_name("value_size")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .validator(validate_value_size)
                                .help("Size in USDC or size in % of balance"),
                        ),
                )
                .subcommand(
                    App::new("dm").about("Sets the default margin").arg(
                        Arg::with_name("margin_type")
                            .required(true)
                            .index(1)
                            .takes_value(true)
                            .possible_values(&["i", "c"])
                            .help("Default margin type Either Isolated(i) or cross margin(c)"),
                    ),
                )
                .subcommand(
                    App::new("dl").about("Sets the default leverage").arg(
                        Arg::with_name("amount")
                            .required(true)
                            .index(1)
                            .takes_value(true)
                            .help("Amount of leverage")
                            .validator(|v| {
                                if v.parse::<f64>().is_ok() {
                                    Ok(())
                                } else {
                                    Err(String::from("Expected a numeric value"))
                                }
                            }),
                    ),
                )
                .subcommand(
                    App::new("da")
                        .about("Sets the default instrument to trade")
                        .arg(
                            Arg::with_name("asset")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("asset to be traded")
                        ),
                ),
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
                    Arg::with_name("asset")
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
                    Arg::with_name("asset")
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
            App::new("buy")
                .about(" Handles the Buy command")
                .arg(
                    Arg::with_name("order_size")
                        .help("size of the order e.g ., $100 ")
                        .long("size")
                        .takes_value(true)
                        .validator(|v| {
                            if v.starts_with('$') {
                                /*    If the parsing is successful, it returns Ok(()) (indicating success but discarding the float).
                                If the parsing fails, it returns an error message as a String. */
                                v[1..].parse::<f64>().map(|_| ()).map_err(|e| e.to_string())
                            }else {
                                Err(String::from("Expected a $ symbol at the start"))
                            }
                        }),
                )
                .arg(
                    Arg::with_name("asset")
                        .help("Asset symbol e.g ETH, SOL, BTC, optional if default asset is defined")
                        .long("symbol")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("limit_price")
                        .help("Limit price e.g ., @1900")
                        .long("price")
                        .takes_value(true)
                        .validator(|v| {
                            if v.starts_with("@"){
                                /*    If the parsing is successful, it returns Ok(()) (indicating success but discarding the float).
                                    If the parsing fails, it returns an error message as a String. */
                                v[1..].parse::<f64>().map(|_| (())).map_err(|e| e.to_string())
                            }else{
                                Err(String::from("Expected an @ symbol at the start"))
                            }
                        })
                )
                .arg(
                    Arg::with_name("take_profit")
                        .help("Take profit value")
                        .long("tp")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("stop_loss")
                        .help("Stop loss value")
                        .long("sl")
                        .takes_value(true),
                )
        )
        .subcommand(
            App::new("twap")
                .about("Handles the twap buy and twap sell logic")
                .subcommand(
                    App::new("buy")
                        .about("twap buy")
                        .arg(
                            Arg::with_name("order_size")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("Total Order Size")
                                .validator(|v| {
                                    if v.parse::<f64>().is_ok(){
                                        Ok(())
                                    }else {
                                        Err(String::from("Expected a numeric value"))
                                    }
                                }),
                        )
                        .arg(
                            Arg::with_name("asset")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .help("asset to be traded"),
                        )
                        .arg(
                            Arg::with_name("interval")
                                .required(true)
                                .index(3)
                                .takes_value(true)
                                .help("Time between intervals in minutes, number of intervals e.g 10 means 10 minutes")
                        ),
                )
                .subcommand(
                    App::new("sell")
                        .about("twap sell")
                        .arg(
                            Arg::with_name("order_size")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("Total Order Size")
                                .validator(|v| {
                                    if v.parse::<f64>().is_ok(){
                                        Ok(())
                                    }else {
                                        Err(String::from("Expected a numeric value"))
                                    }
                                }),
                        )
                        .arg(
                            Arg::with_name("asset")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .help("asset to be traded"),
                        )
                        .arg(
                            Arg::with_name("interval")
                                .required(true)
                                .index(3)
                                .takes_value(true)
                                .help("comma separated values of: Time between intervals in minutes, number of intervals e.g 10 means 10 minutes")
                        ),
                )
        )
        .subcommand(
            App::new("view")
                .about("Handles the view commands")
                .subcommand(
                    App::new("pnl")
                        .about("view pnl")
                        .help("Use to display the account's PNL")
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
                .about("Handles the pair buy and pair sell logic")
                .subcommand(
                    App::new("buy")
                        .about("pair to buy")
                        .arg(
                            Arg::with_name("order_size")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("Total Order Size")
                                .validator(|v| {
                                    if v.parse::<f64>().is_ok(){
                                        Ok(())
                                    }else {
                                        Err(String::from("Expected a numeric value"))
                                    }
                                }),
                        )
                        .arg(
                            Arg::with_name("pair")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .help("forward slash separated assets symbol to be pair traded"),
                        )
                        .arg(
                            Arg::with_name("limit_price")
                                .required(false)
                                .index(3)
                                .takes_value(true)
                                .help("Limit price if applicable ")
                                .validator(validate_limit_price)
                        )
                        .arg(
                            Arg::with_name("stop_loss")
                                .required(false)
                                .index(4)
                                .takes_value(true)
                                .help("stop loss if applicable")
                                .validator(validate_value)
                        )
                        .arg(
                            Arg::with_name("take_profit")
                                .required(false)
                                .index(5)
                                .takes_value(true)
                                .help("Take profit if applicable")
                                .validator(validate_value)
                        )
                )
                .subcommand(
                    App::new("sell")
                        .about("pair to sell")
                        .arg(
                            Arg::with_name("order_size")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("Total Order Size")
                                .validator(validate_value),
                        )
                        .arg(
                            Arg::with_name("pair")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .help("Pair to be traded e.g BTC/USDT")
                                ,
                        )
                        .arg(
                            Arg::with_name("limit_price")
                                .required(false)
                                .index(3)
                                .takes_value(true)
                                .help("Limit price if applicable ")
                                .validator(validate_limit_price)
                        )
                        .arg(
                            Arg::with_name("stop_loss")
                                .required(false)
                                .index(4)
                                .takes_value(true)
                                .help("stop loss if applicable")
                                .validator(|v| {
                                    if v.parse::<f64>().is_ok(){
                                        Ok(())
                                    }else {
                                        Err(String::from("Expected a numeric value"))
                                    }
                                })
                        )
                        .arg(
                            Arg::with_name("take_profit")
                                .required(false)
                                .index(5)
                                .takes_value(true)
                                .help("Take profit if applicable")
                                .validator(validate_value)
                        )
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
                                .validator(validate_value)
                        )
                        .arg(
                            Arg::with_name("interval")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .help("Number of orders to place ie Total Order Size / interval")
                                .validator(validate_value)
                        )
                        .arg(
                            Arg::with_name("asset")
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
                                .validator(validate_value)
                        )
                        .arg(
                            Arg::with_name("upper_price_bracket")
                                .required(true)
                                .index(5)
                                .takes_value(true)
                                .help("Price to stop buying at")
                                .validator(validate_value)
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
                                .validator(validate_value)
                        )
                        .arg(
                            Arg::with_name("interval")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .help("Number of orders to place ie Total Order Size / interval")
                                .validator(validate_value)
                        )
                        .arg(
                            Arg::with_name("asset")
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
                                .validator(validate_value)
                        )
                        .arg(
                            Arg::with_name("upper_price_bracket")
                                .required(true)
                                .index(5)
                                .takes_value(true)
                                .help("Price to stop selling at")
                                .validator(validate_value)
                        )
                )
            )
        .get_matches();

    match matches.subcommand() {
        ("set", Some(set_matches)) => match set_matches.subcommand() {
            ("ds", Some(ds_matches)) => {
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
            }
            ("dm", Some(dm_matches)) => {
                let margin_type = dm_matches.value_of("margin_type").unwrap();
                println!("Margin type: {}", margin_type);

                match margin_type {
                    "i" => handle_isolated_margin(margin_type),
                    "c" => handle_cross_margin(margin_type),
                    _ => unreachable!(), // we should not get here because of the possible value checker
                }
            }

            ("da", Some(da_match)) => {
                let asset = da_match.value_of("asset").unwrap();
                println!("You have set {} as your default asset to be traded", asset)
            }
            ("dl", Some(dl_match)) => {
                let leverage = dl_match.value_of("amount").unwrap().parse::<f64>().unwrap();
                println!("You have set {} as your default leverage size", leverage);
            }
            _ => {
                println!("No subcommand was used");
            }
        },

        ("tp", Some(tp_matches)) => {
            let percentage_order = tp_matches.value_of("percentage_order").unwrap();
            let asset = tp_matches.value_of("asset").unwrap();
            let tp_price = tp_matches.value_of("tp_price").unwrap();

            let converted_percentage_order: Result<f64, ParseFloatError> = {
                percentage_order
                    .trim_end_matches("%")
                    .parse::<f64>()
                    .map(|percent| percent / 100.0)
            };
            println!(
                "converted percentage order: {:?}, asset: {}",
                converted_percentage_order, asset
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
            let asset = sl_matches.value_of("asset").unwrap();
            let sl_price = sl_matches.value_of("sl_price").unwrap();

            let converted_percentage_order: Result<f64, ParseFloatError> = {
                percentage_order
                    .trim_end_matches("%")
                    .parse::<f64>()
                    .map(|percent| percent / 100.0)
            };
            println!(
                "converted percentage order: {:?}, asset: {}",
                converted_percentage_order, asset
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

        ("buy", Some(buy_matches)) => {
            let buy_size = buy_matches.value_of("order_size");
            let asset = buy_matches.value_of("asset");
            let limit_price = buy_matches.value_of("limit_price");
            let take_profit = buy_matches.value_of("take_profit");
            let stop_loss = buy_matches.value_of("stop_loss");

            if let Some(size) = buy_size {
                //preprocess the String to get the numeric size
                let numeric_part = &size[1..].parse::<f64>().unwrap();
                println!("Buy size: {}", numeric_part)
            } else {
                //Filled with the default size already set
                println!("Filled with the default size already specified")
            }
            if let Some(symbol) = asset {
                println!("Asset symbol: {}", symbol);
            } else {
                //Filled with the default size already set
                println!("Filled with the default symbol already specified")
            }
            if let Some(price) = limit_price {
                let numeric_part = &price[1..].parse::<f64>().unwrap();
                println!("Limit price: {}", numeric_part);
            } else {
                //Filled with the default size already set
                println!("Filled with the default limit rules already specified")
            }
            if let Some(tp) = take_profit {
                let numeric_part = &tp.parse::<f64>().unwrap();
                println!("Take profit: {}", numeric_part);
            } else {
                //Filled with the default size already set
                println!("Filled with the default tp rules already specified")
            }

            if let Some(sl) = stop_loss {
                let numeric_part = &sl.parse::<f64>().unwrap();
                println!("Stop Loss: {}", numeric_part);
            } else {
                //Filled with the default size already set
                println!("Filled with the default sl rules already specified")
            }
        }
        ("scale", Some(scale_matches)) => {
            match scale_matches.subcommand() {
                ("buy", Some(scale_buy_matches)) => {
                    let total_order_size = scale_buy_matches.value_of("total_order_size").unwrap();
                    let asset = scale_buy_matches.value_of("asset").unwrap();
                    let lower_price_bracket =
                        scale_buy_matches.value_of("lower_price_bracket").unwrap();
                    let upper_price_bracket =
                        scale_buy_matches.value_of("upper_price_bracket").unwrap();
                    let interval = scale_buy_matches.value_of("interval").unwrap();

                    let converted_total_order_size =
                        total_order_size.parse::<f64>().unwrap() / interval.parse::<f64>().unwrap();

                    println!(
                    "converted_total_order_size: {}, asset: {}, lower_price_bracket: {}, upper_price_bracket: {}, interval: {}",
                    converted_total_order_size,
                    asset,
                    lower_price_bracket,
                    upper_price_bracket,
                    interval
                );

                    //Handle Scale Sell  <total order size/number of intervals> <asset symbol> <lower price bracket> <upper price bracket>
                }
                ("sell", Some(scale_sell_matches)) => {
                    let total_order_size = scale_sell_matches.value_of("total_order_size").unwrap();
                    let interval = scale_sell_matches.value_of("interval").unwrap();
                    let asset = scale_sell_matches.value_of("asset").unwrap();
                    let lower_price_bracket =
                        scale_sell_matches.value_of("lower_price_bracket").unwrap();
                    let upper_price_bracket =
                        scale_sell_matches.value_of("upper_price_bracket").unwrap();

                    let converted_total_order_size =
                        total_order_size.parse::<f64>().unwrap() / interval.parse::<f64>().unwrap();

                    println!(
                    "converted_total_order_size: {}, asset: {}, lower_price_bracket: {}, upper_price_bracket: {}, interval: {}",
                    converted_total_order_size,
                    asset,
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
        ("twap", Some(twap_matches)) => {
            // twap buy <total order size> <asset symbol>  <time between interval in mins, number of intervals>

            match twap_matches.subcommand() {
                ("buy", Some(twapbuy_matches)) => {
                    let order_size = twapbuy_matches
                        .value_of("order_size")
                        .unwrap()
                        .parse::<f64>()
                        .unwrap();
                    let asset = twapbuy_matches.value_of("asset").unwrap();
                    let intervals: Vec<&str> = twapbuy_matches
                        .value_of("interval")
                        .unwrap()
                        .split(",")
                        .collect();

                    println! ("twap sell order size: {}, asset-symbol: {}, intervals: {:?}-> Interval1: {:?}", order_size, asset, intervals, intervals.get(0));

                    //twap sell <total order size> <asset symbol>  <time between interval in mins, number of intervals>
                }
                ("sell", Some(twapsell_matches)) => {
                    let order_size = twapsell_matches
                        .value_of("order_size")
                        .unwrap()
                        .parse::<f64>()
                        .unwrap();
                    let asset = twapsell_matches.value_of("asset").unwrap();
                    let intervals: Vec<&str> = twapsell_matches
                        .value_of("interval")
                        .unwrap()
                        .split(",")
                        .collect();

                    println! ("twap sell order size: {}, asset-symbol: {}, intervals: {:?}-> Interval1: {:?}", order_size, asset, intervals, intervals.get(0));
                }
                _ => {
                    println!("No subcommand was used");
                }
            }
        }
        ("view", Some(view_matches)) => match view_matches.subcommand_name() {
            Some("pnl") => {
                println!("Implement view pnl logic");
            }
            Some("wallet") => {
                println!("Implement view wallet balance logic");
            }
            Some("unfilled") => {
                println!("Implement view unfilled orders logic");
            }
            Some("open") => {
                println!("Implement view open positions logic")
            }
            _ => {
                println! (" Invalid command: expected commands: (view pnl, view wallet balance, view unfilled orders, view open positions");
            }
        },

        ("pair", Some(pair_matches)) => {
            //pair buy <Order Size> <Asset X/Asset Y> <@limit price, if applicable> <sl if applicable> <tp if applicable>

            match pair_matches.subcommand() {
                ("buy", Some(buy_matches)) => {
                    let order_size = buy_matches
                        .value_of("order_size")
                        .unwrap()
                        .parse::<f64>()
                        .unwrap();
                    let pair: Vec<&str> =
                        buy_matches.value_of("pair").unwrap().split("/").collect();
                    let limit_price = buy_matches.value_of("limit_price");
                    let stop_loss = buy_matches.value_of("stop_loss");
                    let take_profit = buy_matches.value_of("take_profit");

                    println!(
                        "pair buy order size: {}, pair: {:?}, asset_1: {:?}, asset_2: {:?}",
                        order_size,
                        pair,
                        pair.get(0),
                        pair.get(1)
                    );

                    if let Some(lp) = limit_price {
                        println!("Limit price provided: {}", lp);
                    } else {
                        println!(" The already set default limit price rules will be used");
                    }
                    if let Some(sl) = stop_loss {
                        println!("Stop loss provided: {}", sl);
                    } else {
                        println!(" The already set stop loss rules will be used");
                    }
                    if let Some(tp) = take_profit {
                        println!("Take profit provided: {}", tp);
                    } else {
                        println!(" The already set default take profit rules will be used");
                    }

                    //pair sell <Order Size> <Asset X/Asset Y> <@limit price, if applicable> <sl if applicable> <tp if applicable>
                }
                ("sell", Some(sell_matches)) => {
                    let order_size = sell_matches
                        .value_of("order_size")
                        .unwrap()
                        .parse::<f64>()
                        .unwrap();
                    let pair: Vec<&str> =
                        sell_matches.value_of("pair").unwrap().split("/").collect();
                    let limit_price = sell_matches.value_of("limit_price");
                    let stop_loss = sell_matches.value_of("stop_loss");
                    let take_profit = sell_matches.value_of("take_profit");

                    println!(
                        "pair sell order size: {}, pair: {:?}, asset_1: {:?}, asset_2: {:?}",
                        order_size,
                        pair,
                        pair.get(0),
                        pair.get(1)
                    );

                    if let Some(lp) = limit_price {
                        println!("Limit price provided: {}", lp);
                    } else {
                        println!(" The already set default limit price rules will be used");
                    }
                    if let Some(sl) = stop_loss {
                        println!("Stop loss provided: {}", sl);
                    } else {
                        println!(" The already set stop loss rules will be used");
                    }
                    if let Some(tp) = take_profit {
                        println!("Take profit provided: {}", tp);
                    } else {
                        println!(" The already set default take profit rules will be used");
                    }
                }

                _ => {
                    println!("Invalid Pair command: We only have pair buy and pair sell");
                }
            }
        }

        _ => {
            println!("Invalid command: expected commands: (buy, sell, twap, view, pair)");
        }
    }
}
