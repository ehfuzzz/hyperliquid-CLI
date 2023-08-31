use serde::Deserialize;

pub enum OrderSize {
    Percent(u8),
    Absolute(f64),
}

impl TryFrom<&str> for OrderSize {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();
        let (size, unit) = value.split_at(value.len() - 1);
        let size = size.parse::<f64>().map_err(|_| "Invalid size")?;
        match unit {
            "%" => Ok(OrderSize::Percent(size as u8)),
            _ => Ok(OrderSize::Absolute(size)),
        }
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
        let (_, value) = value.split_at(1);
        let value = value.parse::<f64>().map_err(|_| "Invalid price")?;

        Ok(LimitPrice::Absolute(value))
    }
}

pub enum TpSl {
    Percent(u8),   // 10%
    Fixed(f64),    // 1990
    Absolute(f64), // +/- 10
}

impl TryFrom<&str> for TpSl {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();
        let (size, unit) = value.split_at(value.len() - 1);

        let size = size.parse::<f64>().map_err(|_| "Invalid size")?;
        match unit {
            "%" => Ok(TpSl::Percent(size as u8)),
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

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarginType {
    Cross,
    Isolated,
}
