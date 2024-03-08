use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use ethers::{
    signers::{LocalWallet, Signer},
    types::Address,
};
use hyperliquid::{
    types::{
        exchange::{
            request::{
                CancelRequest, Limit, ModifyRequest, OrderRequest, OrderType, Tif, TpSl, Trigger,
            },
            response::{Response, Status},
        },
        info::response::{AssetContext, Ctx},
        Cloid, Oid,
    },
    utils::{parse_price, parse_size},
    Exchange, Info,
};

pub fn asset_ctx<'a>(
    asset_ctxs: &'a Vec<AssetContext>,
    asset: &str,
) -> Result<Option<&'a Ctx>, anyhow::Error> {
    let universe = match asset_ctxs.get(0) {
        Some(AssetContext::Meta(universe)) => universe,
        _ => return Ok(None),
    };

    let position = universe
        .universe
        .iter()
        .position(|a| a.name.to_uppercase() == asset.to_uppercase())
        .expect("Asset not found");

    let ctxs = match asset_ctxs.get(1) {
        Some(AssetContext::Ctx(ctxs)) => ctxs,
        _ => return Ok(None),
    };

    let ctx = ctxs.get(position);

    Ok(ctx)
}
pub async fn limit_chase(
    exchange: &Exchange,
    info: &Info,
    wallet: Arc<LocalWallet>,
    mut oid: u64,
    vault_address: Option<Address>,
    symbol: &str,
    asset: u32,
    is_buy: bool,
    sz: f64,
    sz_decimals: u32,
    reduce_only: bool,
    discard_price: Option<f64>,
    cloid: Option<Cloid>,
) -> Option<u64> {
    const SLIPPAGE: f64 = 0.01;

    const MAX_CHASE_COUNT: i32 = 100;

    const MAX_CHASE_DURATION: u64 = 60;

    const CHASE_INTERVAL_IN_SEC: u64 = 5;

    const TRAIL_PCT: f64 = 1.0 / 100.0;

    let start = Instant::now();

    let mut loop_count = 1;

    loop {
        let asset_ctxs = info.contexts().await.expect("Failed to fetch asset ctxs");

        let ctx = asset_ctx(&asset_ctxs, symbol)
            .expect("Failed to fetch asset ctxs")
            .expect("Failed to find asset");

        let market_price = ctx.mark_px.parse::<f64>().unwrap();

        // Give up limit chasing and place a market order if
        // 1. max chase count is exceeded and the order is still resting or
        // 2. if a certain time has passed or
        // 3. if the market price is better than the discard price
        let is_market = loop_count > MAX_CHASE_COUNT
            || start.elapsed().as_secs() > MAX_CHASE_DURATION
            || if let Some(discard_price) = discard_price {
                if is_buy {
                    market_price > discard_price
                } else {
                    market_price < discard_price
                }
            } else {
                false
            };

        let limit_px = parse_price(
            market_price * 1.0
                + if is_market {
                    if is_buy {
                        SLIPPAGE
                    } else {
                        -SLIPPAGE
                    }
                } else {
                    if is_buy {
                        -TRAIL_PCT
                    } else {
                        TRAIL_PCT
                    }
                },
        );

        println!("---\nChasing order with limit price: {}, Market price: {}, Chase interval: {} seconds, Duration: {} seconds, Chase Count: {}", limit_px, market_price, CHASE_INTERVAL_IN_SEC, start.elapsed().as_secs(), loop_count);

        let order = OrderRequest {
            asset,
            is_buy,
            limit_px,
            sz: parse_size(sz, sz_decimals),
            reduce_only,
            order_type: OrderType::Limit(Limit { tif: Tif::Gtc }),
            cloid,
        };

        match exchange
            .batch_modify_orders(
                wallet.clone(),
                vec![ModifyRequest { oid, order }],
                vault_address,
            )
            .await
        {
            Ok(order) => match order {
                Response::Err(err) => {
                    println!("1.{:#?}", err);
                    break None;
                }

                Response::Ok(order) => {
                    let mut filled = false;
                    order
                        .data
                        .expect("expected order response data")
                        .statuses
                        .iter()
                        .for_each(|status| match status {
                            Status::Filled(order) => {
                                println!("\nOrder {} was successfully filled ✓", order.oid);
                                oid = order.oid;
                            }
                            Status::Resting(order) => {
                                println!("\nOrder {} was successfully placed ✔️", order.oid);
                                oid = order.oid;
                            }
                            Status::Error(msg) => {
                                filled = msg.contains("Cannot modify canceled or filled order");
                                if filled {
                                    println!("\nOrder {} was successfully filled ✓", oid);
                                } else {
                                    println!("\nOrder failed with error: {:#?}\n", msg);
                                }
                            }
                            _ => unreachable!(),
                        });

                    if is_market {
                        println!("---\nExited limit chase and placed a market order, Chase interval: {} seconds, Duration: {} seconds, Chase Count: {}", CHASE_INTERVAL_IN_SEC, start.elapsed().as_secs(), loop_count);
                        break Some(oid);
                    }

                    if filled {
                        break Some(oid);
                    }
                }
            },
            Err(err) => {
                println!("2.{:#?}", err);
                return None;
            }
        }

        loop_count += 1;

        tokio::time::sleep(Duration::from_secs(CHASE_INTERVAL_IN_SEC)).await;
    }
}

pub async fn trail_stop_loss(
    exchange: &Exchange,
    info: &Info,
    wallet: Arc<LocalWallet>,
    poid: u64,
    mut oid: u64,
    symbol: &str,
    asset: u32,
    is_buy: bool,
    sz: f64,
    sz_decimals: u32,
) {
    let user = wallet.address();

    const RETRY_INTERVAL_IN_SEC: u64 = 5;
    const MAX_TSL_CHECK_COUNT: u64 = 5;

    let mut entry_price = 0.0; // Entry price of the position
    let mut is_filled = false;

    let mut trigger_price;

    let mut tsl_check_count = 0;

    loop {
        // Skip if the parent order is still open
        if !is_filled {
            let order = info
                .order_status(user, Oid::Order(poid))
                .await
                .expect("Failed to fetch order");

            if let Some(order) = order.order {
                is_filled = order.status == "filled";

                println!(
                    "Order status: {}{}",
                    order.status,
                    if order.status == "open" {
                        ", rechecking in 5s..."
                    } else {
                        ""
                    }
                );

                if order.status == "canceled" {
                    println!(
                        "\nOrder {} was canceled, canceling trailing stop loss order ⚙️",
                        poid
                    );

                    // cancel trailing stop loss order
                    match exchange
                        .cancel_order(wallet.clone(), vec![CancelRequest { asset, oid }], None)
                        .await
                    {
                        Ok(order) => match order {
                            Response::Err(err) => {
                                println!("{:#?}", err);
                                break;
                            }

                            Response::Ok(order) => {
                                let mut cancel = false;

                                order
                                    .data
                                    .expect("expected order response data")
                                    .statuses
                                    .iter()
                                    .for_each(|status| match status {
                                        Status::Success => {
                                            println!(
                                                "\nTrailing stop loss order {} was successfully canceled ✓",
                                                oid
                                            );
                                            cancel = true;
                                        }
                                        Status::Error(msg) => {
                                            println!(
                                                "\nTrailing stop loss order failed with error: {:#?}\n",
                                                msg
                                            );
                                            cancel = true
                                        }
                                        _ => unreachable!(),
                                    });

                                if cancel {
                                    break;
                                }
                            }
                        },
                        Err(err) => {
                            println!("{:#?}", err);
                            return;
                        }
                    }
                }
            }

            if is_filled {
                let positions = info
                    .user_fills(user)
                    .await
                    .expect("Failed to fetch open orders");

                let position = positions.iter().find(|position| position.oid == poid);

                if let Some(position) = position {
                    entry_price = position.px.parse::<f64>().unwrap();
                }
            }

            tokio::time::sleep(Duration::from_secs(RETRY_INTERVAL_IN_SEC)).await;
            continue;
        }

        // fetch tsl order
        let orders = info
            .open_orders(user)
            .await
            .expect("Failed to fetch open orders");

        let order = orders.iter().find(|order| order.oid == oid);

        if let Some(order) = order {
            trigger_price = order.limit_px.parse::<f64>().unwrap();
        } else {
            tsl_check_count += 1;

            println!(
                "CHECK {}/{}: TSL order {} was not found 🙆‍♂️",
                tsl_check_count, MAX_TSL_CHECK_COUNT, oid
            );

            if tsl_check_count >= MAX_TSL_CHECK_COUNT {
                println!(
                    "---\nTrailing stop loss order was not found for {tsl_check_count} times, exiting"
                );
                break;
            }

            tokio::time::sleep(Duration::from_secs(RETRY_INTERVAL_IN_SEC)).await;
            continue;
        }

        let asset_ctxs = info.contexts().await.expect("Failed to fetch asset ctxs");

        let ctx = asset_ctx(&asset_ctxs, symbol)
            .expect("Failed to fetch asset ctxs")
            .expect("Failed to find asset");

        let market_price = ctx.mark_px.parse::<f64>().unwrap();

        if market_price < entry_price && !is_buy || market_price > entry_price && is_buy {
            println!(
                "\n[SKIPPING] Mark {} is not favorable to adjust TSL, rechecking in {}s...",
                parse_price(market_price),
                RETRY_INTERVAL_IN_SEC
            );
            tokio::time::sleep(Duration::from_secs(RETRY_INTERVAL_IN_SEC)).await;
            continue;
        }

        trigger_price += market_price - entry_price;

        entry_price = market_price;

        println!("--\nAdjusting trailing stop loss to {} ⚙️", trigger_price);

        let order_type = OrderType::Trigger(Trigger {
            trigger_px: parse_price(trigger_price).parse().unwrap(),
            is_market: true,
            tpsl: TpSl::Sl,
        });

        let order = OrderRequest {
            asset,
            is_buy,
            limit_px: parse_price(trigger_price),
            sz: parse_size(sz, sz_decimals),
            reduce_only: true,
            order_type,
            cloid: None,
        };

        match exchange
            .batch_modify_orders(wallet.clone(), vec![ModifyRequest { oid, order }], None)
            .await
        {
            Ok(order) => match order {
                Response::Err(err) => {
                    println!("{:#?}", err);
                    return;
                }

                Response::Ok(order) => {
                    let mut filled = false;
                    order
                        .data
                        .expect("expected order response data")
                        .statuses
                        .iter()
                        .for_each(|status| match status {
                            Status::Filled(order) => {
                                println!(
                                    "\nTrailing stop loss order {} was successfully filled ✓",
                                    order.oid
                                );
                            }
                            Status::Resting(order) => {
                                println!(
                                    "\nTrailing stop loss order {} was successfully placed ✔️",
                                    order.oid
                                );

                                oid = order.oid;
                            }
                            Status::Error(msg) => {
                                filled = msg.contains("Cannot modify canceled or filled order");
                                if filled {
                                    println!(
                                        "\nTrailing stop loss order {} was successfully filled ✓",
                                        oid
                                    );
                                } else {
                                    println!(
                                        "\nTrailing stop loss order failed with error: {:#?}\n",
                                        msg
                                    );
                                }
                            }
                            _ => unreachable!(),
                        });

                    if filled {
                        break;
                    }
                }
            },
            Err(err) => {
                println!("{:#?}", err);
                return;
            }
        }

        tokio::time::sleep(Duration::from_secs(RETRY_INTERVAL_IN_SEC)).await;
    }
}
