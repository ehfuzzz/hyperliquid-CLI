//using version 2.33 not the latest one
use clap::{App, Arg};
use std::num::ParseFloatError;

#[tokio::main]
async fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("A CLI bot to interact with the hyperliquid exchange")
        .subcommand(
            App::new("set")
                .about("Handles all the specified set commands")
                .subcommand(
                    App::new("ds")
                        .about("Sets the default size")
                        .arg(
                            Arg::with_name("size_type")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .possible_values(&["risk", "notional"])
                                .help("Either risk or notional"),
                        )
                        .arg(
                            Arg::with_name("value_size")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .validator(validate_value_size)
                                .help("Size in USDC or size in % of balance"),
                        ),

                )
                .subcommand(
                    App::new("dm")
                        .about("Sets the default margin")
                        .arg(
                            Arg::with_name("margin_type")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .possible_values(&["i", "c"])
                                .help("Default margin type Either Isolated(i) or cross margin(c)")
                        )
                )
        )
        .subcommand(
            App::new("tp")
                .about(" Handles Take profit command")
                .arg(
                    Arg::with_name("percentage_order")
                        .required(true)
                        .index(1)
                        .takes_value(true)
                        .help("% of order to TP")
                        .validator(validate_value_size)
                )
                .arg(
                    Arg::with_name("asset_symbol")
                        .required(true)
                        .index(2)
                        .takes_value(true)
                        .help("asset symbol e.g ETH, SOL, BTC .., optional if default asset is provided")

                )
                .arg(
                    Arg::with_name("tp_price")
                        .required(true)
                        .index(3)
                        .takes_value(true)
                        .help(" Take profit price or %/$ gain in asset before tp or %$ gain in pnl before tp")
                        .validator(validate_tp_price)


                )


        )

        .get_matches();

    if let Some(set_matches) = matches.subcommand_matches("set") {
        if let Some(ds_matches) = set_matches.subcommand_matches("ds") {
            let size_type = ds_matches.value_of("size_type").unwrap();
            let value_size = ds_matches.value_of("value_size").unwrap();

            let converted_value: Result<f64, ParseFloatError> = if value_size.ends_with('%') {
                value_size
                    .trim_end_matches('%')
                    .parse::<f64>()
                    .map(|percent| percent / 100.0)
            } else {
                value_size
                    .trim_start_matches('$')
                    .parse::<f64>()
            };

            match size_type {
                "risk" => {
                    match converted_value {
                        Ok(value) => {
                            handle_risk_value(value);
                        }
                        Err(_) => {
                            println!("Invalid value format");
                        }
                    }
                }
                "notional" => {
                    match converted_value {
                        Ok(value) => {
                            handle_notional_value(value);
                        }
                        Err(_) => {
                            println!("Invalid value format");
                        }
                    }
                }
                _ => unreachable!(), 
            }
        }else if let Some(ds_matches) = set_matches.subcommand_matches("dm") {
            let margin_type= ds_matches.value_of("margin_type").unwrap();
            println! ("Margin type: {}", margin_type);
            
            match margin_type {
                "i" => {
                    handle_isolated_margin(margin_type)
                }
                "c" => {
                    handle_cross_margin(margin_type)
                }
                _=> unreachable!(), // we should not get here because of the possible value checker
            
            }
        }
    }else if let Some(tp_matches) = matches.subcommand_matches("tp"){
        let percentage_order = tp_matches.value_of("percentage_order").unwrap();
        let asset_symbol = tp_matches.value_of("asset_symbol").unwrap();
        let tp_price = tp_matches.value_of("tp_price").unwrap();

        let converted_percentage_order: Result<f64, ParseFloatError> = {
            percentage_order
                .trim_end_matches("%")
                .parse::<f64>()
                .map(|percent|percent/ 100.0)
        };
        println! ("converted percentage order: {:?}, asset_symbol: {}", converted_percentage_order, asset_symbol);

        match tp_price {
            tp_price if tp_price.trim_start_matches("+").ends_with("%") => {
                let numeric_part = &tp_price[1..tp_price.len() - 1];
                let converted_value = numeric_part.parse::<f64>().unwrap() / 100.0;
                println!("Logic for handling +10% tp price: {}", converted_value);
            }
            tp_price if tp_price.starts_with("+$") => {
                let numeric_part = &tp_price[2..];
                let converted_value = numeric_part.parse::<u32>().unwrap();
                println!("Logic for handling +$300: {}", converted_value);
            }
            tp_price if tp_price.trim_start_matches("+").ends_with("%pnl") => {
                let numeric_part = &tp_price[1..tp_price.len() - 4];
                let converted_value = numeric_part.parse::<f64>().unwrap() / 100.0;
                println!("Logic for handling +10%pnl: {}", converted_value);
            }            
            tp_price if tp_price.trim_start_matches("+").ends_with("pnl") => {
                let numeric_part = &tp_price[1..tp_price.len() - 3];
                let converted_value = numeric_part.parse::<u32>().unwrap();
                println!("Logic for handling +300pnl: {}", converted_value);
            }

            _ => {
                println!("No matching pattern");
            }
        }
        

    }
}

fn validate_value_size(value: String) -> Result<(), String> {
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

fn validate_tp_price(value: String) -> Result<(), String> {

    if value.starts_with("+"){
        if value[1..].ends_with("%"){
            if value[1..].trim_end_matches("%").parse::<f64>().is_ok(){
                Ok(())
            }else{
                Err(String::from("Invalid percentage format: correct example + 10%"))
            }

        }else if value[1..].starts_with("$") && value[1..].len() > 1{
            if value[2..].parse::<f64>().is_ok(){
                Ok(())
            } else {
                Err(String::from("Invalid USDC format: correct example +$300"))
            }
        }else if value.ends_with("%pnl"){
                if value.trim_end_matches("%pnl").parse::<f64>().is_ok(){
                    Ok(())
                } else {
                    Err(String::from(" Invalid % pnl format: correct example: +30%pnl"))
                }
                       
        } else if value.ends_with("pnl"){
            if value.trim_end_matches("pnl").parse::<f64>().is_ok(){
                Ok(())
            }else{
                Err(String::from(" Invalid pnl format: correct example: +300pnl"))
            }
        } else {
            Err(String::from( "Invalid format: Expected tp format: (+10%, +$300, +300pnl + 34%pnl"))
        }
    } else{
        Err(String::from(" Invalid format: Expected tp format: (+10%, +$300, +300pnl + 34%pnl "))
    }
}

fn handle_risk_value(value: f64) {
    // Logic for handling risk value type
    println!("Handling risk value: {}", value);
}

fn handle_notional_value(value: f64) {
    // Logic for handling notional value type
    println!("Handling notional value: {}", value);
}

fn handle_isolated_margin(margin_type: &str){
    //logic handling Isolated margin
    println! ("Handling Isolated margin: {}", margin_type);
}

fn handle_cross_margin(margin_type: &str){
    //logic handling cross margin type
    println! ("Handling cross margin type: {}", margin_type);
}
