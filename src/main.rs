//using version 2.33 not the latest one
use clap::{App, Arg};
use regex::Regex;


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
                    App::new("dl")
                        .about("Sets the default leverage")
                        .arg(
                            Arg::with_name("amount")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("Amount of leverage")
                                .validator(|v| {
                                    if v.parse::<f64>().is_ok(){
                                        Ok(())
                                    }else {
                                        Err(String::from("Expected a numeric value"))
                                    }
                                }),
                        )


                ) 

                .subcommand(
                    App::new("da")
                        .about("Sets the default instrument to trade")
                        .arg(
                            Arg::with_name("asset_symbol")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("asset symbol to be traded")
                                .validator(|v| {
                                    if Regex::new(r"^[a-zA-Z]+$").unwrap().is_match(&v){
                                        Ok(())
                                    }else {
                                        Err(String::from("Expected alphabetic characters"))
                                    }
                                }),     
                        )


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
                        .takes_value(true)
                        .validator(|v| {
                            if Regex::new(r"^[a-zA-Z]+$").unwrap().is_match(&v){
                                Ok(())
                            }else {
                                Err(String::from("Expected alphabetic characters"))
                            }
                        }),                           

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
                                .help("asset symbol to be traded")
                                .validator(|v| {
                                    if Regex::new(r"^[a-zA-Z]+$").unwrap().is_match(&v){
                                        Ok(())
                                    }else {
                                        Err(String::from("Expected alphabetic characters"))
                                    }
                                }),                                   
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
                                .help("asset symbol to be traded")
                                .validator(|v| {
                                    if Regex::new(r"^[a-zA-Z]+$").unwrap().is_match(&v){
                                        Ok(())
                                    }else {
                                        Err(String::from("Expected alphabetic characters"))
                                    }
                                }),   
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
        .subcommand(
            App::new("view")
                .about("Handles the view commands")
                .subcommand(
                    App::new("pnl")
                        .about("view pnl")
                        .help("Use to display the account's PNL")

                )
                .subcommand(
                    App::new("wallet")
                        .about("view wallet balance")
                        .arg(
                            Arg::with_name("balance")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("argument to complete the view wallet balance command")
                                .possible_values(&["balance"])
                        )

                )
                .subcommand(
                    App::new("unfilled")
                        .about("view unfilled orders")
                        .arg(
                            Arg::with_name("orders")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("argument to complete the view unfilled orders command")
                                .possible_values(&["orders"])
                        )

                ) 
                .subcommand(
                    App::new("open")
                        .about("view open positions")
                        .arg(
                            Arg::with_name("positions")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("argument to complete the view open positions command")
                                .possible_values(&["positions"])
                        )

                )                               
        )

        .subcommand(
            App::new("pair")
                .about("Handles the pair buy and pair sell logic")
                .subcommand(
                    App::new("buy")
                        .about("pair buy")
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
                            Arg::with_name("asset_symbols")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .help("forward slash separated assets symbol to be pair traded"),
                        )
                        .arg(
                            Arg::with_name("limit_price")
                                .required(false)
                                .index(3)
                                .takes_value(true)
                                .help("Limit price if applicable ")
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
                            Arg::with_name("stop_loss")
                                .required(false)
                                .index(4)
                                .takes_value(true)
                                .help("stop loss if applicable")
                                .validator(|v| {
                                    if v.parse::<f64>().is_ok(){
                                        Ok(())
                                    }else {
                                        Err(String::from("Expected a numeric value"))
                                    }
                                })
                        )
                        .arg(
                            Arg::with_name("take_profit")
                                .required(false)
                                .index(5)
                                .takes_value(true)
                                .help("Take profit if applicable")
                                .validator(|v| {
                                    if v.parse::<f64>().is_ok(){
                                        Ok(())
                                    }else {
                                        Err(String::from("Expected a numeric value"))
                                    }
                                })

                        )

                )
                .subcommand(
                    App::new("sell")
                        .about("pair sell")
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
                            Arg::with_name("asset_symbols")
                                .required(true)
                                .index(2)
                                .takes_value(true)
                                .help("forward slash separated assets symbol to be pair traded")
                                .validator(|v| {
                                    if Regex::new(r"^[a-zA-Z]+$").unwrap().is_match(&v){
                                        Ok(())
                                    }else {
                                        Err(String::from("Expected alphabetic characters"))
                                    }
                                }),                                   
                        )
                        .arg(
                            Arg::with_name("limit_price")
                                .required(false)
                                .index(3)
                                .takes_value(true)
                                .help("Limit price if applicable ")
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
                            Arg::with_name("stop_loss")
                                .required(false)
                                .index(4)
                                .takes_value(true)
                                .help("stop loss if applicable")
                                .validator(|v| {
                                    if v.parse::<f64>().is_ok(){
                                        Ok(())
                                    }else {
                                        Err(String::from("Expected a numeric value"))
                                    }
                                })
                        )
                        .arg(
                            Arg::with_name("take_profit")
                                .required(false)
                                .index(5)
                                .takes_value(true)
                                .help("Take profit if applicable")
                                .validator(|v| {
                                    if v.parse::<f64>().is_ok(){
                                        Ok(())
                                    }else {
                                        Err(String::from("Expected a numeric value"))
                                    }
                                })

                        )

                )
        )             

        .get_matches();


    // Conditional logic to handle the command choices from the user : we start with the set command, it has sub commands like ds, dm 
    if let Some(set_matches) = matches.subcommand_matches("set") {

        if let Some(da_match) = set_matches.subcommand_matches("da"){
            let asset_symbol = da_match.value_of("asset_symbol").unwrap();
            println! ("You have set {} as your default asset to be traded", asset_symbol)
        }else if let Some(dl_match) = set_matches.subcommand_matches("dl"){
            let leverage = dl_match.value_of("amount").unwrap().parse::<f64>().unwrap();
            println! ("You have set {} as your default leverage size", leverage);
        }

    // handles the tp <% of order to tp>  <asset symbol> <tp price or %/$ gain in asset before tp or %/$ gain in pnl before tp>
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
            let order_size = twapbuy_matches.value_of("order_size").unwrap().parse::<f64>().unwrap();
            let asset_symbol = twapbuy_matches.value_of("asset_symbol").unwrap();
            let intervals: Vec<&str> = twapbuy_matches.value_of("interval").unwrap().split(",").collect();

            

            println! ("twap sell order size: {}, asset-symbol: {}, intervals: {:?}-> Interval1: {:?}", order_size, asset_symbol, intervals, intervals.get(0));           

            
        //twap sell <total order size> <asset symbol>  <time between interval in mins, number of intervals>
        }else if let Some(twapsell_matches) = twap_matches.subcommand_matches("sell") {
            let order_size = twapsell_matches.value_of("order_size").unwrap().parse::<f64>().unwrap();
            let asset_symbol = twapsell_matches.value_of("asset_symbol").unwrap();
            let intervals: Vec<&str> = twapsell_matches.value_of("interval").unwrap().split(",").collect();

            println! ("twap sell order size: {}, asset-symbol: {}, intervals: {:?}-> Interval1: {:?}", order_size, asset_symbol, intervals, intervals.get(0));
            

    
        }else{
            println! ("Invalid choice: we only have twap buy and twap sell");
        }
    }
    else if let Some(view_matches) = matches.subcommand_matches("view"){

        match view_matches.subcommand_name(){
            Some("pnl") => {
                println! ("Implement view pnl logic");
            }
            Some("wallet") => {
                println!("Implement view wallet balance logic");
            }
            Some("unfilled") => {
                println! ("Implement view unfilled orders logic");
            }
            Some("open") => {
                println! ("Implement view open positions logic")
            }
            _=> {
                println! (" Invalid command: expected commands: (view pnl, view wallet balance, view unfilled orders, view open positions");
            }
        }

    }
    
    else if let Some(pair_matches) = matches.subcommand_matches("pair"){

        //pair buy <Order Size> <Asset X/Asset Y> <@limit price, if applicable> <sl if applicable> <tp if applicable> 

        if let Some(pairbuy_matches) = pair_matches.subcommand_matches("buy"){
            let order_size = pairbuy_matches.value_of("order_size").unwrap().parse::<f64>().unwrap();
            let asset_symbols: Vec<&str> = pairbuy_matches.value_of("asset_symbols").unwrap().split("/").collect();
            let limit_price = pairbuy_matches.value_of("limit_price");
            let stop_loss = pairbuy_matches.value_of("stop_loss");
            let take_profit = pairbuy_matches.value_of("take_profit");

            println! ("pair buy order size: {}, asset_symbols: {:?}, asset_1: {:?}, asset_2: {:?}", 
            order_size,asset_symbols, asset_symbols.get(0), asset_symbols.get(1));  

            if let Some(lp) = limit_price{
                println! ("Limit price provided: {}", lp);
            }else{
                println! (" The already set default limit price rules will be used");
            }
            if let Some(sl) = stop_loss{
                println! ("Stop loss provided: {}", sl);
            }else{
                println! (" The already set stop loss rules will be used");
            }
            if let Some(tp) = take_profit{
                println! ("Take profit provided: {}", tp);
            }else{
                println! (" The already set default take profit rules will be used");
            }                        


        //pair sell <Order Size> <Asset X/Asset Y> <@limit price, if applicable> <sl if applicable> <tp if applicable> 
        } else if let Some(pairsell_matches) = pair_matches.subcommand_matches("sell"){
            let order_size = pairsell_matches.value_of("order_size").unwrap().parse::<f64>().unwrap();
            let asset_symbols: Vec<&str> = pairsell_matches.value_of("asset_symbols").unwrap().split("/").collect();
            let limit_price = pairsell_matches.value_of("limit_price");
            let stop_loss = pairsell_matches.value_of("stop_loss");
            let take_profit = pairsell_matches.value_of("take_profit");     

            println! ("pair sell order size: {}, asset_symbols: {:?}, asset_1: {:?}, asset_2: {:?}", 
            order_size,asset_symbols, asset_symbols.get(0), asset_symbols.get(1));  


            if let Some(lp) = limit_price{
                println! ("Limit price provided: {}", lp);
            }else{
                println! (" The already set default limit price rules will be used");
            }
            if let Some(sl) = stop_loss{
                println! ("Stop loss provided: {}", sl);
            }else{
                println! (" The already set stop loss rules will be used");
            }
            if let Some(tp) = take_profit{
                println! ("Take profit provided: {}", tp);
            }else{
                println! (" The already set default take profit rules will be used");
            }            
        }else{
            println! ("Invalid Pair command: We only have pair buy and pair sell");
        }
    }

}


