use std::time::Duration;

use hyperliquid::types::Chain;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum OrderSize {
    Percent(u8),
    Absolute(f64),
}

impl TryFrom<&str> for OrderSize {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();

        let (size, unit) = if value.ends_with("%") {
            value.split_at(value.len() - 1)
        } else {
            (value, "")
        };

        let size = if size.starts_with("$") {
            let (_, size) = size.split_at(1);
            size
        } else {
            size
        };

        let size = size.parse::<f64>().map_err(|_| "Invalid size")?;
        match unit {
            "%" => Ok(OrderSize::Percent(size as u8)),
            _ => Ok(OrderSize::Absolute(size)),
        }
    }
}

pub struct TwapInterval {
    pub interval: Duration,
    pub num_of_orders: u8,
}

impl TryFrom<&str> for TwapInterval {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();

        let values = value.split(",").collect::<Vec<&str>>();

        let interval = values.get(0).ok_or("Invalid interval")?;
        let num_of_orders = values.get(1).ok_or("Invalid num of orders")?;

        let interval = interval.parse::<u64>().map_err(|_| "Invalid interval")?;
        let interval = Duration::from_secs(interval * 60);

        Ok(TwapInterval {
            interval,
            num_of_orders: num_of_orders
                .parse::<u8>()
                .map_err(|_| "Invalid num of orders")?,
        })
    }
}

pub struct SzPerInterval {
    pub size: f64,
    pub interval: u32,
}

impl TryFrom<&str> for SzPerInterval {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();

        let values = value.split("/").collect::<Vec<&str>>();

        let size = values.get(0).ok_or("Invalid total order size")?;
        let interval = values.get(1).ok_or("Invalid number of intervals")?;

        let size = size
            .parse::<f64>()
            .map_err(|_| "Invalid total order size")?;
        let interval = interval
            .parse::<u32>()
            .map_err(|_| "Invalid number of intervals")?;

        Ok(Self { size, interval })
    }
}

pub enum LimitPrice {
    Absolute(f64),
}

impl TryFrom<&str> for LimitPrice {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();
        // e.g @100 to buy at 100

        let value = if value.starts_with("@") {
            let (_, value) = value.split_at(1);
            value
        } else {
            value
        };

        let value = value.parse::<f64>().map_err(|_| "Invalid price")?;

        Ok(LimitPrice::Absolute(value))
    }
}

pub struct Pair {
    pub base: String,
    pub quote: String,
}

impl TryFrom<&str> for Pair {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();
        let mut value = value.split("/");
        let first = value.next().ok_or("Invalid pair")?;
        let second = value.next().ok_or("Invalid pair")?;

        Ok(Self {
            base: first.to_string(),
            quote: second.to_string(),
        })
    }
}

pub enum TpSl {
    Percent(f64),  // 10%
    Fixed(f64),    // 1990
    Absolute(f64), // +/- 10
}

impl TryFrom<&str> for TpSl {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();
        let (size, unit) = if value.ends_with("%") {
            value.split_at(value.len() - 1)
        } else {
            (value, "")
        };

        let size = size.parse::<f64>().map_err(|_| "Invalid size")?;
        match unit {
            "%" => Ok(TpSl::Percent(size)),
            _ => {
                if value.starts_with("+") || value.starts_with("-") {
                    Ok(TpSl::Absolute(size))
                } else {
                    Ok(TpSl::Fixed(size))
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MarginType {
    Cross,
    Isolated,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub private_key: String,
    pub default_margin: MarginType,
    pub default_asset: String,
    pub default_size: String,
    pub chain: Chain,
}
