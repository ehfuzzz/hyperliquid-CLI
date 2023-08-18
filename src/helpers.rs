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
                "Invalid percentage format: correct example + 10%",
            ))
        }
    } else if value.starts_with("$") && value[1..].len() > 1 {
        if value[2..].parse::<f64>().is_ok() {
            Ok(())
        } else {
            Err(String::from("Invalid USDC format: correct example +$300"))
        }
    } else if value.ends_with("%pnl") {
        if value.trim_end_matches("%pnl").parse::<f64>().is_ok() {
            Ok(())
        } else {
            Err(String::from(
                " Invalid % pnl format: correct example: +30%pnl",
            ))
        }
    } else if value.ends_with("pnl") {
        if value.trim_end_matches("pnl").parse::<f64>().is_ok() {
            Ok(())
        } else {
            Err(String::from(
                " Invalid pnl format: correct example: +300pnl",
            ))
        }
    } else {
        if validate_value(value).is_err(){
            Err(String::from(
                "Invalid format: Expected tp format: (+10%, +$300, +300pnl + 34%pnl",
            ))
        }else{
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
