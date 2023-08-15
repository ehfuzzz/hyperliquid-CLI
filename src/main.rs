//using version 2.33 not the latest one
use clap::{App, Arg};
use std::num::ParseFloatError;

#[tokio::main]
//clip logic to setup commands
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
        .subcommand(
            App::new("buy")
                .about(" Handles the Buy command")
                .arg(
                    Arg::with_name("order_size")
                        .help("size of the order e.g ., $100 ")
                        .long("size")
                        .takes_value(true)
                        .validator(|v| {
                            if v.starts_with('$') {
                                /*    If the parsing is successful, it returns Ok(()) (indicating success but discarding the float).
                                If the parsing fails, it returns an error message as a String. */                                
                                v[1..].parse::<f64>().map(|_| ()).map_err(|e| e.to_string())
                            }else {
                                Err(String::from("Expected a $ symbol at the start"))
                            }
                        }),
                )
                .arg(
                    Arg::with_name("asset_symbol")
                        .help("Asset symbol e.g ETH, SOL, BTC, optional if default asset is defined")
                        .long("symbol")
                        .takes_value(true),

                )
                .arg(
                    Arg::with_name("limit_price")
                        .help("Limit price e.g ., @1900")
                        .long("price")
                        .takes_value(true)
                        .validator(|v| {

                            if v.starts_with("@"){
                                /*    If the parsing is successful, it returns Ok(()) (indicating success but discarding the float).
                                    If the parsing fails, it returns an error message as a String. */
                                v[1..].parse::<f64>().map(|_| (())).map_err(|e| e.to_string())
                            }else{
                                Err(String::from("Expected an @ symbol at the start"))
                            }
                        })
                        
                )
                .arg(
                    Arg::with_name("take_profit")
                        .help("Take profit value")
                        .long("tp")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("stop_loss")
                        .help("Stop loss value")
                        .long("sl")
                        .takes_value(true),
                )
        )
        .subcommand(
            App::new("twap")
                .about("Handles the twap buy and twap sell logic")
                .subcommand(
                    App::new("buy")
                        .about("twap buy")
                        .arg(
                            Arg::with_name("order_size")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("Total Order Size")
                                .validator(|v| {
                                    if v.parse::<f64>().is_ok(){
                                        Ok(())
                                    }else {
                                        Err(String::from("Expected a numeric value"))
                                    }
                                }),
                        )
                        .arg(
                            Arg::with_name("asset_symbol")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .help("asset symbol to be traded"),
                        )
                        .arg(
                            Arg::with_name("interval")
                                .required(true)
                                .index(3)
                                .takes_value(true)
                                .help("Time between intervals in minutes, number of intervals e.g 10 means 10 minutes")
                        ),

                )
                .subcommand(
                    App::new("sell")
                        .about("twap sell")
                        .arg(
                            Arg::with_name("order_size")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("Total Order Size")
                                .validator(|v| {
                                    if v.parse::<f64>().is_ok(){
                                        Ok(())
                                    }else {
                                        Err(String::from("Expected a numeric value"))
                                    }
                                }),
                        )
                        .arg(
                            Arg::with_name("asset_symbol")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .help("asset symbol to be traded"),
                        )
                        .arg(
                            Arg::with_name("interval")
                                .required(true)
                                .index(3)
                                .takes_value(true)
                                .help("comma separated values of: Time between intervals in minutes, number of intervals e.g 10 means 10 minutes")
                        ),
                )
        )   

        .get_matches();


    // Conditional logic to handle the command choices from the user : we start with the set command, it has sub commands like ds, dm 
    if let Some(set_matches) = matches.subcommand_matches("set") {

        // handles the set ds <size type> <size in USDC OR size in % of balance> 
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

        // handles the set ds <size type> <size in USDC OR size in % of balance>             
        }else if let Some(dm_matches) = set_matches.subcommand_matches("dm") {
            let margin_type= dm_matches.value_of("margin_type").unwrap();
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

    // handles the tp <% of order to tp>  <asset symbol> <tp price or %/$ gain in asset before tp or %/$ gain in pnl before tp>
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
        
//handles the buy <order size> <asset symbol> <@limit price, if applicable> <sl if applicable> <tp if applicable> 
    }else if let Some(buy_matches) = matches.subcommand_matches("buy"){
        let buy_size = buy_matches.value_of("order_size");
        let asset_symbol = buy_matches.value_of("asset_symbol");
        let limit_price = buy_matches.value_of("limit_price");
        let take_profit = buy_matches.value_of("take_profit");
        let stop_loss = buy_matches.value_of("stop_loss");

        if let Some(size) = buy_size {
            //preprocess the String to get the numeric size
            let numeric_part = &size[1..].parse::<f64>().unwrap();
            println! ("Buy size: {}", numeric_part)
        }else{
            //Filled with the default size already set
            println! ("Filled with the default size already specified")
        }
        if let Some(symbol) = asset_symbol {
            println! ("Asset symbol: {}", symbol);
        }else{
            //Filled with the default size already set
            println! ("Filled with the default symbol already specified")            
        }
        if let Some(price) = limit_price {
            let numeric_part = &price[1..].parse::<f64>().unwrap();
            println! ("Limit price: {}", numeric_part);
        }else{
            //Filled with the default size already set
            println! ("Filled with the default limit rules already specified")            
        }
        if let Some(tp) = take_profit {
            let numeric_part = &tp.parse::<f64>().unwrap();
            println! ("Take profit: {}", numeric_part);
        }else{
            //Filled with the default size already set
            println! ("Filled with the default tp rules already specified")            
        }

        if let Some(sl) = stop_loss {
            let numeric_part = &sl.parse::<f64>().unwrap();
            println! ("Stop Loss: {}", numeric_part);
        }else{
            //Filled with the default size already set
            println! ("Filled with the default sl rules already specified")            
        }    

        
    }else if let Some(twap_matches) = matches.subcommand_matches("twap") {

        // twap buy <total order size> <asset symbol>  <time between interval in mins, number of intervals>
        if let Some(twapbuy_matches) = twap_matches.subcommand_matches("buy") {
            let order_size = twapbuy_matches.value_of("order_size").unwrap();
            let asset_symbol = twapbuy_matches.value_of("asset_symbol").unwrap();
            let intervals: Vec<&str> = twapbuy_matches.value_of("interval").unwrap().split(",").collect();

            

            println! ("twap sell order size: {}, asset-symbol: {}, intervals: {:?}-> Interval1: {:?}", order_size, asset_symbol, intervals, intervals.get(0));           

            
        //twap sell <total order size> <asset symbol>  <time between interval in mins, number of intervals>
        }else if let Some(twapsell_matches) = twap_matches.subcommand_matches("sell") {
            let order_size = twapsell_matches.value_of("order_size").unwrap();
            let asset_symbol = twapsell_matches.value_of("asset_symbol").unwrap();
            let intervals: Vec<&str> = twapsell_matches.value_of("interval").unwrap().split(",").collect();

            println! ("twap sell order size: {}, asset-symbol: {}, intervals: {:?}-> Interval1: {:?}", order_size, asset_symbol, intervals, intervals.get(0));
            

    
        }else{
            println! ("Invalid choice: we only have twap buy and twap sell");
        }
    }

}


//Helper functions to handle validation and logic: we start with one to validate the value size for some of the commands
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

// handles the validation for take profit price as it has a special case: starts with "+" character
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


//  handling logic for risk type
fn handle_risk_value(value: f64) {
    // Logic for handling risk value type
    println!("Handling risk value: {}", value);
}

//  handling logic for notional type
fn handle_notional_value(value: f64) {
    // Logic for handling notional value type
    println!("Handling notional value: {}", value);
}

//handling logic for implementing isolated margin
fn handle_isolated_margin(margin_type: &str){
    //logic handling Isolated margin
    println! ("Handling Isolated margin: {}", margin_type);
}

//handling logic for implementing cross margin type
fn handle_cross_margin(margin_type: &str){
    //logic handling cross margin type
    println! ("Handling cross margin type: {}", margin_type);
}
