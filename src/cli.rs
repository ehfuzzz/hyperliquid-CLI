use std::collections::HashMap;

//using version 2.33 not the latest one
use clap::{App, Arg};

use crate::helpers::{
    validate_limit_price, validate_sl_price, validate_tp_price, validate_value, validate_value_size,
};
use crate::hyperliquid::HyperLiquid;
use crate::model::{
    AssetPosition, ClearingHouseState, Limit, OrderRequest, OrderType, Tif, Trigger, TriggerType,
};
use crate::settings::Settings;

pub async fn cli(config: &Settings, hyperliquid: &HyperLiquid) {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("A CLI bot to interact with the hyperliquid exchange")
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
                .about("Handles the Buy command")
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

    let metadata = hyperliquid
        .metadata()
        .await
        .expect("Failed to fetch metadata");

    let assets = metadata
        .universe
        .into_iter()
        .map(|asset| (asset.name.to_uppercase(), asset.sz_decimals))
        .collect::<HashMap<String, u32>>();

    println!("{:#?}", assets);

    match matches.subcommand() {
        ("tp", Some(tp_matches)) => {
            let percentage_order = tp_matches.value_of("percentage_order").unwrap();
            let percentage_order: f64 = percentage_order
                .trim_end_matches("%")
                .parse::<f64>()
                .unwrap();

            // let asset = tp_matches.value_of("asset").unwrap();
            // let asset: u32 = calculate_asset_to_id(&asset);
            // let tp_price = tp_matches.value_of("tp_price").unwrap();
            // let oid = 1234567;
            // let sz: f64 = get_sz_from_oid(oid) * percentage_order / 100.0;
            // let sz: String = sz.to_string();
            // let reduce_only = false;
            // let is_buy: bool = get_side_from_oid(oid);

            let limit_px = "1900";

            // match tp_price {
            //     tp_price
            //         if tp_price.ends_with("%")
            //             || tp_price.starts_with("$")
            //             || tp_price.ends_with("%pnl")
            //             || tp_price.ends_with("pnl") =>
            //     {
            //         place_tp_order(asset, is_buy, tp_price, limit_px, &sz, reduce_only, true).await;
            //     }
            //     tp_price if validate_value(tp_price.to_string()).is_ok() => {
            //         place_tp_order(asset, is_buy, tp_price, limit_px, &sz, reduce_only, false)
            //             .await;
            //         println!("Logic for handling + 100: {}", &tp_price);
            //     }
            //     _ => {
            //         println!("No matching pattern");
            //     }
            // }
        }
        ("sl", Some(sl_matches)) => {
            let percentage_order = sl_matches.value_of("percentage_order").unwrap();
            let percentage_order: f64 = percentage_order
                .trim_end_matches("%")
                .parse::<f64>()
                .unwrap();

            // let asset = sl_matches.value_of("asset").unwrap();
            // let asset: u32 = calculate_asset_to_id(&asset);
            // let sl_price = sl_matches.value_of("sl_price").unwrap();
            // let oid = 1234567;
            // let sz: f64 = get_sz_from_oid(oid) * percentage_order / 100.0;
            // let sz: String = sz.to_string();
            // let reduce_only = false;
            // let is_buy: bool = get_side_from_oid(oid);
            // let limit_px = "1900";

            // match sl_price {
            //     sl_price
            //         if sl_price.trim_start_matches("-").ends_with("%")
            //             || sl_price.starts_with("-$")
            //             || sl_price.trim_start_matches("-").ends_with("%pnl")
            //             || sl_price.trim_start_matches("-").ends_with("pnl") =>
            //     {
            //         place_sl_order(asset, is_buy, sl_price, limit_px, &sz, reduce_only, true).await;
            //     }
            //     sl_price if validate_value(sl_price.to_string()).is_ok() => {
            //         place_sl_order(asset, is_buy, sl_price, limit_px, &sz, reduce_only, false)
            //             .await;
            //     }
            //     _ => {
            //         println!("No matching pattern");
            //     }
            // }

            //Handle Scale Buy  <total order size/number of intervals> <asset symbol> <lower price bracket> <upper price bracket>
        }

        ("buy", Some(buy_matches)) => {
            let order_size = buy_matches.value_of("order_size");
            let asset = buy_matches.value_of("asset");
            let limit_price = buy_matches.value_of("limit_price");
            let take_profit = buy_matches.value_of("take_profit");
            let stop_loss = buy_matches.value_of("stop_loss");

            let asset = asset.unwrap_or_else(|| &config.default_asset.value);

            let triger_px = take_profit
                .unwrap_or("0")
                .parse::<f64>()
                .expect("Expected a numeric value for take profit");

            let mut orders: Vec<OrderRequest> = Vec::new();

            let order_type = OrderType::Limit(Limit { tif: Tif::Gtc });

            let sz_decimals = *assets
                .get(&asset.to_uppercase())
                .expect("Failed to find asset");

            let asset_ctx = hyperliquid
                .asset_ctx(asset)
                .await
                .expect("Failed to fetch asset ctxs")
                .expect("Failed to find asset");

            let mark_price = asset_ctx.mark_px.parse::<f64>().unwrap();

            let sz = {
                let sz = order_size.unwrap_or_else(|| &config.default_size.size)[1..].to_string();

                println!("Sz: {}", sz);

                let sz = sz
                    .parse::<f64>()
                    .expect("Expected a numeric value for order size");

                let sz = (sz / mark_price) as f64;

                format!("{:.*}", sz_decimals as usize, sz)
            };

            let limit_px = limit_price.unwrap_or(&format!("@{}", mark_price))[1..].to_string();

            println!("Limit Px: {}", limit_px);
            println!("Sz: {}", sz);
            println!("Sz Decimals: {}", sz_decimals);

            let asset = sz_decimals;

            let order = OrderRequest {
                asset,
                is_buy: true,
                limit_px: limit_px.clone(),
                sz: sz.clone(),
                reduce_only: false,
                order_type,
            };

            orders.push(order);

            if triger_px > 0.0 {
                let order_type = OrderType::Trigger(Trigger {
                    triger_px,
                    is_market: true,
                    tpsl: TriggerType::Tp,
                });

                let order = OrderRequest {
                    asset,
                    is_buy: true,
                    limit_px: limit_px.clone(),
                    sz: sz.clone(),
                    reduce_only: false,
                    order_type,
                };

                orders.push(order);
            };

            let triger_px = stop_loss
                .unwrap_or("0")
                .parse::<f64>()
                .expect("Expected a numeric value for stop loss");

            if triger_px > 0.0 {
                let order_type = OrderType::Trigger(Trigger {
                    triger_px,
                    is_market: true,
                    tpsl: TriggerType::Sl,
                });

                let order = OrderRequest {
                    asset,
                    is_buy: true,
                    limit_px,
                    sz,
                    reduce_only: false,
                    order_type,
                };

                orders.push(order);
            };

            let res = hyperliquid.place_order(orders).await;

            println!("{:#?}", res);
        }

        ("sell", Some(sell_matches)) => {
            let order_size = sell_matches.value_of("order_size");
            let asset = sell_matches.value_of("asset");
            let limit_price = sell_matches.value_of("limit_price");
            let take_profit = sell_matches.value_of("take_profit");
            let stop_loss = sell_matches.value_of("stop_loss");

            let sz = order_size
                .unwrap_or_else(|| &config.default_size.size)
                .to_string();

            let asset = 4;
            // asset
            //     .unwrap_or_else(|| &config.default_asset.value)
            //     .to_string();

            let limit_px = limit_price.unwrap_or_default().to_string();

            let triger_px = take_profit
                .unwrap_or("0")
                .parse::<f64>()
                .expect("Expected a numeric value for take profit");

            let mut orders: Vec<OrderRequest> = Vec::new();

            let order_type = OrderType::Limit(Limit { tif: Tif::Gtc });

            let order = OrderRequest {
                asset: asset.clone(),
                is_buy: false,
                limit_px: limit_px.clone(),
                sz: sz.clone(),
                reduce_only: false,
                order_type,
            };

            orders.push(order);

            if triger_px > 0.0 {
                let order_type = OrderType::Trigger(Trigger {
                    triger_px,
                    is_market: true,
                    tpsl: TriggerType::Tp,
                });

                let order = OrderRequest {
                    asset: asset.clone(),
                    is_buy: false,
                    limit_px: limit_px.clone(),
                    sz: sz.clone(),
                    reduce_only: false,
                    order_type,
                };

                orders.push(order);
            };

            let triger_px = stop_loss
                .unwrap_or("0")
                .parse::<f64>()
                .expect("Expected a numeric value for stop loss");

            if triger_px > 0.0 {
                let order_type = OrderType::Trigger(Trigger {
                    triger_px,
                    is_market: true,
                    tpsl: TriggerType::Sl,
                });

                let order = OrderRequest {
                    asset,
                    is_buy: false,
                    limit_px,
                    sz,
                    reduce_only: false,
                    order_type,
                };

                orders.push(order);
            };

            hyperliquid.place_order(orders).await;
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
                let res = hyperliquid.pnl().await;

                println!("{:#?}", res);
            }
            Some("wallet") => {
                let state = hyperliquid
                    .clearing_house_state()
                    .await
                    .expect("Failed to fetch wallet balance");

                let margin_summary = state.margin_summary;

                let repeat = 35;
                println!("{}", format!("{}", "-".repeat(repeat)));

                println!("Margin Wallet Summary");
                println!("{}", format!("{}", "-".repeat(repeat)));
                println!("Account Value: {}", margin_summary.account_value);
                println!("Total Margin Used: {}", margin_summary.total_margin_used);
                println!("Total Ntl Position: {}", margin_summary.total_ntl_pos);
                println!("Total Raw Usd : {}", margin_summary.total_raw_usd);

                let cms = state.cross_margin_summary;

                println!();
                println!("Cross Margin Wallet Summary");
                println!("{}", format!("{}", "-".repeat(repeat)));
                println!("Account Value: {}", cms.account_value);
                println!("Total Margin Used: {}", cms.total_margin_used);
                println!("Total Ntl Position: {}", cms.total_ntl_pos);
                println!("Total Raw Usd : {}", cms.total_raw_usd);
            }
            Some("unfilled") => {
                println!("Implement view unfilled orders logic");
            }
            Some("open") => {
                let state = hyperliquid.clearing_house_state().await.unwrap();

                let open_positions = state
                    .asset_positions
                    .iter()
                    .filter(|ap| ap.position.entry_px.is_some())
                    .collect::<Vec<_>>();

                let repeat = 35;
                for ap in open_positions {
                    let entry_position = ap.position.entry_px.as_ref().unwrap();

                    println! ("{}", format!("{}", "_".repeat(repeat)));
                    println!();
                    println!("Asset: {}", ap.position.coin);
                    println!("Entry Price: {:#?}", entry_position);
                    println!("Position Size: {}", format!("{}",ap.position.szi));
                    println!("Position Value: {}", format!("${}",ap.position.position_value));
                    println!("Return on Equity: {}", format!("{}%", ap.position.return_on_equity));
                    println!("Unrealized Pnl: {}",format!("${}", ap.position.unrealized_pnl));
                }
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
