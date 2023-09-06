use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

//using version 2.33 not the latest one
use ethers::signers::LocalWallet;
use secrecy::ExposeSecret;

use crate::command::command;
use crate::helpers::{format_price, format_size};
use crate::hyperliquid::{
    Exchange, ExchangeResponse, HyperLiquid, Info, Limit, OrderRequest, OrderStatus, OrderType,
    Tif, Trigger, TriggerType,
};
use crate::settings::Settings;
use crate::types::{LimitPrice, MarginType, OrderSize, Pair, SzPerInterval, TpSl, TwapInterval};

pub async fn startup(config: &Settings) {

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

    match command().get_matches().subcommand() {
        Some(("tp", matches)) => {
            let sz: OrderSize = matches
                .get_one::<String>("size")
                .expect("Order size is required")
                .as_str()
                .try_into()
                .expect("Failed to parse order size");

            let symbol = matches
                .get_one::<String>("asset")
                .unwrap_or(&config.default_asset.value);

            let tp: TpSl = matches
                .get_one::<String>("tp")
                .expect("Tp price is required")
                .as_str()
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
                trigger_px: format_price(trigger_price).parse().unwrap(),
                is_market: true,
                tpsl: TriggerType::Tp,
            });

            let (sz_decimals, asset) = *assets
                .get(&symbol.to_uppercase())
                .expect("Failed to find asset");

            let order = OrderRequest {
                asset,
                is_buy: !is_buy,
                limit_px: format_price(trigger_price),
                sz: format_size(sz, sz_decimals),
                reduce_only: true,
                order_type,
            };

            println!("{}", "---".repeat(20));
            println!("Side: Close Long");
            println!(
                "Size in {}: {}",
                symbol,
                order.sz
            );
            println!(
                "Size in USD: {}",
                format_size(sz * entry_price, sz_decimals)
            );
            println!("Entry price: {}", entry_price);

            match exchange.place_order(order).await {
                Ok(order) => {
                    match order {
                        ExchangeResponse::Err(err) => {
                            println!("{:#?}", err);
                            return;
                        }
                        ExchangeResponse::Ok(order) => {
                            order.data.statuses.iter().for_each(|status| match status {
                            OrderStatus::Filled(order) => {
                                println!("Take profit order {} was successfully filled.\n", order.oid);
                                
                            }
                            OrderStatus::Resting(order) => {
                                println!("Take profit order {} was successfully placed.\n", order.oid);
                                
                            }
                            OrderStatus::Error(msg) => {
                                println!("Take profit order failed with error: {:#?}\n", msg)
                            }
                        });
                        }
                    }
                }
                Err(err) => {
                    println!("{:#?}", err);
                    return;
                }
            }

        
        }
        Some(("sl", matches)) => {
            let sz: OrderSize = matches
                .get_one::<String>("size")
                .expect("Order size is required")
                .as_str()
                .try_into()
                .expect("Failed to parse order size");

            let symbol = matches
                .get_one::<String>("asset")
                .unwrap_or(&config.default_asset.value);

            let sl: TpSl = matches
                .get_one::<String>("sl")
                .expect("Sl price is required")
                .as_str()
                .try_into()
                .expect("Failed to parse stop loss price");

            let (sz, entry_price, is_buy) = match sz {
                OrderSize::Percent(sz) => {
                    let state = info
                        .clearing_house_state()
                        .await
                        .expect("Failed to fetch balance");

                    let order = state.asset_positions.iter().find(|ap| {
                        ap.position.coin.to_uppercase() == symbol.to_uppercase()
                            && ap.position.entry_px.is_some()
                    });

                    let order = match order {
                        Some(order) => order,
                        None => {
                            println!("{}", "-".repeat(35));
                            println!("No open position found for {}", symbol);
                            return;
                        }
                    };

                    let (is_buy, order_size) = order.position.szi.split_at(1);

                    let order_size = order_size
                        .parse::<f64>()
                        .expect("Failed to parse order size");
                    let is_buy = !is_buy.starts_with("-");

                    (
                        order_size * (sz as f64 / 100.0),
                        order
                            .position
                            .entry_px
                            .as_ref()
                            .expect("Failed to find entry price")
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

            let trigger_price = match sl {
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
                trigger_px: format_price(trigger_price).parse().unwrap(),
                is_market: true,
                tpsl: TriggerType::Sl,
            });

            let (sz_decimals, asset) = *assets
                .get(&symbol.to_uppercase())
                .expect("Failed to find asset");

            let order = OrderRequest {
                asset,
                is_buy: is_buy,
                limit_px: format_price(trigger_price),
                sz: format_size(sz, sz_decimals),
                reduce_only: true,
                order_type,
            };

            println!("{}", "---".repeat(20));

            println!("Side: Close Long");
            println!(
                "Size in {}: {}",
                symbol,
                order.sz
            );
            println!(
                "Size in USD: {}",
                format_size(sz * entry_price, sz_decimals)
            );
            println!("Entry price: {}", entry_price);

            match exchange.place_order(order).await {
                Ok(order) => {
                    match order {
                        ExchangeResponse::Err(err) => {
                            println!("{:#?}", err);
                            return;
                        }
                        ExchangeResponse::Ok(order) => {
                            order.data.statuses.iter().for_each(|status| match status {
                            OrderStatus::Filled(order) => {
                                println!("Stop loss order {} was successfully filled.\n", order.oid);
                                
                            }
                            OrderStatus::Resting(order) => {
                                println!("Stop loss order {} was successfully placed.\n", order.oid);
                                
                            }
                            OrderStatus::Error(msg) => {
                                println!("Stop loss order failed with error: {:#?}\n", msg)
                            }
                        });
                        }
                    }
                }
                Err(err) => {
                    println!("{:#?}", err);
                    return;
                }
            }
        }

        Some(("buy", matches)) => {
            let order_size: OrderSize = matches
                .get_one::<String>("size")
                .unwrap_or(&config.default_size.value)
                .as_str()
                .try_into()
                .expect("Failed to parse order size");

            let symbol = matches
                .get_one::<String>("asset")
                .unwrap_or(&config.default_asset.value);

            let limit_price: LimitPrice = matches
                .get_one::<String>("price")
                .unwrap_or(&"@0".to_string())
                .as_str()
                .try_into()
                .expect("Failed to parse limit price");

            let tp: Option<TpSl> = matches.get_one::<String>("tp").map(|price| {
                price.as_str().try_into().expect(
                    "Invalid take profit value, expected a number or a percentage value e.g 10%",
                )
            });

            let sl: Option<TpSl> = matches.get_one::<String>("sl").map(|price| {
                price.as_str().try_into().expect(
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

            // FIXME: update_leverage(&exchange, &config).await;

            let order = OrderRequest {
                asset,
                is_buy: true,
                limit_px: format_price(limit_price),
                sz: format_size(sz, sz_decimals),
                reduce_only: false,
                order_type,
            };

            let limit_price = match &order.order_type {
                OrderType::Limit(Limit { tif: Tif::Ioc }) => market_price,
                _ => limit_price,
            };

        
            println!("{}", "---".repeat(20));
            println!("Side: Buy");
            println!(
                "Size in {}: {}",
                symbol,
                order.sz
            );
            println!(
                "Size in USD: {}",
                format_size(sz * market_price, sz_decimals)
            );
            println!("Market price: {}\n", market_price);

            match exchange.place_order(order).await {
                Ok(order) => {
                    match order {
                        ExchangeResponse::Err(err) => {
                            println!("{:#?}", err);
                            return;
                        }
                        ExchangeResponse::Ok(order) => {
                            order.data.statuses.iter().for_each(|status| match status {
                            OrderStatus::Filled(order) => {
                                println!("Order {} was successfully filled.\n", order.oid);
                                
                            }
                            OrderStatus::Resting(order) => {
                                println!("Order {} was successfully placed.\n", order.oid);
                                
                            }
                            OrderStatus::Error(msg) => {
                                println!("Order failed with error: {:#?}\n", msg)
                            }
                        });
                        }
                    }
                }
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
                    trigger_px: format_price(trigger_price).parse().unwrap(),
                    is_market: true,
                    tpsl: TriggerType::Tp,
                });

                let order = OrderRequest {
                    asset,
                    is_buy: false,
                    limit_px: format_price(trigger_price),
                    sz: format_size(sz, sz_decimals),
                    reduce_only: true,
                    order_type,
                };

                println!("{}", "---".repeat(20));
                println!("Side: Close Long");
                println!(
                    "Size in {}: {}",
                    symbol,
                    order.sz
                );
                println!(
                    "Size in USD: {}",
                    format_size(sz * market_price, sz_decimals)
                );
                println!("Entry price: {}", order.limit_px);
                println!("Market price: {}\n", market_price);
    
                match exchange.place_order(order).await {
                    Ok(order) => {
                        match order {
                            ExchangeResponse::Err(err) => {
                                println!("{:#?}", err);
                                return;
                            }
                            ExchangeResponse::Ok(order) => {
                                order.data.statuses.iter().for_each(|status| match status {
                                OrderStatus::Filled(order) => {
                                    println!("Take profit order {} was successfully filled.\n", order.oid);
                                    
                                }
                                OrderStatus::Resting(order) => {
                                    println!("Take profit order {} was successfully placed.\n", order.oid);
                                    
                                }
                                OrderStatus::Error(msg) => {
                                    println!("Take profit order failed with error: {:#?}\n", msg)
                                }
                            });
                            }
                        }
                    }
                    Err(err) => {
                        println!("{:#?}", err);
                        return;
                    }
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
                    trigger_px: format_price(trigger_price).parse().unwrap(),
                    is_market: true,
                    tpsl: TriggerType::Sl,
                });

                let order = OrderRequest {
                    asset,
                    is_buy: false,
                    limit_px: format_price(trigger_price),
                    sz: format_size(sz, sz_decimals),
                    reduce_only: true,
                    order_type,
                };

                println!("{}", "---".repeat(20));
                println!("Side: Close Long");
                println!(
                    "Size in {}: {}",
                    symbol,
                    order.sz
                );
                println!(
                    "Size in USD: {}",
                    format_size(sz * market_price, sz_decimals)
                );
                println!("Entry price: {}", order.limit_px);
                println!("Market price: {}\n", market_price);

                match exchange.place_order(order).await {
                    Ok(order) => {
                        match order {
                            ExchangeResponse::Err(err) => {
                                println!("{:#?}", err);
                                return;
                            }
                            ExchangeResponse::Ok(order) => {
                                order.data.statuses.iter().for_each(|status| match status {
                                OrderStatus::Filled(order) => {
                                    println!("Stop loss order {} was successfully filled.\n", order.oid);
                                    
                                }
                                OrderStatus::Resting(order) => {
                                    println!("Stop loss order {} was successfully placed.\n", order.oid);
                                    
                                }
                                OrderStatus::Error(msg) => {
                                    println!("Stop loss order failed with error: {:#?}\n", msg)
                                }
                            });
                            }
                        }
                    }
                    Err(err) => {
                        println!("{:#?}", err);
                        return;
                    }
                }

            }
        }

        Some(("sell", matches)) => {
            let order_size: OrderSize = matches
                .get_one::<String>("size")
                .unwrap_or(&config.default_size.value)
                .as_str()
                .try_into()
                .expect("Failed to parse order size");

            let symbol = matches
                .get_one::<String>("asset")
                .unwrap_or(&config.default_asset.value);

            let limit_price: LimitPrice = matches
                .get_one::<String>("price")
                .unwrap_or(&"@0".to_string())
                .as_str()
                .try_into()
                .expect("Failed to parse limit price");

            let tp: Option<TpSl> = matches.get_one::<String>("tp").map(|price| {
                price.as_str().try_into().expect(
                    "Invalid take profit value, expected a number or a percentage value e.g 10%",
                )
            });

            let sl: Option<TpSl> = matches.get_one::<String>("sl").map(|price| {
                price.as_str().try_into().expect(
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

            let slippage = 3.0 / 100.0;

            let (order_type, limit_price) = match limit_price {
                LimitPrice::Absolute(price) => {
                    if price == 0.0 {
                        // slippage of 3% for buy 'll be 103/100 = 1.03
                        (
                            OrderType::Limit(Limit { tif: Tif::Ioc }),
                            market_price * (1.0 - slippage),
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

            // Update leverage

            let order = OrderRequest {
                asset,
                is_buy: false,
                limit_px: format_price(limit_price),
                sz: format_size(sz, sz_decimals),
                reduce_only: false,
                order_type,
            };

            let limit_price = match &order.order_type {
                OrderType::Limit(Limit { tif: Tif::Ioc }) => market_price,
                _ => limit_price,
            };

            println!("{}", "---".repeat(20));
            println!("Side: Sell");
            println!(
                "Size in {}: {}",
                symbol,
                order.sz
            );
            println!(
                "Size in USD: {}",
                format_size(sz * market_price, sz_decimals)
            );
            println!("Market price: {}\n", market_price);

            match exchange.place_order(order).await {
                Ok(order) => {
                    match order {
                        ExchangeResponse::Err(err) => {
                            println!("{:#?}", err);
                            return;
                        }
                        ExchangeResponse::Ok(order) => {
                            order.data.statuses.iter().for_each(|status| match status {
                            OrderStatus::Filled(order) => {
                                println!("Order {} was successfully filled.\n", order.oid);
                                
                            }
                            OrderStatus::Resting(order) => {
                                println!("Order {} was successfully placed.\n", order.oid);
                                
                            }
                            OrderStatus::Error(msg) => {
                                println!("Order failed with error: {:#?}\n", msg)
                            }
                        });
                        }
                    }
                }
                Err(err) => {
                    println!("{:#?}", err);
                    return;
                }
            }



            if tp.is_some() {
                let trigger_price = match tp {
                    Some(TpSl::Absolute(value)) => limit_price - value,
                    Some(TpSl::Percent(value)) => limit_price * (100.0 - value as f64) / 100.0,
                    Some(TpSl::Fixed(value)) => value,

                    None => unreachable!("Expected a take profit value"),
                };

                let order_type = OrderType::Trigger(Trigger {
                    trigger_px: format_price(trigger_price).parse().unwrap(),
                    is_market: true,
                    tpsl: TriggerType::Tp,
                });

                let order = OrderRequest {
                    asset,
                    is_buy: true,
                    limit_px: format_price(trigger_price),
                    sz: format_size(sz, sz_decimals),
                    reduce_only: true,
                    order_type,
                };

                println!("{}", "---".repeat(20));
                println!("Side: Close Short");
                println!(
                    "Size in {}: {}",
                    symbol,
                    order.sz
                );
                println!(
                    "Size in USD: {}",
                    format_size(sz * market_price, sz_decimals)
                );
                println!("Entry price: {}", order.limit_px);
                println!("Market price: {}\n", market_price);

                match exchange.place_order(order).await {
                    Ok(order) => {
                        match order {
                            ExchangeResponse::Err(err) => {
                                println!("{:#?}", err);
                                return;
                            }
                            ExchangeResponse::Ok(order) => {
                                order.data.statuses.iter().for_each(|status| match status {
                                OrderStatus::Filled(order) => {
                                    println!("Take profit order {} was successfully filled.\n", order.oid);
                                    
                                }
                                OrderStatus::Resting(order) => {
                                    println!("Take profit order {} was successfully placed.\n", order.oid);
                                    
                                }
                                OrderStatus::Error(msg) => {
                                    println!("Take profit order failed with error: {:#?}\n", msg)
                                }
                            });
                            }
                        }
                    }
                    Err(err) => {
                        println!("{:#?}", err);
                        return;
                    }
                }
                
            }

            if sl.is_some() {
                let trigger_price = match sl {
                    Some(TpSl::Absolute(value)) => limit_price + value,
                    Some(TpSl::Percent(value)) => limit_price * (100.0 + value as f64) / 100.0,
                    Some(TpSl::Fixed(value)) => value,

                    None => unreachable!("Expected a stop loss value"),
                };

                let order_type = OrderType::Trigger(Trigger {
                    trigger_px: format_price(trigger_price).parse().unwrap(),
                    is_market: true,
                    tpsl: TriggerType::Sl,
                });

                let order = OrderRequest {
                    asset,
                    is_buy: true,
                    limit_px: format_price(trigger_price),
                    sz: format_size(sz, sz_decimals),
                    reduce_only: true,
                    order_type,
                };

                println!("{}", "---".repeat(20));
                println!("Side: Close Short");
                println!(
                    "Size in {}: {}",
                    symbol,
                    order.sz
                );
                println!(
                    "Size in USD: {}",
                    format_size(sz * market_price, sz_decimals)
                );
                println!("Entry price: {}", order.limit_px);
                println!("Market price: {}\n", market_price);

                match exchange.place_order(order).await {
                    Ok(order) => {
                        match order {
                            ExchangeResponse::Err(err) => {
                                println!("{:#?}", err);
                                return;
                            }
                            ExchangeResponse::Ok(order) => {
                                order.data.statuses.iter().for_each(|status| match status {
                                OrderStatus::Filled(order) => {
                                    println!("Stop loss order {} was successfully filled.\n", order.oid);
                                    
                                }
                                OrderStatus::Resting(order) => {
                                    println!("Stop loss order {} was successfully placed.\n", order.oid);
                                    
                                }
                                OrderStatus::Error(msg) => {
                                    println!("Stop loss order failed with error: {:#?}\n", msg)
                                }
                            });
                            }
                        }
                    }
                    Err(err) => {
                        println!("{:#?}", err);
                        return;
                    }
                }   
            }
        }

        Some(("scale", matches)) => match matches.subcommand() {
            Some(("buy", matches)) => {
                let sz_per_interval: SzPerInterval = matches
                    .get_one::<String>("size_per_interval")
                    .expect("Order size is required")
                    .as_str()
                    .try_into()
                    .expect("Failed to parse order size");

                let symbol = matches.get_one::<String>("asset").expect("Asset is required");

                let lower = matches
                    .get_one::<String>("lower")
                    .expect("Lower price bracket is required")
                    .parse::<f64>()
                    .expect("Failed to parse lower price bracket");

                let upper = matches
                    .get_one::<String>("upper")
                    .expect("Upper price bracket is required")
                    .parse::<f64>()
                    .expect("Failed to parse upper price bracket");

                // ----------------------------------------------

                let asset_ctx = info
                    .asset_ctx(symbol)
                    .await
                    .expect("Failed to fetch asset ctxs")
                    .expect("Failed to find asset");

                let market_price = asset_ctx.mark_px.parse::<f64>().unwrap();

                let (sz_decimals, asset) = *assets
                    .get(&symbol.to_uppercase())
                    .expect("Failed to find asset");

                let interval = (upper - lower) / (sz_per_interval.interval - 1) as f64;

                let sz = (sz_per_interval.size / sz_per_interval.interval as f64) / market_price;

                for i in 0..sz_per_interval.interval {
                    let limit_price = lower + (interval * i as f64);

                    println!("{}", "---".repeat(20));
                    println!("Order {} of {}", i + 1, sz_per_interval.interval);
                    println!("Side: Buy");
                    println!("Size in {symbol}: {}", format_size(sz, sz_decimals));
                    println!(
                        "Size in USD: {}",
                        format_size(sz * market_price, sz_decimals)
                    );
                    println!("Entry price: {}", format_price(limit_price));
                    println!("Market price: {}\n", market_price);

                    let order = OrderRequest {
                        asset,
                        is_buy: true,
                        limit_px: format_price(limit_price),
                        sz: format_size(sz, sz_decimals),
                        reduce_only: false,
                        order_type: OrderType::Limit(Limit { tif: Tif::Gtc }),
                    };

                    match exchange.place_order(order).await {
                        Ok(order) => match order {
                            ExchangeResponse::Err(err) => {
                                println!("{:#?}", err);
                                return;
                            }
                            ExchangeResponse::Ok(order) => {
                                order.data.statuses.iter().for_each(|status| match status {
                                    OrderStatus::Filled(order) => {
                                        println!("Order {} was successfully filled.\n", order.oid)
                                    }
                                    OrderStatus::Resting(order) => {
                                        println!("Order {} was successfully placed.\n", order.oid)
                                    }
                                    OrderStatus::Error(msg) => {
                                        println!("Order failed with error: {:#?}\n", msg)
                                    }
                                });
                            }
                        },
                        Err(err) => {
                            println!("{:#?}", err);
                            return;
                        }
                    }
                }
            }

            Some(("sell", matches)) => {
                let sz_per_interval: SzPerInterval = matches
                    .get_one::<String>("size_per_interval")
                    .expect("Order size is required")
                    .as_str()
                    .try_into()
                    .expect("Failed to parse order size");

                let symbol = matches.get_one::<String>("asset").expect("Asset is required");
                let lower = matches
                    .get_one::<String>("lower")
                    .expect("Lower price bracket is required")
                    .as_str()
                    .parse::<f64>()
                    .expect("Failed to parse lower price bracket");
                let upper = matches
                    .get_one::<String>("upper")
                    .expect("Upper price bracket is required")
                    .as_str()
                    .parse::<f64>()
                    .expect("Failed to parse upper price bracket");
                //------------------------------------

                let asset_ctx = info
                    .asset_ctx(symbol)
                    .await
                    .expect("Failed to fetch asset ctxs")
                    .expect("Failed to find asset");
                let market_price = asset_ctx.mark_px.parse::<f64>().unwrap();

                let (sz_decimals, asset) = *assets
                    .get(&symbol.to_uppercase())
                    .expect("Failed to find asset");

                let interval = (upper - lower) / (sz_per_interval.interval - 1) as f64;

                let sz = (sz_per_interval.size / sz_per_interval.interval as f64) / market_price;

                for i in 0..sz_per_interval.interval {
                    let limit_price = lower + (interval * i as f64);

                    println!("{}", "---".repeat(20));
                    println!("Order {} of {}", i + 1, sz_per_interval.interval);
                    println!("Side: Sell");
                    println!("Size in {symbol}: {}", format_size(sz, sz_decimals));
                    println!(
                        "Size in USD: {}",
                        format_size(sz * market_price, sz_decimals)
                    );
                    println!("Entry price: {}", format_price(limit_price));
                    println!("Market price: {}\n", market_price);

                    let order = OrderRequest {
                        asset,
                        is_buy: false,
                        limit_px: format_price(limit_price),
                        sz: format_size(sz, sz_decimals),
                        reduce_only: false,
                        order_type: OrderType::Limit(Limit { tif: Tif::Gtc }),
                    };

                    match exchange.place_order(order).await {
                        Ok(order) => match order {
                            ExchangeResponse::Err(err) => {
                                println!("{:#?}", err);
                                return;
                            }
                            ExchangeResponse::Ok(order) => {
                                order.data.statuses.iter().for_each(|status| match status {
                                    OrderStatus::Filled(order) => {
                                        println!("Order {} was successfully filled.\n", order.oid)
                                    }
                                    OrderStatus::Resting(order) => {
                                        println!("Order {} was successfully placed.\n", order.oid)
                                    }
                                    OrderStatus::Error(msg) => {
                                        println!("Order failed with error: {:#?}\n", msg)
                                    }
                                });
                            }
                        },
                        Err(err) => {
                            println!("{:#?}", err);
                            return;
                        }
                    }
                }
            }
            _ => {
                println!("No matching pattern");
            }
        },
        Some(("twap", matches)) => {
            match matches.subcommand() {
                Some(("buy", matches)) => {
                    let sz: OrderSize = matches
                        .get_one::<String>("size")
                        .expect("Size is required")
                        .as_str()
                        .try_into()
                        .expect("Failed to parse order size");

                    let symbol = matches.get_one::<String>("asset").expect("Asset is required");

                    let interval: TwapInterval = matches.get_one::<String>("interval")
                    .expect("Interval is required")
                    .as_str()
                    .try_into().expect(
                        "Invalid interval value, correct format is <time between interval in mins, number of intervals> e.g 5,10",
                    );

                    let sz = match sz {
                        OrderSize::Absolute(sz) => sz,

                        _ => {
                            println!("{}", "-".repeat(35));

                            println!("\nOnly absolute order size is supported for now");
                            return;
                        }
                    } / interval.num_of_orders as f64;

                    let (sz_decimals, asset) = *assets
                        .get(&symbol.to_uppercase())
                        .expect("Failed to find asset");

                    let slippage = 3.0 / 100.0;

                    for i in 1..=interval.num_of_orders {
                        let market_price = info
                            .asset_ctx(&symbol.to_uppercase())
                            .await
                            .expect("Failed to fetch asset ctxs")
                            .expect("Failed to find asset")
                            .mark_px
                            .parse::<f64>()
                            .unwrap();

                        let sz = sz / market_price;
                        let limit_price = market_price * (1.0 + slippage);

                        println!("{}", "---".repeat(20));
                        println!("Order {} of {}", i, interval.num_of_orders);
                        println!("Side: Buy");
                        println!("Size in {symbol}: {}", format_size(sz, sz_decimals));
                        println!(
                            "Size in USD: {}",
                            format_size(sz * market_price, sz_decimals)
                        );
                        println!("Market price: {}\n", market_price);

                        let order = OrderRequest {
                            asset,
                            is_buy: true,
                            limit_px: format_price(limit_price),
                            sz: format_size(sz, sz_decimals),
                            reduce_only: false,
                            order_type: OrderType::Limit(Limit { tif: Tif::Ioc }),
                        };

                        match exchange.place_order(order).await {
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

                        if i != interval.num_of_orders {
                            println!("Waiting for {} minutes", interval.interval.as_secs() / 60);
                            println!("{}", "-".repeat(5));
                            tokio::time::sleep(interval.interval).await;
                        }
                    }
                }
                Some(("sell", matches)) => {
                    let sz: OrderSize = matches
                        .get_one::<String>("size")
                        .expect("Size is required")
                        .as_str()
                        .try_into()
                        .expect("Failed to parse order size");

                    let symbol = matches.get_one::<String>("asset").expect("Asset is required");

                    let interval: TwapInterval = matches.get_one::<String>("interval")
                    .expect("Interval is required")
                    .as_str()
                    .try_into().expect(
                        "Invalid interval value, correct format is <time between interval in mins, number of intervals> e.g 5,10",
                    );

                    let sz = match sz {
                        OrderSize::Absolute(sz) => sz,

                        _ => {
                            println!("{}", "-".repeat(35));

                            println!("\nOnly absolute order size is supported for now");
                            return;
                        }
                    } / interval.num_of_orders as f64;

                    let (sz_decimals, asset) = *assets
                        .get(&symbol.to_uppercase())
                        .expect("Failed to find asset");

                    let slippage = 3.0 / 100.0;

                    for i in 1..=interval.num_of_orders {
                        let market_price = info
                            .asset_ctx(&symbol.to_uppercase())
                            .await
                            .expect("Failed to fetch asset ctxs")
                            .expect("Failed to find asset")
                            .mark_px
                            .parse::<f64>()
                            .unwrap();

                        let sz = sz / market_price;
                        let limit_price = market_price * (1.0 - slippage);

                        println!("{}", "---".repeat(20));
                        println!("Order {} of {}", i, interval.num_of_orders);
                        println!("Side: Sell");
                        println!("Size in {symbol}: {}", format_size(sz, sz_decimals));
                        println!(
                            "Size in USD: {}",
                            format_size(sz * market_price, sz_decimals)
                        );
                        println!("Market price: {}\n", market_price);

                        let order = OrderRequest {
                            asset,
                            is_buy: false,
                            limit_px: format_price(limit_price),
                            sz: format_size(sz, sz_decimals),
                            reduce_only: false,
                            order_type: OrderType::Limit(Limit { tif: Tif::Ioc }),
                        };

                        match exchange.place_order(order).await {
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

                        if i != interval.num_of_orders {
                            println!("Waiting for {} minutes", interval.interval.as_secs() / 60);
                            println!("{}", "-".repeat(5));
                            tokio::time::sleep(interval.interval).await;
                        }
                    }
                }
                _ => {
                    println!("No matching pattern");
                }
            }
        }

        Some(("view", matches)) => match matches.subcommand_name() {
            Some("upnl") => {
                let state = info
                    .clearing_house_state()
                    .await
                    .expect("Failed to fetch unrealized pnl");

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
                        " Invalid command: expected commands: (view upnl, view wallet balance, view unfilled orders, view open positions"
                    );
            }
        },

        Some(("pair", matches)) => match matches.subcommand() {
            Some(("buy", matches)) => {
                let sz: f64 = match matches
                    .get_one::<String>("size")
                    .expect("Order size required")
                    .as_str()
                    .try_into()
                    .expect("Failed to parse order size")
                {
                    OrderSize::Absolute(sz) => sz,
                    _ => {
                        println!("{}", "-".repeat(35));

                        println!("\nOnly absolute order size is supported for now");
                        return;
                    }
                };

                let pair: Pair = matches
                    .get_one::<String>("pair")
                    .expect("Pair is required")
                    .as_str()
                    .try_into()
                    .expect("Failed to parse pair");

                let limit_price: LimitPrice = matches
                    .get_one::<String>("price")
                    .unwrap_or(&"@0".to_string())
                    .as_str()
                    .try_into()
                    .expect("Failed to parse limit price");


                let tp: Option<f64> = matches.get_one::<String>("tp").map(|price| {
                        price.parse::<f64>().expect(
                            "Invalid take profit value, expected a number or a percentage value e.g 10%",
                        )
                    });

                let sl: Option<f64> = matches.get_one::<String>("sl").map(|price| {
                    price.parse::<f64>().expect(
                        "Invalid stop loss value, expected a number or a percentage value e.g 10%",
                    )
                });

                // ----------------------------------------------
                let slippage = 3.0 / 100.0;

                let base_sz = sz / 2.0;
                let quote_sz = sz / 2.0;

                let (base_sz_decimals, base_asset) = *assets
                    .get(&pair.base.to_uppercase())
                    .expect(&format!("Failed to find base asset:  {}", pair.base));

                let (quote_sz_decimals, quote_asset) = *assets
                    .get(&pair.quote.to_uppercase())
                    .expect(&format!("Failed to find quote asset:  {}", pair.quote));

                match limit_price {
                    LimitPrice::Absolute(target) => {
                        if target == 0.0 {
                            // Takes 50% of order size and longs Asset X and
                            {
                                let asset_ctx = info
                                    .asset_ctx(&pair.base)
                                    .await
                                    .expect("Failed to fetch asset ctxs")
                                    .expect(&format!("Failed to find base asset:  {}", pair.base));

                                let market_price = asset_ctx.mark_px.parse::<f64>().unwrap();
                                let limit_price = market_price * (1.0 + slippage);

                                let sz = base_sz / market_price;

                                let order = OrderRequest {
                                    asset: base_asset,
                                    is_buy: true,
                                    limit_px: format_price(limit_price),
                                    sz: format_size(sz, base_sz_decimals),
                                    reduce_only: false,
                                    order_type: OrderType::Limit(Limit { tif: Tif::Ioc }),
                                };

                                println!("{}", "---".repeat(20));
                                println!("Order 1 of 2");
                                println!("Side: Buy");
                                println!(
                                    "Size in {}: {}",
                                    pair.base,
                                    format_size(sz, base_sz_decimals)
                                );
                                println!("Size in USD: {}", format_size(base_sz, base_sz_decimals));
                                println!("Market price: {}\n", market_price);

                                match exchange.place_order(order).await {
                                    Ok(order) => {
                                        match order {
                                            ExchangeResponse::Err(err) => {
                                                println!("{:#?}", err);
                                                return;
                                            }
                                            ExchangeResponse::Ok(order) => {
                                                order.data.statuses.iter().for_each(|status| match status {
                                                OrderStatus::Filled(order) => {
                                                    println!("Order {} was successfully filled.\n", order.oid);
                                                    
                                                }
                                                OrderStatus::Resting(order) => {
                                                    println!("Order {} was successfully placed.\n", order.oid);
                                                    
                                                }
                                                OrderStatus::Error(msg) => {
                                                    println!("Order failed with error: {:#?}\n", msg)
                                                }
                                            });
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        println!("{:#?}", err);
                                        return;
                                    }
                                }
                            }

                            // takes another 50% of order size and shorts Asset Y
                            {
                                let asset_ctx = info
                                    .asset_ctx(&pair.quote)
                                    .await
                                    .expect("Failed to fetch asset ctxs")
                                    .expect(&format!(
                                        "Failed to find quote asset:  {}",
                                        pair.quote
                                    ));

                                let market_price = asset_ctx.mark_px.parse::<f64>().unwrap();
                                let limit_price = market_price * (1.0 - slippage);

                                let sz = quote_sz / market_price;

                                let order = OrderRequest {
                                    asset: quote_asset,
                                    is_buy: false,
                                    limit_px: format_price(limit_price),
                                    sz: format_size(sz, quote_sz_decimals),
                                    reduce_only: false,
                                    order_type: OrderType::Limit(Limit { tif: Tif::Ioc }),
                                };

                                println!("{}", "---".repeat(20));
                                println!("Order 2 of 2");
                                println!("Side: Sell");
                                println!(
                                    "Size in {}: {}",
                                    pair.quote,
                                    format_size(sz, quote_sz_decimals)
                                );
                                println!(
                                    "Size in USD: {}",
                                    format_size(quote_sz, quote_sz_decimals)
                                );
                                println!("Market price: {}\n", market_price);

                                match exchange.place_order(order).await {
                                    Ok(order) => {
                                        match order {
                                            ExchangeResponse::Err(err) => {
                                                println!("{:#?}", err);
                                                return;
                                            }
                                            ExchangeResponse::Ok(order) => {
                                                order.data.statuses.iter().for_each(|status| match status {
                                                OrderStatus::Filled(order) => {
                                                    println!("Order {} was successfully filled.\n", order.oid);
                                                    
                                                }
                                                OrderStatus::Resting(order) => {
                                                    println!("Order {} was successfully placed.\n", order.oid);
                                                    
                                                }
                                                OrderStatus::Error(msg) => {
                                                    println!("Order failed with error: {:#?}\n", msg)
                                                }
                                            });
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        println!("{:#?}", err);
                                        return;
                                    }
                                }
                            }
                        } else {
                            // If limit price for eth/btc is .06, wait for the eth/btc ratio to become .06,
                            // then long eth and short btc at market
                            let (
                                base_sz,
                                base_market_price,
                                quote_sz,
                                quote_market_price,
                                current_ratio,
                            ) = loop {
                                let base_limit_price = {
                                    let base_asset_ctx = info
                                        .asset_ctx(&pair.base)
                                        .await
                                        .expect("Failed to fetch asset ctxs")
                                        .expect(&format!(
                                            "Failed to find quote asset:  {}",
                                            pair.quote
                                        ));
                                    base_asset_ctx.mark_px.parse::<f64>().unwrap()
                                };

                                let quote_market_price = {
                                    let quote_asset_ctx = info
                                        .asset_ctx(&pair.quote)
                                        .await
                                        .expect("Failed to fetch asset ctxs")
                                        .expect(&format!(
                                            "Failed to find quote asset:  {}",
                                            pair.quote
                                        ));

                                    quote_asset_ctx.mark_px.parse::<f64>().unwrap()
                                };

                                let current_ratio =
                                    format!("{:.2}", base_limit_price / quote_market_price)
                                        .parse::<f64>()
                                        .unwrap();

                                if current_ratio >= target {
                                    println!("Ratio reached: {} >= {}", current_ratio, target);
                                    let base_sz = base_sz / base_limit_price;
                                    let quote_sz = quote_sz / quote_market_price;

                                    break (
                                        base_sz,
                                        base_limit_price,
                                        quote_sz,
                                        quote_market_price,
                                        current_ratio,
                                    );
                                }

                                println!(
                                    "Current Ratio: {}, Target Ratio: {}, Diff: {}. Checking again in 5 seconds\n---",
                                    format!("{:.2}", current_ratio).parse::<f64>().unwrap(),
                                    format!("{:.2}", target).parse::<f64>().unwrap(),
                                    format!("{:.2}", current_ratio - target).parse::<f64>().unwrap(),
                                );
                                tokio::time::sleep(Duration::from_secs(5)).await;
                            };

                            // send buy order request
                            {
                                let order = OrderRequest {
                                    asset: base_asset,
                                    is_buy: true,
                                    limit_px: format_price(
                                        base_market_price * (1.0 + slippage),
                                    ),
                                    sz: format_size(base_sz, base_sz_decimals),
                                    reduce_only: false,
                                    order_type: OrderType::Limit(Limit { tif: Tif::Ioc }),
                                };

                                println!("{}", "---".repeat(20));
                                println!("Order 1 of 2");
                                println!("Side: Buy");
                                println!(
                                    "Size in {}: {}",
                                    pair.base,
                                    format_size(base_sz, base_sz_decimals)
                                );
                                println!(
                                    "Size in USD: {}",
                                    format_size(base_sz * base_market_price, base_sz_decimals)
                                );
                                println!("Market price: {}\n", base_market_price);
                                println!("Ratio: {}\n", current_ratio);

                                match exchange.place_order(order).await {
                                    Ok(order) => {
                                        match order {
                                            ExchangeResponse::Err(err) => {
                                                println!("{:#?}", err);
                                                return;
                                            }
                                            ExchangeResponse::Ok(order) => {
                                                order.data.statuses.iter().for_each(|status| match status {
                                                OrderStatus::Filled(order) => {
                                                    println!("Order {} was successfully filled.\n", order.oid);
                                                    
                                                }
                                                OrderStatus::Resting(order) => {
                                                    println!("Order {} was successfully placed.\n", order.oid);
                                                    
                                                }
                                                OrderStatus::Error(msg) => {
                                                    println!("Order failed with error: {:#?}\n", msg)
                                                }
                                            });
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        println!("{:#?}", err);
                                        return;
                                    }
                                }
                            }
                            // send sell order request
                            {
                                let order = OrderRequest {
                                    asset: quote_asset,
                                    is_buy: false,
                                    limit_px: format_price(
                                        quote_market_price * (1.0 - slippage),
                                    ),
                                    sz: format_size(quote_sz, quote_sz_decimals),
                                    reduce_only: false,
                                    order_type: OrderType::Limit(Limit { tif: Tif::Ioc }),
                                };

                                println!("{}", "---".repeat(20));
                                println!("Order 2 of 2");
                                println!("Side: Sell");
                                println!(
                                    "Size in {}: {}",
                                    pair.quote,
                                    format_size(quote_sz, quote_sz_decimals)
                                );
                                println!(
                                    "Size in USD: {}",
                                    format_size(quote_sz * quote_market_price, quote_sz_decimals)
                                );
                                println!("Market price: {}\n", quote_market_price);
                                println!("Ratio: {}\n", current_ratio);

                                match exchange.place_order(order).await {
                                    Ok(order) => {
                                        match order {
                                            ExchangeResponse::Err(err) => {
                                                println!("{:#?}", err);
                                                return;
                                            }
                                            ExchangeResponse::Ok(order) => {
                                                order.data.statuses.iter().for_each(|status| match status {
                                                OrderStatus::Filled(order) => {
                                                    println!("Order {} was successfully filled.\n", order.oid);
                                                    
                                                }
                                                OrderStatus::Resting(order) => {
                                                    println!("Order {} was successfully placed.\n", order.oid);
                                                    
                                                }
                                                OrderStatus::Error(msg) => {
                                                    println!("Order failed with error: {:#?}\n", msg)
                                                }
                                            });
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        println!("{:#?}", err);
                                        return;
                                    }
                                }
                            }
                        }

                        if tp.is_none() && sl.is_none() {
                            return;
                        };

                        println!("Monitoring positions for tp or sl\n---");

                        let (exit_long_order, exit_short_order, current_ratio) = loop {
                            let base_market_price = {
                                let base_asset_ctx = info
                                    .asset_ctx(&pair.base)
                                    .await
                                    .expect("Failed to fetch asset ctxs")
                                    .expect(&format!(
                                        "Failed to find quote asset:  {}",
                                        pair.quote
                                    ));
                                base_asset_ctx.mark_px.parse::<f64>().unwrap()
                            };

                            let quote_market_price = {
                                let quote_asset_ctx = info
                                    .asset_ctx(&pair.quote)
                                    .await
                                    .expect("Failed to fetch asset ctxs")
                                    .expect(&format!(
                                        "Failed to find quote asset:  {}",
                                        pair.quote
                                    ));

                                quote_asset_ctx.mark_px.parse::<f64>().unwrap()
                            };

                            let current_ratio =
                                format!("{:.2}", base_market_price / quote_market_price)
                                    .parse::<f64>()
                                    .unwrap();

                            // check if tp or sl has been reached
                            if let Some(tp) = tp {
                                if current_ratio >= tp {
                                    println!("Take profit reached: {} >= {}", current_ratio, tp);

                                    let exit_long_order = OrderRequest {
                                        asset: base_asset,
                                        is_buy: false,
                                        limit_px: format_price(
                                            base_market_price * (1.0 - slippage),
                                        ),
                                        sz: format_size(base_sz, base_sz_decimals),
                                        reduce_only: true,
                                        order_type: OrderType::Limit(Limit { tif: Tif::Ioc }),
                                    };

                                    let exit_short_order = OrderRequest {
                                        asset: quote_asset,
                                        is_buy: true,
                                        limit_px: format_price(
                                            quote_market_price * (1.0 + slippage),
                                        ),
                                        sz: format_size(quote_sz, quote_sz_decimals),
                                        reduce_only: true,
                                        order_type: OrderType::Limit(Limit { tif: Tif::Ioc }),
                                    };

                                    break (exit_long_order, exit_short_order, current_ratio);
                                }
                            }

                            if let Some(sl) = sl {
                                if current_ratio <= sl {
                                    println!("Stop loss reached: {} <= {}", current_ratio, sl);

                                    let exit_long_order = OrderRequest {
                                        asset: base_asset,
                                        is_buy: false,
                                        limit_px: format_price(
                                            base_market_price * (1.0 - slippage),
                                        ),
                                        sz: format_size(base_sz, base_sz_decimals),
                                        reduce_only: true,
                                        order_type: OrderType::Limit(Limit { tif: Tif::Ioc }),
                                    };

                                    let exit_short_order = OrderRequest {
                                        asset: quote_asset,
                                        is_buy: true,
                                        limit_px: format_price(
                                            quote_market_price * (1.0 + slippage),
                                        ),
                                        sz: format_size(quote_sz, quote_sz_decimals),
                                        reduce_only: true,
                                        order_type: OrderType::Limit(Limit { tif: Tif::Ioc }),
                                    };

                                    break (exit_long_order, exit_short_order, current_ratio);
                                }
                            }

                            println!(
                                "Current Ratio: {}, Target Ratio Tp: {}, Target Ratio Sl: {}, Tp Diff: {}, Sl Diff: {}. Checking again in 5 seconds\n---",
                                format!("{:.2}", current_ratio).parse::<f64>().unwrap(),
                                format!("{:.2}", tp.unwrap_or(0.0)).parse::<f64>().unwrap(),
                                format!("{:.2}", sl.unwrap_or(0.0)).parse::<f64>().unwrap(),
                                format!("{:.2}", current_ratio - tp.unwrap_or(0.0)).parse::<f64>().unwrap(),
                                format!("{:.2}", current_ratio - sl.unwrap_or(0.0)).parse::<f64>().unwrap(),
                            );

                            tokio::time::sleep(Duration::from_secs(5)).await;
                        };

                        // place exit orders
                        println!("{}", "---".repeat(20));
                        println!("Order 1 of 2");
                        println!("Side: Sell");
                        println!(
                            "Size in {}: {}",
                            pair.base,
                            format_size(base_sz, base_sz_decimals)
                        );
                        println!("Ratio: {}\n", current_ratio);

                        match exchange.place_order(exit_long_order).await {
                            Ok(order) => match order {
                                ExchangeResponse::Err(err) => {
                                    println!("{:#?}", err);
                                    return;
                                }
                                ExchangeResponse::Ok(order) => {
                                    order.data.statuses.iter().for_each(|status| match status {
                                        OrderStatus::Filled(order) => {
                                            println!(
                                                "Order {} was successfully filled.\n",
                                                order.oid
                                            );
                                        }
                                        OrderStatus::Resting(order) => {
                                            println!(
                                                "Order {} was successfully placed.\n",
                                                order.oid
                                            );
                                        }
                                        OrderStatus::Error(msg) => {
                                            println!("Order failed with error: {:#?}\n", msg)
                                        }
                                    });
                                }
                            },
                            Err(err) => {
                                println!("{:#?}", err);
                                return;
                            }
                        }

                        println!("{}", "---".repeat(20));
                        println!("Order 2 of 2");
                        println!("Side: Buy");
                        println!(
                            "Size in {}: {}",
                            pair.quote,
                            format_size(quote_sz, quote_sz_decimals)
                        );
                        println!("Ratio: {}\n", current_ratio);

                        match exchange.place_order(exit_short_order).await {
                            Ok(order) => match order {
                                ExchangeResponse::Err(err) => {
                                    println!("{:#?}", err);
                                    return;
                                }
                                ExchangeResponse::Ok(order) => {
                                    order.data.statuses.iter().for_each(|status| match status {
                                        OrderStatus::Filled(order) => {
                                            println!(
                                                "Order {} was successfully filled.\n",
                                                order.oid
                                            );
                                        }
                                        OrderStatus::Resting(order) => {
                                            println!(
                                                "Order {} was successfully placed.\n",
                                                order.oid
                                            );
                                        }
                                        OrderStatus::Error(msg) => {
                                            println!("Order failed with error: {:#?}\n", msg)
                                        }
                                    });
                                }
                            },
                            Err(err) => {
                                println!("{:#?}", err);
                                return;
                            }
                        }
                    }
                }
            }
            Some(("sell", matches)) => {
                let sz: f64 = match matches
                    .get_one::<String>("size")
                    .expect("Order size required")
                    .as_str()
                    .try_into()
                    .expect("Failed to parse order size")
                {
                    OrderSize::Absolute(sz) => sz,
                    _ => {
                        println!("{}", "-".repeat(35));

                        println!("\nOnly absolute order size is supported for now");
                        return;
                    }
                };

                let pair: Pair = matches
                    .get_one::<String>("pair")
                    .expect("Pair is required")
                    .as_str()
                    .try_into()
                    .expect("Failed to parse pair");

                let limit_price: LimitPrice = matches
                    .get_one::<String>("price")
                    .unwrap_or(&"@0".to_string())
                    .as_str()
                    .try_into()
                    .expect("Failed to parse limit price");

                let sl: Option<f64> = matches.get_one::<String>("sl").map(|price| {
                    price.parse::<f64>().expect(
                        "Invalid stop loss value, expected a number or a percentage value e.g 10%",
                    )
                });

                let tp: Option<f64> = matches.get_one::<String>("tp").map(|price| {
                    price.parse::<f64>().expect(
                        "Invalid take profit value, expected a number or a percentage value e.g 10%",
                    )
                });

                // ----------------------------------------------
                let slippage = 3.0 / 100.0;

                let base_sz = sz / 2.0;
                let quote_sz = sz / 2.0;

                let (base_sz_decimals, base_asset) = *assets
                    .get(&pair.base.to_uppercase())
                    .expect(&format!("Failed to find base asset:  {}", pair.base));

                let (quote_sz_decimals, quote_asset) = *assets
                    .get(&pair.quote.to_uppercase())
                    .expect(&format!("Failed to find quote asset:  {}", pair.quote));

                match limit_price {
                    LimitPrice::Absolute(target) => {
                        if target == 0.0 {
                            // Takes 50% of order size and shorts Asset X and
                            {
                                let asset_ctx = info
                                    .asset_ctx(&pair.base)
                                    .await
                                    .expect("Failed to fetch asset ctxs")
                                    .expect(&format!("Failed to find base asset:  {}", pair.base));

                                let market_price = asset_ctx.mark_px.parse::<f64>().unwrap();
                                let limit_price = market_price * (1.0 - slippage);

                                let sz = base_sz / market_price;

                                let order = OrderRequest {
                                    asset: base_asset,
                                    is_buy: false,
                                    limit_px: format_price(limit_price),
                                    sz: format_size(sz, base_sz_decimals),
                                    reduce_only: false,
                                    order_type: OrderType::Limit(Limit { tif: Tif::Ioc }),
                                };

                                println!("{}", "---".repeat(20));
                                println!("Order 1 of 2");
                                println!("Side: Sell");
                                println!(
                                    "Size in {}: {}",
                                    pair.base,
                                    format_size(sz, base_sz_decimals)
                                );
                                println!("Size in USD: {}", format_size(base_sz, base_sz_decimals));
                                println!("Market price: {}\n", market_price);

                                match exchange.place_order(order).await {
                                    Ok(order) => {
                                        match order {
                                            ExchangeResponse::Err(err) => {
                                                println!("{:#?}", err);
                                                return;
                                            }
                                            ExchangeResponse::Ok(order) => {
                                                order.data.statuses.iter().for_each(|status| match status {
                                                OrderStatus::Filled(order) => {
                                                    println!("Order {} was successfully filled.\n", order.oid);
                                                    
                                                }
                                                OrderStatus::Resting(order) => {
                                                    println!("Order {} was successfully placed.\n", order.oid);
                                                    
                                                }
                                                OrderStatus::Error(msg) => {
                                                    println!("Order failed with error: {:#?}\n", msg)
                                                }
                                            });
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        println!("{:#?}", err);
                                        return;
                                    }
                                }
                            }

                            // takes another 50% of order size and longs Asset Y
                            {
                                let asset_ctx = info
                                    .asset_ctx(&pair.quote)
                                    .await
                                    .expect("Failed to fetch asset ctxs")
                                    .expect(&format!(
                                        "Failed to find quote asset:  {}",
                                        pair.quote
                                    ));

                                let market_price = asset_ctx.mark_px.parse::<f64>().unwrap();
                                let limit_price = market_price * (1.0 + slippage);

                                let sz = quote_sz / market_price;

                                let order = OrderRequest {
                                    asset: quote_asset,
                                    is_buy: true,
                                    limit_px: format_price(limit_price),
                                    sz: format_size(sz, quote_sz_decimals),
                                    reduce_only: false,
                                    order_type: OrderType::Limit(Limit { tif: Tif::Ioc }),
                                };

                                println!("{}", "---".repeat(20));
                                println!("Order 2 of 2");
                                println!("Side: Buy");
                                println!(
                                    "Size in {}: {}",
                                    pair.quote,
                                    format_size(sz, quote_sz_decimals)
                                );
                                println!(
                                    "Size in USD: {}",
                                    format_size(quote_sz, quote_sz_decimals)
                                );
                                println!("Market price: {}\n", market_price);

                                match exchange.place_order(order).await {
                                    Ok(order) => {
                                        match order {
                                            ExchangeResponse::Err(err) => {
                                                println!("{:#?}", err);
                                                return;
                                            }
                                            ExchangeResponse::Ok(order) => {
                                                order.data.statuses.iter().for_each(|status| match status {
                                                OrderStatus::Filled(order) => {
                                                    println!("Order {} was successfully filled.\n", order.oid);
                                                    
                                                }
                                                OrderStatus::Resting(order) => {
                                                    println!("Order {} was successfully placed.\n", order.oid);
                                                    
                                                }
                                                OrderStatus::Error(msg) => {
                                                    println!("Order failed with error: {:#?}\n", msg)
                                                }
                                            });
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        println!("{:#?}", err);
                                        return;
                                    }
                                }
                            }
                        } else {
                            // If limit price for eth/btc is .06, wait for the eth/btc ratio to become .06,
                            // then short eth and long btc at market

                            let (
                                base_sz,
                                base_market_price,
                                quote_sz,
                                quote_market_price,
                                current_ratio,
                            ) = loop {
                                let base_market_price = {
                                    let base_asset_ctx = info
                                        .asset_ctx(&pair.base)
                                        .await
                                        .expect("Failed to fetch asset ctxs")
                                        .expect(&format!(
                                            "Failed to find quote asset:  {}",
                                            pair.quote
                                        ));
                                    base_asset_ctx.mark_px.parse::<f64>().unwrap()
                                };

                                let quote_market_price = {
                                    let quote_asset_ctx = info
                                        .asset_ctx(&pair.quote)
                                        .await
                                        .expect("Failed to fetch asset ctxs")
                                        .expect(&format!(
                                            "Failed to find quote asset:  {}",
                                            pair.quote
                                        ));

                                    quote_asset_ctx.mark_px.parse::<f64>().unwrap()
                                };

                                let current_ratio =
                                    format!("{:.2}", base_market_price / quote_market_price)
                                        .parse::<f64>()
                                        .unwrap();

                                if current_ratio <= target {
                                    println!("Ratio reached: {} <= {}", current_ratio, target);
                                    let base_sz = base_sz / base_market_price;
                                    let quote_sz = quote_sz / quote_market_price;

                                    break (
                                        base_sz,
                                        base_market_price,
                                        quote_sz,
                                        quote_market_price,
                                        current_ratio,
                                    );
                                }

                                println!(
                                    "Current Ratio: {}, Target Ratio: {}, Diff: {}. Checking again in 5 seconds\n---",
                                    format!("{:.2}", current_ratio).parse::<f64>().unwrap(),
                                    format!("{:.2}", target).parse::<f64>().unwrap(),
                                    format!("{:.2}", current_ratio - target).parse::<f64>().unwrap(),
                                );
                                tokio::time::sleep(Duration::from_secs(5)).await;
                            };

                            // send sell order request
                            {
                                let order = OrderRequest {
                                    asset: base_asset,
                                    is_buy: false,
                                    limit_px: format_price(
                                        base_market_price * (1.0 - slippage),
                                    ),
                                    sz: format_size(base_sz, base_sz_decimals),
                                    reduce_only: false,
                                    order_type: OrderType::Limit(Limit { tif: Tif::Ioc }),
                                };

                                println!("{}", "---".repeat(20));
                                println!("Order 1 of 2");
                                println!("Side: Sell");
                                println!(
                                    "Size in {}: {}",
                                    pair.base,
                                    format_size(base_sz, base_sz_decimals)
                                );
                                println!(
                                    "Size in USD: {}",
                                    format_size(base_sz * base_market_price, base_sz_decimals)
                                );
                                println!("Market price: {}\n", base_market_price);
                                println!("Ratio: {}\n", current_ratio);

                                match exchange.place_order(order).await {
                                    Ok(order) => {
                                        match order {
                                            ExchangeResponse::Err(err) => {
                                                println!("{:#?}", err);
                                                return;
                                            }
                                            ExchangeResponse::Ok(order) => {
                                                order.data.statuses.iter().for_each(|status| match status {
                                                OrderStatus::Filled(order) => {
                                                    println!("Order {} was successfully filled.\n", order.oid);
                                                    
                                                }
                                                OrderStatus::Resting(order) => {
                                                    println!("Order {} was successfully placed.\n", order.oid);
                                                    
                                                }
                                                OrderStatus::Error(msg) => {
                                                    println!("Order failed with error: {:#?}\n", msg)
                                                }
                                            });
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        println!("{:#?}", err);
                                        return;
                                    }
                                }
                            }

                            // send buy order request
                            {
                                let order = OrderRequest {
                                    asset: quote_asset,
                                    is_buy: true,
                                    limit_px: format_price(
                                        quote_market_price * (1.0 + slippage),
                                    ),
                                    sz: format_size(quote_sz, quote_sz_decimals),
                                    reduce_only: false,
                                    order_type: OrderType::Limit(Limit { tif: Tif::Ioc }),
                                };

                                println!("{}", "---".repeat(20));
                                println!("Order 2 of 2");
                                println!("Side: Buy");
                                println!(
                                    "Size in {}: {}",
                                    pair.quote,
                                    format_size(quote_sz, quote_sz_decimals)
                                );
                                println!(
                                    "Size in USD: {}",
                                    format_size(quote_sz * quote_market_price, quote_sz_decimals)
                                );
                                println!("Market price: {}\n", quote_market_price);
                                println!("Ratio: {}\n", current_ratio);

                                match exchange.place_order(order).await {
                                    Ok(order) => {
                                        match order {
                                            ExchangeResponse::Err(err) => {
                                                println!("{:#?}", err);
                                                return;
                                            }
                                            ExchangeResponse::Ok(order) => {
                                                order.data.statuses.iter().for_each(|status| match status {
                                                OrderStatus::Filled(order) => {
                                                    println!("Order {} was successfully filled.\n", order.oid);
                                                    
                                                }
                                                OrderStatus::Resting(order) => {
                                                    println!("Order {} was successfully placed.\n", order.oid);
                                                    
                                                }
                                                OrderStatus::Error(msg) => {
                                                    println!("Order failed with error: {:#?}\n", msg)
                                                }
                                            });
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        println!("{:#?}", err);
                                        return;
                                    }
                                }
                            }
                        }

                        if tp.is_none() && sl.is_none() {
                            return;
                        };

                        println!("Monitoring positions for tp or sl\n---");

                        let (exit_short_order, exit_long_order, current_ratio) = loop {
                            let base_market_price = {
                                let base_asset_ctx = info
                                    .asset_ctx(&pair.base)
                                    .await
                                    .expect("Failed to fetch asset ctxs")
                                    .expect(&format!(
                                        "Failed to find quote asset:  {}",
                                        pair.quote
                                    ));
                                base_asset_ctx.mark_px.parse::<f64>().unwrap()
                            };

                            let quote_market_price = {
                                let quote_asset_ctx = info
                                    .asset_ctx(&pair.quote)
                                    .await
                                    .expect("Failed to fetch asset ctxs")
                                    .expect(&format!(
                                        "Failed to find quote asset:  {}",
                                        pair.quote
                                    ));

                                quote_asset_ctx.mark_px.parse::<f64>().unwrap()
                            };

                            let current_ratio =
                                format!("{:.2}", base_market_price / quote_market_price)
                                    .parse::<f64>()
                                    .unwrap();

                            // check if tp or sl has been reached
                            if let Some(tp) = tp {
                                if current_ratio <= tp {
                                    println!("Take profit reached: {} <= {}", current_ratio, tp);

                                    let exit_short_order = OrderRequest {
                                        asset: base_asset,
                                        is_buy: true,
                                        limit_px: format_price(
                                            base_market_price * (1.0 + slippage),
                                        ),
                                        sz: format_size(base_sz, base_sz_decimals),
                                        reduce_only: true,
                                        order_type: OrderType::Limit(Limit { tif: Tif::Ioc }),
                                    };

                                    let exit_long_order = OrderRequest {
                                        asset: quote_asset,
                                        is_buy: false,
                                        limit_px: format_price(
                                            quote_market_price * (1.0 - slippage),
                                        ),
                                        sz: format_size(quote_sz, quote_sz_decimals),
                                        reduce_only: true,
                                        order_type: OrderType::Limit(Limit { tif: Tif::Ioc }),
                                    };

                                    break (exit_short_order, exit_long_order, current_ratio);
                                }
                            }

                            if let Some(sl) = sl {
                                if current_ratio >= sl {
                                    println!("Stop loss reached: {} >= {}", current_ratio, sl);

                                    let exit_short_order = OrderRequest {
                                        asset: base_asset,
                                        is_buy: true,
                                        limit_px: format_price(
                                            base_market_price * (1.0 + slippage),
                                        ),
                                        sz: format_size(base_sz, base_sz_decimals),
                                        reduce_only: true,
                                        order_type: OrderType::Limit(Limit { tif: Tif::Ioc }),
                                    };

                                    let exit_long_order = OrderRequest {
                                        asset: quote_asset,
                                        is_buy: false,
                                        limit_px: format_price(
                                            quote_market_price * (1.0 - slippage),
                                        ),
                                        sz: format_size(quote_sz, quote_sz_decimals),
                                        reduce_only: true,
                                        order_type: OrderType::Limit(Limit { tif: Tif::Ioc }),
                                    };

                                    break (exit_short_order, exit_long_order, current_ratio);
                                }
                            }

                            println!(
                                "Current Ratio: {}, Target Ratio Tp: {}, Target Ratio Sl: {}, Tp Diff: {}, Sl Diff: {}. Checking again in 5 seconds\n---",
                                format!("{:.2}", current_ratio).parse::<f64>().unwrap(),
                                format!("{:.2}", tp.unwrap_or(0.0)).parse::<f64>().unwrap(),
                                format!("{:.2}", sl.unwrap_or(0.0)).parse::<f64>().unwrap(),
                                format!("{:.2}", current_ratio - tp.unwrap_or(0.0)).parse::<f64>().unwrap(),
                                format!("{:.2}", current_ratio - sl.unwrap_or(0.0)).parse::<f64>().unwrap(),
                            );

                            tokio::time::sleep(Duration::from_secs(5)).await;
                        };

                        // place exit orders
                        println!("{}", "---".repeat(20));
                        println!("Order 1 of 2");
                        println!("Side: Buy");
                        println!(
                            "Size in {}: {}",
                            pair.base,
                            format_size(base_sz, base_sz_decimals)
                        );
                        println!("Ratio: {}\n", current_ratio);

                        match exchange.place_order(exit_short_order).await {
                            Ok(order) => match order {
                                ExchangeResponse::Err(err) => {
                                    println!("{:#?}", err);
                                    return;
                                }
                                ExchangeResponse::Ok(order) => {
                                    order.data.statuses.iter().for_each(|status| match status {
                                        OrderStatus::Filled(order) => {
                                            println!(
                                                "Order {} was successfully filled.\n",
                                                order.oid
                                            );
                                        }
                                        OrderStatus::Resting(order) => {
                                            println!(
                                                "Order {} was successfully placed.\n",
                                                order.oid
                                            );
                                        }
                                        OrderStatus::Error(msg) => {
                                            println!("Order failed with error: {:#?}\n", msg)
                                        }
                                    });
                                }
                            },
                            Err(err) => {
                                println!("{:#?}", err);
                                return;
                            }
                        }

                        println!("{}", "---".repeat(20));
                        println!("Order 2 of 2");
                        println!("Side: Sell");
                        println!(
                            "Size in {}: {}",
                            pair.quote,
                            format_size(quote_sz, quote_sz_decimals)
                        );
                        println!("Ratio: {}\n", current_ratio);

                        match exchange.place_order(exit_long_order).await {
                            Ok(order) => match order {
                                ExchangeResponse::Err(err) => {
                                    println!("{:#?}", err);
                                    return;
                                }
                                ExchangeResponse::Ok(order) => {
                                    order.data.statuses.iter().for_each(|status| match status {
                                        OrderStatus::Filled(order) => {
                                            println!(
                                                "Order {} was successfully filled.\n",
                                                order.oid
                                            );
                                        }
                                        OrderStatus::Resting(order) => {
                                            println!(
                                                "Order {} was successfully placed.\n",
                                                order.oid
                                            );
                                        }
                                        OrderStatus::Error(msg) => {
                                            println!("Order failed with error: {:#?}\n", msg)
                                        }
                                    });
                                }
                            },
                            Err(err) => {
                                println!("{:#?}", err);
                                return;
                            }
                        }
                    }
                }
            }

            _ => {
                println!("Invalid command: expected commands: (buy, sell)");
            }
        },

        _ => {
            println!("Invalid command: expected commands: (buy, sell, twap, view, pair)");
        }
};
}
