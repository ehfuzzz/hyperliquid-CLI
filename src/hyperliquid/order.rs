use crate::hyperliquid::order_payload::{
    GainOptions, OrderPayload, OrderType, Orders, RequestBody, Trigger,
};
use crate::hyperliquid::order_responses::PlaceResponse;
use reqwest::Client;

pub async fn place_order(
    order_payload: OrderPayload,
) -> Result<PlaceResponse, Box<dyn std::error::Error>> {
    let client = Client::new();
    let requestbody = RequestBody {
        action: order_payload,
        nonce: 0,
        signature: String::from(""),
        vaultaddress: None,
    };
    let json_body =
        serde_json::to_string(&requestbody).expect("Failed to serialize the request body");
    let resp = client
        .post("https://api.hyperliquid.xyz/exchange")
        .body(json_body)
        .header("Content-Type", "application/json")
        .send()
        .await?
        .json::<PlaceResponse>()
        .await?;
    Ok(resp)
}

pub fn handle_tp_logic(gain: GainOptions, is_buy: bool, gain_flag: bool) -> Trigger {
    let mut trigger = Trigger::new("tp");
    let entry_price = 2000.0; // Example entry price
    let leverage = 50.0; // Leverage factor

    let actual_tp_price = if gain_flag {
        match gain {
            GainOptions::PercentageGain(percentage) => {
                let target_price_gain = entry_price * (percentage / 100.0);
                let target_price = if is_buy {
                    entry_price + target_price_gain
                } else {
                    entry_price - target_price_gain
                };

                target_price / leverage
            }
            GainOptions::DollarGain(dollar) => {
                let target_price = if is_buy {
                    entry_price - dollar
                } else {
                    entry_price + dollar
                };

                target_price / leverage
            }
        }
    } else {
        match gain {
            GainOptions::DollarGain(dollar) => {
                dollar // Use the dollar directly if gain_flag is false
            }
            _ => {
                panic!("Invalid gain option when gain_flag is false");
            }
        }
    };

    if (is_buy && actual_tp_price <= entry_price) || (!is_buy && actual_tp_price >= entry_price) {
        panic!("TP price must be higher than entry price for longs and lower than entry price for shorts");
    }

    trigger.set_trigger_px(actual_tp_price);
    trigger.set_is_market(false);

    trigger
}

pub fn handle_sl_logic(gain: GainOptions, is_buy: bool, gain_flag: bool) -> Trigger {
    let mut trigger = Trigger::new("sl");

    let entry_price = 2000.0; // Example entry price
    let leverage = 50.0; // Leverage factor

    let actual_sl_price: f64 = if gain_flag {
        match gain {
            GainOptions::PercentageGain(percentage) => {
                let target_price_gain = entry_price * (percentage / 100.0);
                let target_price = if is_buy {
                    entry_price - target_price_gain
                } else {
                    entry_price + target_price_gain
                };

                target_price / leverage
            }
            GainOptions::DollarGain(dollar) => {
                let target_price = if is_buy {
                    entry_price + dollar
                } else {
                    entry_price - dollar
                };

                target_price / leverage
            }
        }
    } else {
        match gain {
            GainOptions::DollarGain(dollar) => {
                dollar // Use the dollar directly if gain_flag is false
            }
            _ => {
                panic!("Invalid gain option when gain_flag is false");
            }
        }
    };

    if (is_buy && actual_sl_price >= entry_price) || (!is_buy && actual_sl_price <= entry_price) {
        panic!("SL price must be lower than entry price for longs and higher than entry price for shorts");
    }

    trigger.set_trigger_px(actual_sl_price);
    trigger.set_is_market(false);

    trigger
}

pub fn build_tp_payload(
    asset: u32,
    is_buy: bool,
    limit_px: &str,
    sz: &str,
    reduce_only: bool,
    gain: GainOptions,
    gain_flag: bool,
) -> OrderPayload {
    let mut order_payload = OrderPayload::new();
    let mut tp_order = Orders::new();
    let trigger = handle_tp_logic(gain, is_buy, gain_flag);
    tp_order.set_asset(asset);
    tp_order.set_is_buy(is_buy);
    tp_order.set_limit_px(&limit_px);
    tp_order.set_sz(&sz);
    tp_order.set_reduce_only(reduce_only);
    tp_order.set_order_type(OrderType::Trigger(trigger));
    order_payload.add_order(tp_order);
    order_payload
}

pub fn build_sl_payload(
    asset: u32,
    is_buy: bool,
    limit_px: &str,
    sz: &str,
    reduce_only: bool,
    gain: GainOptions,
    gain_flag: bool,
) -> OrderPayload {
    let mut order_payload = OrderPayload::new();
    let mut sl_order = Orders::new();
    let trigger = handle_sl_logic(gain, is_buy, gain_flag);
    sl_order.set_asset(asset);
    sl_order.set_is_buy(is_buy);
    sl_order.set_limit_px(&limit_px);
    sl_order.set_sz(&sz);
    sl_order.set_reduce_only(reduce_only);
    sl_order.set_order_type(OrderType::Trigger(trigger));
    order_payload.add_order(sl_order);
    order_payload
}

pub fn build_tp_order_helper(
    asset: u32,
    is_buy: bool,
    limit_px: &str,
    sz: &str,
    reduce_only: bool,
    gain: GainOptions,
    gain_flag: bool,
) -> Orders {
    let mut tp_order = Orders::new();
    let trigger = handle_tp_logic(gain, is_buy, gain_flag);
    tp_order.set_asset(asset);
    tp_order.set_is_buy(is_buy);
    tp_order.set_limit_px(&limit_px);
    tp_order.set_sz(&sz);
    tp_order.set_reduce_only(reduce_only);
    tp_order.set_order_type(OrderType::Trigger(trigger));
    tp_order
}

pub fn build_sl_order_helper(
    asset: u32,
    is_buy: bool,
    limit_px: &str,
    sz: &str,
    reduce_only: bool,
    gain: GainOptions,
    gain_flag: bool,
) -> Orders {
    let mut sl_order = Orders::new();
    let trigger = handle_sl_logic(gain, is_buy, gain_flag);
    sl_order.set_asset(asset);
    sl_order.set_is_buy(is_buy);
    sl_order.set_limit_px(&limit_px);
    sl_order.set_sz(&sz);
    sl_order.set_reduce_only(reduce_only);
    sl_order.set_order_type(OrderType::Trigger(trigger));
    sl_order
}

pub fn build_buy_order(
    buy_order: Orders,
    tp_order: Option<Orders>,
    sl_order: Option<Orders>,
) -> OrderPayload {
    let mut order_payload = OrderPayload::new();
    order_payload.add_order(buy_order);

    if let Some(tp_order) = tp_order {
        order_payload.add_order(tp_order);
    }

    if let Some(sl_order) = sl_order {
        order_payload.add_order(sl_order);
    }

    order_payload
}

pub fn build_sell_order(
    sell_order: Orders,
    tp_order: Option<Orders>,
    sl_order: Option<Orders>,
) -> OrderPayload {
    let mut order_payload = OrderPayload::new();
    order_payload.add_order(sell_order);

    if let Some(tp_order) = tp_order {
        order_payload.add_order(tp_order);
    }

    if let Some(sl_order) = sl_order {
        order_payload.add_order(sl_order);
    }

    order_payload
}
