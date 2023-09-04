use hex;
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};

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
        if value.parse::<f64>().is_ok() {
            Ok(())
        } else {
            Err(String::from(
                "Expected amount in USDC (e.g., '$100' or %balance of your account, e.g., 10%)",
            ))
        }
    }
}

pub fn validate_tp_price(value: String) -> Result<(), String> {
    if value.ends_with("%") {
        if value.trim_end_matches("%").parse::<f64>().is_ok() {
            Ok(())
        } else {
            Err(String::from(
                "tp price or %/$ gain in asset before tp or %/$ gain in pnl before tp e.g 10%",
            ))
        }
    } else {
        if value.parse::<f64>().is_ok() {
            Ok(())
        } else {
            Err(String::from("Invalid USDC format e.g 100"))
        }
    }
}

pub fn validate_sl_price(value: String) -> Result<(), String> {
    if value.ends_with("%") {
        if value.trim_end_matches("%").parse::<f64>().is_ok() {
            Ok(())
        } else {
            Err(String::from(
                "sl price or %/$ loss in asset before sl or %/$ loss in pnl before sl e.g 10%",
            ))
        }
    } else {
        if value.parse::<f64>().is_ok() {
            Ok(())
        } else {
            Err(String::from("Invalid USDC format e.g 100"))
        }
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
        if validate_value(value).is_ok() {
            Ok(())
        } else {
            Err(String::from(
                "Invalid limit price format: correct example 100",
            ))
        }
    }
}

pub fn validate_value(value: String) -> Result<(), String> {
    if value.parse::<f64>().is_ok() {
        Ok(())
    } else {
        Err(String::from("Invalid price format: correct example 100"))
    }
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

pub fn float_to_int_for_hashing(num: f64) -> u64 {
    (num * 100_000_000.0).round() as u64
}

pub fn format_size(sz: f64, decimals: u32) -> String {
    format!("{sz:.0$}", decimals as usize)
}

pub fn format_limit_price(price: f64) -> String {
    let price = format!("{price:.5}");

    if price.starts_with("0.") {
        price
    } else {
        let price: Vec<&str> = price.split(".").collect();
        let whole = price[0];
        let decimals = price[1];

        let diff = 5 - whole.len(); // 0
        let sep = if diff > 0 { "." } else { "" };

        format!("{whole}{sep}{decimals:.0$}", diff)
    }
}

#[cfg(test)]
mod tests {
    use crate::helpers::{format_limit_price, format_size};

    #[test]
    fn test_format_limit_price() {
        assert_eq!(format_limit_price(1234.5), "1234.5");
        assert_eq!(format_limit_price(1234.56), "1234.5");
        assert_eq!(format_limit_price(0.001234), "0.001234");
        assert_eq!(format_limit_price(0.0012345), "0.001234");
        assert_eq!(format_limit_price(1.2345678), "1.2345");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(1.001, 3), "1.001");
        assert_eq!(format_size(1.001, 2), "1.00");
        assert_eq!(format_size(1.0001, 3), "1.000");

        assert_eq!(format_size(1.001, 0), "1");

        assert_eq!(format_size(1.001, 5), "1.00100");
    }
}
