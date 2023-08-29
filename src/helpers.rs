use crate::hyperliquid::order::{
    build_sl_order_helper, build_sl_payload, build_tp_order_helper, build_tp_payload, place_order,
};
use crate::hyperliquid::order_payload::GainOptions;
use crate::hyperliquid::order_payload::{OrderPayload, Orders};
use std::time::{SystemTime, UNIX_EPOCH};
use rand::Rng;
use hex;

pub fn validate_value_size(value: String) -> Result<(), String> {
    if value.ends_with('%') {
        if value.trim_end_matches('%').parse::<f64>().is_ok() {
            Ok(())
        } else {
            Err(String::from("Invalid percentage format"))
        }
    } else if value.starts_with('$') && value.len() > 1 {
        if value[1..].parse::<f64>().is_ok() {
            Ok(())
        } else {
            Err(String::from("Invalid USDC format"))
        }
    } else {
        Err(String::from(
            "Expected amount in USDC (e.g., '$100' or %balance of your account, e.g., 10%)",
        ))
    }
}

pub fn validate_tp_price(value: String) -> Result<(), String> {
    if value.ends_with("%") {
        if value.trim_end_matches("%").parse::<f64>().is_ok() {
            Ok(())
        } else {
            Err(String::from(
                "Invalid percentage format: correct example 10%",
            ))
        }
    } else if value.starts_with("$") && value.len() > 1 {
        if value[2..].parse::<f64>().is_ok() {
            Ok(())
        } else {
            Err(String::from("Invalid USDC format: correct example $300"))
        }
    } else if value.ends_with("%pnl") {
        if value.trim_end_matches("%pnl").parse::<f64>().is_ok() {
            Ok(())
        } else {
            Err(String::from(
                " Invalid % pnl format: correct example: 30%pnl",
            ))
        }
    } else if value.ends_with("pnl") {
        if value.trim_end_matches("pnl").parse::<f64>().is_ok() {
            Ok(())
        } else {
            Err(String::from(" Invalid pnl format: correct example: 300pnl"))
        }
    } else {
        if validate_value(value).is_err() {
            Err(String::from(
                "Invalid format: Expected tp format: (10%, $300, 300pnl 34%pnl, 1990)",
            ))
        } else {
            Ok(())
        }
    }
}

pub fn validate_sl_price(value: String) -> Result<(), String> {
    if value.starts_with("-") {
        if value[1..].ends_with("%") {
            if value[1..].trim_end_matches("%").parse::<f64>().is_ok() {
                Ok(())
            } else {
                Err(String::from(
                    "Invalid percentage format: correct example - 10%",
                ))
            }
        } else if value[1..].starts_with("$") && value[1..].len() > 1 {
            if value[2..].parse::<f64>().is_ok() {
                Ok(())
            } else {
                Err(String::from("Invalid USDC format: correct example -$300"))
            }
        } else if value.ends_with("%pnl") {
            if value[1..].trim_end_matches("%pnl").parse::<f64>().is_ok() {
                Ok(())
            } else {
                Err(String::from(
                    " Invalid % pnl format: correct example: -30%pnl",
                ))
            }
        } else if value.ends_with("pnl") {
            if value[1..].trim_end_matches("pnl").parse::<f64>().is_ok() {
                Ok(())
            } else {
                Err(String::from(
                    " Invalid pnl format: correct example: -300pnl",
                ))
            }
        } else {
            Err(String::from(
                "Invalid format: Expected sl format: (-10%, -$300, -300pnl - 34%pnl",
            ))
        }
    } else {
        Err(String::from(
            " Invalid format: Expected sl format: (-10%, -$300, -300pnl - 34%pnl ",
        ))
    }
}

pub fn validate_limit_price(value: String) -> Result<(), String> {
    if value.starts_with("@") && value.len() > 1 {
        if validate_value(value[1..].to_string()).is_ok() {
            Ok(())
        } else {
            Err(String::from(
                "Invalid limit price format: correct example @100",
            ))
        }
    } else {
        Err(String::from(
            "Invalid limit price format: correct example @100",
        ))
    }
}

pub fn validate_value(value: String) -> Result<(), String> {
    if value.parse::<f64>().is_ok() {
        Ok(())
    } else {
        Err(String::from("Invalid price format: correct example 100"))
    }
}

pub async fn place_sl_order(
    asset: u32,
    is_buy: bool,
    sl_price: &str,
    limit_px: &str,
    sz: &str,
    reduce_only: bool,
    gain_flag: bool,
) {
    let numeric_part: f64 = match sl_price {
        sl_price if sl_price.trim_start_matches("-").ends_with("%") => {
            sl_price[0..sl_price.len() - 1].parse().unwrap()
        }
        sl_price if sl_price.starts_with("-$") => sl_price[2..].parse().unwrap(),
        sl_price if validate_value(sl_price.to_string()).is_ok() => sl_price.parse().unwrap(),
        sl_price if sl_price.trim_start_matches("-").ends_with("%pnl") => {
            sl_price[0..sl_price.len() - 4].parse().unwrap()
        }
        sl_price if sl_price.trim_start_matches("-").ends_with("pnl") => {
            sl_price[0..sl_price.len() - 3].parse().unwrap()
        }
        _ => return,
    };

    let gain = if sl_price.trim_start_matches("-").ends_with("%") {
        GainOptions::PercentageGain(numeric_part)
    } else {
        GainOptions::DollarGain(numeric_part)
    };

    let sl_payload = build_sl_payload(asset, is_buy, &limit_px, sz, reduce_only, gain, gain_flag);
    let response = place_order(sl_payload).await;

    println!("Logic for handling sl price: {:#?}", response);
}

pub async fn place_tp_order(
    asset: u32,
    is_buy: bool,
    tp_price: &str,
    limit_px: &str,
    sz: &str,
    reduce_only: bool,
    gain_flag: bool,
) {
    let numeric_part: f64 = match tp_price {
        tp_price if tp_price.ends_with("%") => tp_price[0..tp_price.len() - 1].parse().unwrap(),
        tp_price if tp_price.starts_with("$") => tp_price[1..].parse().unwrap(),
        tp_price if tp_price.ends_with("%pnl") => tp_price[0..tp_price.len() - 4].parse().unwrap(),
        tp_price if tp_price.ends_with("pnl") => tp_price[0..tp_price.len() - 3].parse().unwrap(),
        tp_price if validate_value(tp_price.to_string()).is_ok() => tp_price.parse().unwrap(),
        _ => return,
    };

    let gain = if tp_price.ends_with("%") || tp_price.ends_with("%pnl") {
        GainOptions::PercentageGain(numeric_part)
    } else {
        GainOptions::DollarGain(numeric_part)
    };

    let _tp_payload: OrderPayload =
        build_tp_payload(asset, is_buy, &limit_px, sz, reduce_only, gain, gain_flag);
    let response = place_order(_tp_payload).await;

    println!("Logic for handling {} tp price: {:#?}", tp_price, response);
}

pub fn build_tp_order(
    asset: u32,
    is_buy: bool,
    limit_px: &str,
    tp_price: &str,
    sz: &str,
    reduce_only: bool,
    gain_flag: bool,
) -> Option<Orders> {
    let numeric_part: f64 = match tp_price {
        tp_price if tp_price.ends_with("%") => tp_price[0..tp_price.len() - 1].parse().unwrap(),
        tp_price if tp_price.starts_with("$") => tp_price[1..].parse().unwrap(),
        tp_price if tp_price.ends_with("%pnl") => tp_price[0..tp_price.len() - 4].parse().unwrap(),
        tp_price if tp_price.ends_with("pnl") => tp_price[0..tp_price.len() - 3].parse().unwrap(),
        tp_price if validate_value(tp_price.to_string()).is_ok() => tp_price.parse().unwrap(),
        _ => unreachable!("Invalid tp price format"),
    };

    let gain = if tp_price.ends_with("%") || tp_price.ends_with("%pnl") {
        GainOptions::PercentageGain(numeric_part)
    } else {
        GainOptions::DollarGain(numeric_part)
    };

    let tp_order: Orders =
        build_tp_order_helper(asset, is_buy, &limit_px, sz, reduce_only, gain, gain_flag);
    Some(tp_order)
}

pub fn build_sl_order(
    asset: u32,
    is_buy: bool,
    limit_px: &str,
    sl_price: &str,
    sz: &str,
    reduce_only: bool,
    gain_flag: bool,
) -> Option<Orders> {
    let numeric_part: f64 = match sl_price {
        sl_price if sl_price.trim_start_matches("-").ends_with("%") => {
            sl_price[0..sl_price.len() - 1].parse().unwrap()
        }
        sl_price if sl_price.starts_with("-$") => sl_price[2..].parse().unwrap(),
        sl_price if validate_value(sl_price.to_string()).is_ok() => sl_price.parse().unwrap(),
        sl_price if sl_price.trim_start_matches("-").ends_with("%pnl") => {
            sl_price[0..sl_price.len() - 4].parse().unwrap()
        }
        sl_price if sl_price.trim_start_matches("-").ends_with("pnl") => {
            sl_price[0..sl_price.len() - 3].parse().unwrap()
        }
        _ => unreachable!("Invalid sl price format"),
    };

    let gain = if sl_price.trim_start_matches("-").ends_with("%") {
        GainOptions::PercentageGain(numeric_part)
    } else {
        GainOptions::DollarGain(numeric_part)
    };

    let sl_order: Orders =
        build_sl_order_helper(asset, is_buy, &limit_px, sz, reduce_only, gain, gain_flag);
    Some(sl_order)
}

pub fn get_current_time_in_milliseconds() -> u128 {
    let now = SystemTime::now();
    now.duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
}

pub fn generate_transaction_signature() -> String {
    let mut rng = rand::thread_rng();
    let random_number: u128 = rng.gen();
    let random_number_string = random_number.to_string();
    let random_number_string = random_number_string.as_bytes();
    let random_number_string = hex::encode(random_number_string);
    random_number_string
}
