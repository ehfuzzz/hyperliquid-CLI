//using version 2.33 not the latest one

use crate::handlers::{
    handle_cross_margin, handle_isolated_margin, handle_notional_value, handle_risk_value,
};
use crate::helpers::{
    build_sl_order, build_tp_order, place_sl_order, place_tp_order, validate_limit_price,
    validate_sl_price, validate_tp_price, validate_value, validate_value_size,
};
use crate::hyperliquid::meta_info::calculate_asset_to_id;
use crate::hyperliquid::open_orders::{get_side_from_oid, get_sz_from_oid};

use crate::hyperliquid::order::{build_buy_order, build_sell_order};
use crate::hyperliquid::order_payload::{Limit, OrderType, Orders};
use clap::{App, Arg};
use std::num::ParseFloatError;

pub async fn cli() {
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
                .subcommand(
                    App::new("dl")
                        .about("Sets the default leverage")
                        .arg(
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
                                })
                        )
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
                .help(
                    " The way we call it is: buy --size '$100' --asset eth --price @1900 --sl 1920 --tp 1865"
                )
                .arg(
                    Arg::with_name("order_size")
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
                    Arg::with_name("limit_price")
                        .help("Limit price e.g ., @1900")
                        .long("price")
                        .takes_value(true)
                        .validator(|v| {
                            if v.starts_with("@") {
                                /*    If the parsing is successful, it returns Ok(()) (indicating success but discarding the float).
                                        If the parsing fails, it returns an error message as a String. */
                                v[1..]
                                    .parse::<f64>()
                                    .map(|_| ())
                                    .map_err(|e| e.to_string())
                            } else {
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
                    Arg::with_name("stop_loss").help("Stop loss value").long("sl").takes_value(true)
                )
        )
        .subcommand(
            App::new("sell")
                .about(" Handles the Sell command")
                .help(
                    " The way we call it is: sell --size '$100' --asset eth --price @1900 --sl 1920 --tp 1865"
                )
                .arg(
                    Arg::with_name("order_size")
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
                    Arg::with_name("limit_price")
                        .help("Limit price e.g ,. @1900")
                        .long("price")
                        .takes_value(true)
                        .validator(validate_limit_price)
                )
                .arg(
                    Arg::with_name("take_profit")
                        .help("Take profit value")
                        .long("tp")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("stop_loss").help("Stop Loss Value").long("sl").takes_value(true)
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
                                    if v.parse::<f64>().is_ok() {
                                        Ok(())
                                    } else {
                                        Err(String::from("Expected a numeric value"))
                                    }
                                })
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
                                    "Time between intervals in minutes, number of intervals e.g 10 means 10 minutes"
                                )
                        )
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
                                    if v.parse::<f64>().is_ok() {
                                        Ok(())
                                    } else {
                                        Err(String::from("Expected a numeric value"))
                                    }
                                })
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
                                    "comma separated values of: Time between intervals in minutes, number of intervals e.g 10 means 10 minutes"
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
                                    if v.parse::<f64>().is_ok() {
                                        Ok(())
                                    } else {
                                        Err(String::from("Expected a numeric value"))
                                    }
                                })
                        )
                        .arg(
                            Arg::with_name("pair")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .help("forward slash separated assets symbol to be pair traded")
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
                                .validator(validate_value)
                        )
                        .arg(
                            Arg::with_name("pair")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .help("Pair to be traded e.g BTC/USDT")
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
                                    if v.parse::<f64>().is_ok() {
                                        Ok(())
                                    } else {
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
                            Arg::with_name("total_order_size/interval")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help(
                                    "Forward slash separated total order size and number of intervals"
                                )
                        )
                        // .arg(
                        //     Arg::with_name("interval")
                        //         .required(true)
                        //         .index(2)
                        //         .takes_value(true)
                        //         .help("Number of orders to place ie Total Order Size / interval")
                        //         .validator(validate_value)
                        // )
                        .arg(
                            Arg::with_name("asset")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .help("asset symbol e.g ETH, SOL, BTC")
                        )
                        .arg(
                            Arg::with_name("lower_price_bracket")
                                .required(true)
                                .index(3)
                                .takes_value(true)
                                .help("Price to start buying from")
                                .validator(validate_value)
                        )
                        .arg(
                            Arg::with_name("upper_price_bracket")
                                .required(true)
                                .index(4)
                                .takes_value(true)
                                .help("Price to stop buying at")
                                .validator(validate_value)
                        )
                )
                .subcommand(
                    App::new("sell")
                        .about("scale sell")
                        .arg(
                            Arg::with_name("total_order_size/interval")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help(
                                    "Forward slash separated total order size and number of intervals"
                                )
                        )
                        .arg(
                            Arg::with_name("asset")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .help("asset symbol e.g ETH, SOL, BTC")
                        )
                        .arg(
                            Arg::with_name("lower_price_bracket")
                                .required(true)
                                .index(3)
                                .takes_value(true)
                                .help("Price to start selling from")
                                .validator(validate_value)
                        )
                        .arg(
                            Arg::with_name("upper_price_bracket")
                                .required(true)
                                .index(4)
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
            let percentage_order: f64 = percentage_order
                .trim_end_matches("%")
                .parse::<f64>()
                .unwrap();

            let asset = tp_matches.value_of("asset").unwrap();
            let asset: u32 = calculate_asset_to_id(&asset);
            let tp_price = tp_matches.value_of("tp_price").unwrap();
            let oid = 1234567;
            let sz: f64 = get_sz_from_oid(oid) * percentage_order / 100.0;
            let sz: String = sz.to_string();
            let reduce_only = false;
            let is_buy: bool = get_side_from_oid(oid);

            let limit_px = "1900";

            match tp_price {
                tp_price
                    if tp_price.ends_with("%")
                        || tp_price.starts_with("$")
                        || tp_price.ends_with("%pnl")
                        || tp_price.ends_with("pnl") =>
                {
                    place_tp_order(asset, is_buy, tp_price, limit_px, &sz, reduce_only, true).await;
                }
                tp_price if validate_value(tp_price.to_string()).is_ok() => {
                    place_tp_order(asset, is_buy, tp_price, limit_px, &sz, reduce_only, false)
                        .await;
                }
                _ => {
                    println!("No matching pattern");
                }
            }
        }
        ("sl", Some(sl_matches)) => {
            let percentage_order = sl_matches.value_of("percentage_order").unwrap();
            let percentage_order: f64 = percentage_order
                .trim_end_matches("%")
                .parse::<f64>()
                .unwrap();

            let asset = sl_matches.value_of("asset").unwrap();
            let asset: u32 = calculate_asset_to_id(&asset);
            let sl_price = sl_matches.value_of("sl_price").unwrap();
            let oid = 1234567;
            let sz: f64 = get_sz_from_oid(oid) * percentage_order / 100.0;
            let sz: String = sz.to_string();
            let reduce_only = false;
            let is_buy: bool = get_side_from_oid(oid);
            let limit_px = "1900";

            // Inside your original function
            match sl_price {
                sl_price
                    if sl_price.trim_start_matches("-").ends_with("%")
                        || sl_price.starts_with("-$")
                        || sl_price.trim_start_matches("-").ends_with("%pnl")
                        || sl_price.trim_start_matches("-").ends_with("pnl") =>
                {
                    place_sl_order(asset, is_buy, sl_price, limit_px, &sz, reduce_only, true).await;
                }
                sl_price if validate_value(sl_price.to_string()).is_ok() => {
                    place_sl_order(asset, is_buy, sl_price, limit_px, &sz, reduce_only, false)
                        .await;
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

            let mut buy_order = Orders::new();
            let limit: Limit = Limit::new();
            let reduce_only = false;
            let is_buy = true;

            let mut tp_order: Option<Orders> = None;
            let mut sl_order: Option<Orders> = None;

            buy_order.set_reduce_only(reduce_only);
            buy_order.set_is_buy(is_buy);
            buy_order.set_order_type(OrderType::Limit(limit));

            if let Some(size) = buy_size {
                //preprocess the String to get the numeric size
                let numeric_part = &size[1..].parse::<f64>().unwrap();
                buy_order.set_sz(&numeric_part.to_string());
                println!("Buy size: {}", numeric_part);
            } else {
                //Filled with the default size already set
                let default_size = 100;
                buy_order.set_sz(&default_size.to_string());
                println!("Filled with the default size already specified");
            }
            if let Some(symbol) = asset {
                let asset = calculate_asset_to_id(&symbol);
                buy_order.set_asset(asset);
                println!("Asset symbol: {}", symbol);
            } else {
                //Filled with the default size already set
                let default_asset = "ETH";
                let default_asset = calculate_asset_to_id(default_asset);
                buy_order.set_asset(default_asset);
                println!("Filled with the default symbol already specified");
            }
            if let Some(price) = limit_price {
                let numeric_part = &price[1..].parse::<f64>().unwrap();
                buy_order.set_limit_px(&numeric_part.to_string());
                println!("Limit price: {}", numeric_part);
            } else {
                //Filled with the default size already set
                let market_price = 1990;
                buy_order.set_limit_px(&market_price.to_string());
                println!("Filled with the default limit rules already specified");
            }
            if let Some(tp) = take_profit {
                // here we need to build a tp order different from the buy order

                match tp {
                    tp if tp.ends_with("%")
                        || tp.starts_with("$")
                        || tp.ends_with("%pnl")
                        || tp.ends_with("pnl") =>
                    {
                        tp_order = build_tp_order(
                            buy_order.get_asset(),
                            is_buy,
                            &buy_order.get_limit_px(),
                            tp,
                            &buy_order.get_sz(),
                            reduce_only,
                            false,
                        );
                    }
                    tp if validate_value(tp.to_string()).is_ok() => {
                        tp_order = build_tp_order(
                            buy_order.get_asset(),
                            is_buy,
                            &buy_order.get_limit_px(),
                            tp,
                            &buy_order.get_sz(),
                            reduce_only,
                            false,
                        );
                    }
                    _ => {
                        println!("No matching pattern");
                    }
                }

                let numeric_part = &tp.parse::<f64>().unwrap();
                println!("Take profit: {}", numeric_part);
            } else {
                //Filled with the default size already set
                println!("No TP was provided");
            }

            if let Some(sl) = stop_loss {
                let numeric_part = &sl.parse::<f64>().unwrap();
                match sl {
                    sl if sl.trim_start_matches("-").ends_with("%")
                        || sl.starts_with("-$")
                        || sl.trim_start_matches("-").ends_with("%pnl")
                        || sl.trim_start_matches("-").ends_with("pnl") =>
                    {
                        sl_order = build_sl_order(
                            buy_order.get_asset(),
                            is_buy,
                            &buy_order.get_limit_px(),
                            sl,
                            &buy_order.get_sz(),
                            reduce_only,
                            false,
                        );
                    }
                    sl if validate_value(sl.to_string()).is_ok() => {
                        sl_order = build_sl_order(
                            buy_order.get_asset(),
                            is_buy,
                            &buy_order.get_limit_px(),
                            sl,
                            &buy_order.get_sz(),
                            reduce_only,
                            false,
                        );
                    }
                    _ => {
                        println!("No matching pattern");
                    }
                }
                println!("Stop Loss: {}", numeric_part);
            } else {
                //Filled with the default size already set
                println!("No sell was provided");
            }

            let buy_payload = build_buy_order(buy_order, tp_order, sl_order);
            println!("Buy payload Confirmation: {:#?}", buy_payload);
        }

        ("sell", Some(sell_matches)) => {
            let sell_size = sell_matches.value_of("order_size");
            let asset = sell_matches.value_of("asset");
            let limit_price = sell_matches.value_of("limit_price");
            let take_profit = sell_matches.value_of("take_profit");
            let stop_loss = sell_matches.value_of("stop_loss");

            let mut sell_order = Orders::new();
            let limit: Limit = Limit::new();
            let reduce_only = false;
            let is_buy = false;

            let mut tp_order: Option<Orders> = None;
            let mut sl_order: Option<Orders> = None;

            sell_order.set_reduce_only(reduce_only);
            sell_order.set_order_type(OrderType::Limit(limit));

            if let Some(size) = sell_size {
                let numeric_part = &size[1..].parse::<f64>().unwrap();
                sell_order.set_sz(&numeric_part.to_string());
                println!("Sell size: {}", numeric_part);
            } else {
                let default_size = 100;
                sell_order.set_sz(&default_size.to_string());
                println!("Fill with the default size already specified");
            }

            if let Some(symbol) = asset {
                let asset = calculate_asset_to_id(&symbol);
                sell_order.set_asset(asset);
                println!("Asset symbol: {}", symbol);
            } else {
                let default_asset = "ETH";
                let default_asset = calculate_asset_to_id(default_asset);
                sell_order.set_asset(default_asset);
                println!("FIlled with the default asset already specified");
            }
            if let Some(price) = limit_price {
                let numeric_part = &price[1..].parse::<f64>().unwrap();
                sell_order.set_limit_px(&numeric_part.to_string());
                println!(" Limit price: {}", numeric_part);
            } else {
                //Filled with the default size already set
                let market_price = 1990;
                sell_order.set_limit_px(&market_price.to_string());
                println!("FIlled with the default limit price already specified");
            }

            if let Some(sl) = stop_loss {
                let numeric_part = &sl.parse::<f64>().unwrap();
                match sl {
                    sl if sl.trim_start_matches("-").ends_with("%")
                        || sl.starts_with("-$")
                        || sl.trim_start_matches("-").ends_with("%pnl")
                        || sl.trim_start_matches("-").ends_with("pnl") =>
                    {
                        sl_order = build_sl_order(
                            sell_order.get_asset(),
                            is_buy,
                            &sell_order.get_limit_px(),
                            sl,
                            &sell_order.get_sz(),
                            reduce_only,
                            false,
                        );
                    }
                    sl if validate_value(sl.to_string()).is_ok() => {
                        sl_order = build_sl_order(
                            sell_order.get_asset(),
                            is_buy,
                            &sell_order.get_limit_px(),
                            sl,
                            &sell_order.get_sz(),
                            reduce_only,
                            false,
                        );
                    }
                    _ => {
                        println!("No matching pattern");
                    }
                }
                println!("Stop loss set at: {}", numeric_part);
            } else {
                println!("No SL was provided");
            }

            if let Some(tp) = take_profit {
                let numeric_part = &tp.parse::<f64>().unwrap();
                match tp {
                    tp if tp.ends_with("%")
                        || tp.starts_with("$")
                        || tp.ends_with("%pnl")
                        || tp.ends_with("pnl") =>
                    {
                        tp_order = build_tp_order(
                            sell_order.get_asset(),
                            is_buy,
                            &sell_order.get_limit_px(),
                            tp,
                            &sell_order.get_sz(),
                            reduce_only,
                            false,
                        );
                    }
                    tp if validate_value(tp.to_string()).is_ok() => {
                        tp_order = build_tp_order(
                            sell_order.get_asset(),
                            is_buy,
                            &sell_order.get_limit_px(),
                            tp,
                            &sell_order.get_sz(),
                            reduce_only,
                            false,
                        );
                    }
                    _ => {
                        println!("No matching pattern");
                    }
                }
                println!("Take profit set at : {}", numeric_part);
            } else {
                println!("No TP was provided");
            }

            let sell_payload = build_sell_order(sell_order, tp_order, sl_order);
            println!("Sell payload Confirmation: {:#?}", sell_payload);
        }

        ("scale", Some(scale_matches)) => {
            match scale_matches.subcommand() {
                ("buy", Some(scale_buy_matches)) => {
                    let total_order_size: Vec<&str> = scale_buy_matches
                        .value_of("total_order_size/interval")
                        .unwrap()
                        .split("/")
                        .collect();

                    let asset = scale_buy_matches.value_of("asset").unwrap();
                    let lower_price_bracket =
                        scale_buy_matches.value_of("lower_price_bracket").unwrap();
                    let upper_price_bracket =
                        scale_buy_matches.value_of("upper_price_bracket").unwrap();

                    let converted_total_order_size = total_order_size[0].parse::<f64>().unwrap()
                        / total_order_size[1].parse::<f64>().unwrap();

                    let interval = total_order_size[1].parse::<f64>().unwrap();

                    println!(
                        "Buy {}{} each with limit orders at {}, {}, {}, {}, {}...,{}
{}{} bought in total with {} limit orders",
                        converted_total_order_size,
                        asset,
                        lower_price_bracket,
                        lower_price_bracket.parse::<i32>().unwrap() + ((interval * 1.0) as i32),
                        lower_price_bracket.parse::<i32>().unwrap() + ((interval * 2.0) as i32),
                        lower_price_bracket.parse::<i32>().unwrap() + ((interval * 3.0) as i32),
                        lower_price_bracket.parse::<i32>().unwrap() + ((interval * 4.0) as i32),
                        upper_price_bracket,
                        total_order_size[0].parse::<f64>().unwrap(),
                        asset,
                        total_order_size[1].parse::<f64>().unwrap()
                    );

                    //Handle Scale Sell  <total order size/number of intervals> <asset symbol> <lower price bracket> <upper price bracket>
                }

                ("sell", Some(scale_sell_matches)) => {
                    let total_order_size: Vec<&str> = scale_sell_matches
                        .value_of("total_order_size/interval")
                        .unwrap()
                        .split("/")
                        .collect();

                    let asset = scale_sell_matches.value_of("asset").unwrap();
                    let lower_price_bracket =
                        scale_sell_matches.value_of("lower_price_bracket").unwrap();
                    let upper_price_bracket =
                        scale_sell_matches.value_of("upper_price_bracket").unwrap();

                    let converted_total_order_size = total_order_size[0].parse::<f64>().unwrap()
                        / total_order_size[1].parse::<f64>().unwrap();

                    let interval = total_order_size[1].parse::<f64>().unwrap();

                    println!(
                        "Sell {}{} each with limit orders at {}, {}, {}, {}, {},...,{}
{}{} sold in total with {} limit orders",
                        converted_total_order_size,
                        asset,
                        lower_price_bracket,
                        lower_price_bracket.parse::<i32>().unwrap() + ((interval * 1.0) as i32),
                        lower_price_bracket.parse::<i32>().unwrap() + ((interval * 2.0) as i32),
                        lower_price_bracket.parse::<i32>().unwrap() + ((interval * 3.0) as i32),
                        lower_price_bracket.parse::<i32>().unwrap() + ((interval * 4.0) as i32),
                        upper_price_bracket,
                        total_order_size[0].parse::<f64>().unwrap(),
                        asset,
                        total_order_size[1].parse::<f64>().unwrap()
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

                    println!(
                        "twap sell order size: {}, asset-symbol: {}, intervals: {:?}-> Interval1: {:?}",
                        order_size,
                        asset,
                        intervals,
                        intervals.get(0)
                    );

                    let interval_minutes: f64 =
                        intervals[0].parse().expect("Invalid Interval Value");
                    let interval_range: f64 = intervals[1].parse().expect("Invalid interval Value");

                    let amount_asset = order_size / interval_range;

                    println!(
                        "Buying {} {} at intervals of {} minutes ",
                        amount_asset, asset, interval_minutes
                    );

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

                    println!(
                        "twap sell order size: {}, asset-symbol: {}, intervals: {:?}-> Interval1: {:?}",
                        order_size,
                        asset,
                        intervals,
                        intervals.get(0)
                    );

                    let interval_minutes: f64 =
                        intervals[0].parse().expect("Invalid Internal Value");
                    let interval_range: f64 = intervals[1].parse().expect("Invalid Interval Value");

                    let amount_asset = order_size / interval_range;

                    println!(
                        "Selling {} {} at intervals of {} minutes",
                        amount_asset, asset, interval_minutes
                    );
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
                println!(
                        " Invalid command: expected commands: (view pnl, view wallet balance, view unfilled orders, view open positions"
                    );
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
                        .unwrap()
                        / 2.0;
                    let pair: Vec<&str> =
                        buy_matches.value_of("pair").unwrap().split("/").collect();

                    let limit_price = buy_matches.value_of("limit_price");
                    let stop_loss = buy_matches.value_of("stop_loss");
                    let take_profit = buy_matches.value_of("take_profit");
                    let pair_one: String =
                        pair[0].parse().expect("Expected a valid string literal");
                    let pair_two: String =
                        pair[1].parse().expect("Expected a valid string literal");

                    println!(
                        "Longing {} {} and shorting {} {}",
                        order_size, pair_one, order_size, pair_two
                    );

                    if let Some(lp) = limit_price {
                        println!("Entering at this : {} limit price", lp);
                    } else {
                        println!(" Entering at Market price");
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
                        .unwrap()
                        / 2.0;
                    let pair: Vec<&str> =
                        sell_matches.value_of("pair").unwrap().split("/").collect();

                    let limit_price = sell_matches.value_of("limit_price");
                    let stop_loss = sell_matches.value_of("stop_loss");
                    let take_profit = sell_matches.value_of("take_profit");
                    let pair_one: String =
                        pair[0].parse().expect("Expected a valid string literal");
                    let pair_two: String =
                        pair[1].parse().expect("Expected a valid string literal");

                    println!(
                        "Shorting {} {} and Longing {} {}",
                        order_size, pair_one, order_size, pair_two
                    );

                    if let Some(lp) = limit_price {
                        println!("Entering at this: {} market price", lp);
                    } else {
                        println!(" Entering at market price");
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
