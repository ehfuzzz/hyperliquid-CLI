use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::thread;

//using version 2.33 not the latest one
use clap::{App, Arg};
use ethers::signers::LocalWallet;
use secrecy::ExposeSecret;

use crate::helpers::{
    format_limit_price, format_size, validate_limit_price, validate_sl_price, validate_tp_price,
    validate_value, validate_value_size,
};
use crate::hyperliquid::{
    Exchange, ExchangeResponse, HyperLiquid, Info, Limit, OrderRequest, OrderType, Tif, Trigger,
    TriggerType,
};
use crate::settings::Settings;
use crate::types::{LimitPrice, MarginType, OrderSize, TpSl};

pub async fn cli(config: &Settings) {
    let matches = App::new(env!("CARGO_PKG_NAME"))
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

    let wallet = Arc::new(
        config
            .account
            .private_key
            .expose_secret()
            .parse::<LocalWallet>()
            .expect("Failed to parse private key"),
    );

    let info: Info = HyperLiquid::new(wallet.clone());
    let exchange: Exchange = HyperLiquid::new(wallet.clone());

    let metadata = info.metadata().await.expect("Failed to fetch metadata");

    let assets = metadata
        .universe
        .into_iter()
        .enumerate()
        .map(|(i, asset)| (asset.name.to_uppercase(), (asset.sz_decimals, i as u32)))
        .collect::<HashMap<String, (u32, u32)>>();

    match matches.subcommand() {
        ("tp", Some(matches)) => {
            let sz: OrderSize = matches
                .value_of("size")
                .unwrap()
                .try_into()
                .expect("Failed to parse order size");

            let symbol = matches
                .value_of("asset")
                .unwrap_or(&config.default_asset.value);

            let tp: TpSl = matches
                .value_of("tp")
                .expect("Tp price is required")
                .try_into()
                .expect("Invalid tp price, valid values e.g 10% | +10 | 1900");

            // ----------------------------------------------

            let (sz, entry_price, is_buy) = match sz {
                OrderSize::Percent(sz) => {
                    let state = info
                        .clearing_house_state()
                        .await
                        .expect("Failed to fetch open positions");

                    let order = state.asset_positions.iter().find(|ap| {
                        ap.position.coin.to_uppercase() == symbol.to_uppercase()
                            && ap.position.entry_px.is_some()
                    });

                    let order = match order {
                        Some(order) => order,
                        None => {
                            println!("{}", "-".repeat(35));

                            println!("\nNo open order for {}", symbol);
                            return;
                        }
                    };

                    let (is_buy, order_size) = order.position.szi.split_at(1);

                    let order_size = order_size
                        .parse::<f64>()
                        .expect("Failed to parse order size");

                    // Positive for long, negative for short
                    let is_buy = !is_buy.starts_with("-");

                    (
                        order_size * (sz as f64 / 100.0),
                        order
                            .position
                            .entry_px
                            .as_ref()
                            .expect("Failed to parse entry price")
                            .parse::<f64>()
                            .expect("Failed to parse entry price"),
                        is_buy,
                    )
                }

                _ => {
                    println!("{}", "-".repeat(35));

                    println!("\nOnly % of order to tp is supported for now");
                    return;
                }
            };

            let trigger_price = match tp {
                TpSl::Absolute(value) => entry_price + if is_buy { value } else { -value },
                TpSl::Percent(value) => {
                    entry_price
                        * if is_buy {
                            (100.0 + value as f64) / 100.0
                        } else {
                            (100.0 - value as f64) / 100.0
                        }
                }
                TpSl::Fixed(value) => value,
            };

            let order_type = OrderType::Trigger(Trigger {
                trigger_px: format_limit_price(trigger_price).parse().unwrap(),
                is_market: true,
                tpsl: TriggerType::Tp,
            });

            let (sz_decimals, asset) = *assets
                .get(&symbol.to_uppercase())
                .expect("Failed to find asset");

            let order = OrderRequest {
                asset,
                is_buy: !is_buy,
                limit_px: format_limit_price(trigger_price),
                sz: format_size(sz, sz_decimals),
                reduce_only: true,
                order_type,
            };

            let res = exchange.place_order(order).await;
            match res {
                Ok(order) => match order {
                    ExchangeResponse::Err(err) => println!("{:#?}", err),
                    ExchangeResponse::Ok(_order) => {
                        // println!("Order placed: {:#?}", order);
                        println!("{}", "-".repeat(35));
                        println!("\nTake profit order was successfully placed.\n")
                    }
                },
                Err(err) => println!("{:#?}", err),
            }
        }
        ("sl", Some(_sl_matches)) => {
            // let percentage_order = sl_matches.value_of("percentage_order").unwrap();
            // let percentage_order: f64 = percentage_order
            //     .trim_end_matches("%")
            //     .parse::<f64>()
            //     .unwrap();

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

        ("buy", Some(matches)) => {
            let order_size: OrderSize = matches
                .value_of("order_size")
                .unwrap_or(&config.default_size.value)
                .try_into()
                .expect("Failed to parse order size");

            let symbol = matches
                .value_of("asset")
                .unwrap_or(&config.default_asset.value);

            let limit_price: LimitPrice = matches
                .value_of("limit_price")
                .unwrap_or("@0")
                .try_into()
                .expect("Failed to parse limit price");

            let tp: Option<TpSl> = matches.value_of("tp").map(|price| {
                price.try_into().expect(
                    "Invalid take profit value, expected a number or a percentage value e.g 10%",
                )
            });

            let sl: Option<TpSl> = matches.value_of("sl").map(|price| {
                price.try_into().expect(
                    "Invalid stop loss value, expected a number or a percentage value e.g 10%",
                )
            });

            // ----------------------------------------------

            let asset_ctx = info
                .asset_ctx(symbol)
                .await
                .expect("Failed to fetch asset ctxs")
                .expect("Failed to find asset");

            let market_price = asset_ctx.mark_px.parse::<f64>().unwrap();

            println!("Market price: {}", market_price);

            let slippage = 3.0 / 100.0;

            let (order_type, limit_price) = match limit_price {
                LimitPrice::Absolute(price) => {
                    if price == 0.0 {
                        // slippage of 3% for buy 'll be 103/100 = 1.03
                        (
                            OrderType::Limit(Limit { tif: Tif::Ioc }),
                            market_price * (1.0 + slippage),
                        )
                    } else {
                        (OrderType::Limit(Limit { tif: Tif::Gtc }), price)
                    }
                }
            };

            let sz = match order_size {
                OrderSize::Absolute(sz) => sz,
                OrderSize::Percent(sz) => {
                    let state = info
                        .clearing_house_state()
                        .await
                        .expect("Failed to fetch balance");

                    let balance = match config.default_margin.value {
                        MarginType::Cross => state.cross_margin_summary.account_value,
                        MarginType::Isolated => state.margin_summary.account_value,
                    };

                    let balance = balance.parse::<f64>().expect("Failed to parse balance");

                    balance * (sz as f64 / 100.0)
                }
            };
            let (sz_decimals, asset) = *assets
                .get(&symbol.to_uppercase())
                .expect("Failed to find asset");

            // convert $sz to base asset
            let sz = sz / market_price;

            println!("{}", "---".repeat(20));
            // FIXME: update_leverage(&exchange, &config).await;
            println!("{}", "---".repeat(20));

            let order = OrderRequest {
                asset,
                is_buy: true,
                limit_px: format_limit_price(limit_price),
                sz: format_size(sz, sz_decimals),
                reduce_only: false,
                order_type,
            };

            let limit_price = match &order.order_type {
                OrderType::Limit(Limit { tif: Tif::Ioc }) => market_price,
                _ => limit_price,
            };

            println!(
                "\nPlacing a buy order for {symbol} at {}",
                format_limit_price(limit_price)
            );

            let res = exchange.place_order(order).await;

            match res {
                Ok(order) => match order {
                    ExchangeResponse::Err(err) => {
                        println!("{:#?}", err);
                        return;
                    }
                    ExchangeResponse::Ok(_order) => {
                        // println!("Order placed: {:#?}", order);
                        println!("Buy order was successfully placed.\n")
                    }
                },
                Err(err) => {
                    println!("{:#?}", err);
                    return;
                }
            }

            // tp
            if tp.is_some() {
                let trigger_price = match tp {
                    Some(TpSl::Absolute(value)) => limit_price + value,
                    Some(TpSl::Percent(value)) => limit_price * (100.0 + value as f64) / 100.0,
                    Some(TpSl::Fixed(value)) => value,

                    None => unreachable!("Expected a take profit value"),
                };

                let order_type = OrderType::Trigger(Trigger {
                    trigger_px: format_limit_price(trigger_price).parse().unwrap(),
                    is_market: true,
                    tpsl: TriggerType::Tp,
                });

                let order = OrderRequest {
                    asset,
                    is_buy: false,
                    limit_px: format_limit_price(trigger_price),
                    sz: format_size(sz, sz_decimals),
                    reduce_only: true,
                    order_type,
                };

                println!(
                    "Placing a take profit order for {symbol} at {}",
                    order.limit_px
                );
                let res = exchange.place_order(order).await;
                match res {
                    Ok(order) => match order {
                        ExchangeResponse::Err(err) => println!("{:#?}", err),
                        ExchangeResponse::Ok(_order) => {
                            // println!("Order placed: {:#?}", order);
                            println!("Take profit order was successfully placed.\n")
                        }
                    },
                    Err(err) => println!("{:#?}", err),
                }
            }

            // sl
            if sl.is_some() {
                let trigger_price = match sl {
                    Some(TpSl::Absolute(value)) => limit_price - value,
                    Some(TpSl::Percent(value)) => limit_price * (100.0 - value as f64) / 100.0,
                    Some(TpSl::Fixed(value)) => value,

                    None => unreachable!("Expected a stop loss value"),
                };

                let order_type = OrderType::Trigger(Trigger {
                    trigger_px: format_limit_price(trigger_price).parse().unwrap(),
                    is_market: true,
                    tpsl: TriggerType::Sl,
                });

                let order = OrderRequest {
                    asset,
                    is_buy: false,
                    limit_px: format_limit_price(trigger_price),
                    sz: format_size(sz, sz_decimals),
                    reduce_only: true,
                    order_type,
                };

                println!(
                    "Placing a stop loss order for {symbol} at {}",
                    order.limit_px
                );
                let res = exchange.place_order(order).await;
                match res {
                    Ok(order) => match order {
                        ExchangeResponse::Err(err) => println!("{:#?}", err),
                        ExchangeResponse::Ok(_order) => {
                            // println!("Order placed: {:#?}", order);
                            println!("Stop loss order was successfully placed.\n")
                        }
                    },
                    Err(err) => println!("{:#?}", err),
                }
            }
        }

        ("sell", Some(_sell_matches)) => {
            // let order_size = sell_matches.value_of("order_size");
            // let asset = sell_matches.value_of("asset");
            // let limit_price = sell_matches.value_of("limit_price");
            // let take_profit = sell_matches.value_of("take_profit");
            // let stop_loss = sell_matches.value_of("stop_loss");
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
        ("twap", Some(matches)) => {
            // twap buy <total order size> <asset symbol>  <time between interval in mins, number of intervals>

            match matches.subcommand() {
                ("buy", Some(matches)) => {
                    let order_size = matches
                        .value_of("order_size")
                        .unwrap()
                        .parse::<f64>()
                        .unwrap();
                    let symbol = matches.value_of("asset").unwrap();
                    let intervals: Vec<&str> = matches
                        .value_of("interval")
                        .unwrap()
                        .split(",")
                        .collect();

                    let interval_minutes: f64 =
                        intervals[0].parse().expect("Invalid Interval Value");
                    let interval_range: f64 = intervals[1].parse().expect("Invalid interval Value");

                    let amount_asset = order_size / interval_range;



                    let (sz_decimals, asset) = *assets
                        .get(&symbol.to_uppercase())
                        .expect("Failed to find asset");

                    println!("Amount to buy: {}", amount_asset);


                    //now place this order at intervals of interval_minutes
                    for i in 0..interval_range as i32 {

                        let market_price = info
                        .asset_ctx(&symbol.to_uppercase())
                        .await
                        .expect("Failed to fetch asset ctxs")
                        .expect("Failed to find asset")
                        .mark_px
                        .parse::<f64>()
                        .unwrap();

                        println!("Market price is: {}", market_price);
                        let slippage = 3.0 / 100.0;
                        let limit_price = market_price * (1.0 + slippage);                        

                        let order = OrderRequest {
                            asset,
                            is_buy: true,
                            limit_px: format_limit_price(limit_price),
                            sz: format_size(amount_asset, sz_decimals),
                            reduce_only: false,
                            order_type: OrderType::Limit(Limit { tif: Tif::Ioc }),
                        };

                        println! ("Placing order number: {}", i+1);

                        let res = exchange.place_order(order).await;

                        match res {
                            Ok(order) => match order {
                                ExchangeResponse::Err(err) => {
                                    println!("{:#?}", err);
                                    return;
                                }
                                ExchangeResponse::Ok(_order) => {
                                    // println!("Order placed: {:#?}", order);
                                    println!("Buy order was successfully placed.\n")
                                }
                            },
                            Err(err) => {
                                println!("{:#?}", err);
                                return;
                            }
                        }

                        println!("Sleeping for {} minutes", interval_minutes);
                        thread::sleep(Duration::from_secs(
                            (interval_minutes * 60.0) as u64,
                        ));
                    }


                    
                }
                ("sell", Some(matches)) => {
                    let order_size = matches
                        .value_of("order_size")
                        .unwrap()
                        .parse::<f64>()
                        .unwrap();
                    let symbol = matches.value_of("asset").unwrap();
                    let intervals: Vec<&str> = matches
                        .value_of("interval")
                        .unwrap()
                        .split(",")
                        .collect();

                    let interval_minutes: f64 =
                        intervals[0].parse().expect("Invalid Interval Value");
                    let interval_range: f64 = intervals[1].parse().expect("Invalid interval Value");

                    let amount_asset = order_size / interval_range;

                    let (sz_decimals, asset) = *assets
                        .get(&symbol.to_uppercase())
                        .expect("Failed to find asset");

                    println!("Amount to sell: {}", amount_asset);

                    //now place this order at intervals of interval_minutes

                    for i in 0..interval_range as i32 {

                        let market_price = info
                        .asset_ctx(&symbol.to_uppercase())
                        .await
                        .expect("Failed to fetch asset ctxs")
                        .expect("Failed to find asset")
                        .mark_px
                        .parse::<f64>()
                        .unwrap();

                        println!("Market price is: {}", market_price);
                        let slippage = 3.0 / 100.0;
                        let limit_price = market_price * (1.0 - slippage);                        

                        let order = OrderRequest {
                            asset,
                            is_buy: false,
                            limit_px: format_limit_price(limit_price),
                            sz: format_size(amount_asset, sz_decimals),
                            reduce_only: false,
                            order_type: OrderType::Limit(Limit { tif: Tif::Ioc }),
                        };

                        println! ("Placing order number: {}", i+1);

                        let res = exchange.place_order(order).await;

                        match res {
                            Ok(order) => match order {
                                ExchangeResponse::Err(err) => {
                                    println!("{:#?}", err);
                                    return;
                                }
                                ExchangeResponse::Ok(_order) => {
                                    // println!("Order placed: {:#?}", order);
                                    println!("Sell order was successfully placed.\n")
                                }
                            },
                            Err(err) => {
                                println!("{:#?}", err);
                                return;
                            }
                        }

                        println!("Sleeping for {} minutes", interval_minutes);
                        thread::sleep(Duration::from_secs(
                            (interval_minutes * 60.0) as u64,
                        ));
                    }
                }
                _ => {
                    println!("No matching pattern");
                }
                
            }
        }

        ("view", Some(view_matches)) => match view_matches.subcommand_name() {
            Some("pnl") => {
                let state = info
                    .clearing_house_state()
                    .await
                    .expect("Failed to fetch pnl");

                let open_positions = state
                    .asset_positions
                    .iter()
                    .filter(|ap| ap.position.entry_px.is_some())
                    .collect::<Vec<_>>();

                let total_unrealized_pnl: f64 = open_positions
                    .iter()
                    .map(|ap| ap.position.unrealized_pnl.parse::<f64>().unwrap_or(0.0))
                    .sum();

                println!("Total Unrealized PNL: ${:.4} ", total_unrealized_pnl);
            }

            Some("wallet") => {
                let state = info
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
                let unfilled_orders = info.open_orders().await.unwrap();
                let repeat = 35;
                for order in unfilled_orders.iter() {
                    println!("{}", format!("{}", "_".repeat(repeat)));
                    println!();
                    println!("Asset: {}", order.coin);
                    println!("Limit Price: {}", order.limit_px);
                    println!("Side: {}", if order.side == "B" { "Buy" } else { "Sell" });
                    println!("Size: {} {}", order.sz, order.coin);
                }

                println!("{}", format!("{}", "_".repeat(repeat)));
                println!("\nTotal Unfilled Orders: {}", unfilled_orders.len());
            }
            Some("open") => {
                let state = info.clearing_house_state().await.unwrap();

                let open_positions = state
                    .asset_positions
                    .iter()
                    .filter(|ap| ap.position.entry_px.is_some())
                    .collect::<Vec<_>>();

                let repeat = 35;
                for op in open_positions.iter() {
                    let entry_position = op.position.entry_px.as_ref().unwrap();

                    println!("{}", format!("{}", "_".repeat(repeat)));
                    println!();
                    println!("Asset: {}", op.position.coin);
                    println!("Entry Price: {:#?}", entry_position);
                    println!("Position Size: {}", format!("{}", op.position.szi));
                    println!(
                        "Position Value: {}",
                        format!("${}", op.position.position_value)
                    );
                    println!(
                        "Return on Equity: {}",
                        format!("{}%", op.position.return_on_equity)
                    );
                    println!(
                        "Unrealized Pnl: {}",
                        format!("${}", op.position.unrealized_pnl)
                    );
                }

                println!("{}", format!("{}", "_".repeat(repeat)));
                println!("\nTotal Open Positions: {}", open_positions.len());
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
