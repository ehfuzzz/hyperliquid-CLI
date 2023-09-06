pub fn format_size(sz: f64, decimals: u32) -> String {
    format!("{sz:.0$}", decimals as usize)
}

pub fn format_price(price: f64) -> String {
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
    use crate::helpers::{format_price, format_size};

    #[test]
    fn test_format_limit_price() {
        assert_eq!(format_price(1234.5), "1234.5");
        assert_eq!(format_price(1234.56), "1234.5");
        assert_eq!(format_price(0.001234), "0.001234");
        assert_eq!(format_price(0.0012345), "0.001234");
        assert_eq!(format_price(1.2345678), "1.2345");
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
