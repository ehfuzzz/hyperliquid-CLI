use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use ethers::{signers::LocalWallet, types::Address};
use hyperliquid::{
    types::{
        exchange::{
            request::{Cloid, Limit, ModifyRequest, OrderRequest, OrderType, Tif},
            response::{Response, Status},
        },
        info::response::{AssetContext, Ctx},
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
    cloid: Option<Cloid>,
) {
    const SLIPPAGE: f64 = 0.01;

    const MAX_CHASE_COUNT: i32 = 100;

    const MAX_CHASE_DURATION: u64 = 60;

    const CHASE_INTERVAL_IN_SEC: u64 = 5;

    const TRAIL_PCT: f64 = 1.0 / 100.0;

    let start = Instant::now();

    let mut loop_count = 1;

    loop {
        // Give up limit chasing and place a market order if max chase count is exceeded and the order is still resting or if a certain time has passed
        let is_market =
            loop_count > MAX_CHASE_COUNT || start.elapsed().as_secs() > MAX_CHASE_DURATION;

        let asset_ctxs = info.contexts().await.expect("Failed to fetch asset ctxs");

        let ctx = asset_ctx(&asset_ctxs, symbol)
            .expect("Failed to fetch asset ctxs")
            .expect("Failed to find asset");

        let market_price = ctx.mark_px.parse::<f64>().unwrap();

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

        println!("---\nChasing order with Limit price: {}, Market price: {}, Chase interval: {} seconds, Duration: {} seconds, Chase Count: {}", limit_px, market_price, CHASE_INTERVAL_IN_SEC, start.elapsed().as_secs(), loop_count);

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
                    break;
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
                        break;
                    }

                    if filled {
                        break;
                    }
                }
            },
            Err(err) => {
                println!("2.{:#?}", err);
                return;
            }
        }

        loop_count += 1;

        tokio::time::sleep(Duration::from_secs(CHASE_INTERVAL_IN_SEC)).await;
    }
}
