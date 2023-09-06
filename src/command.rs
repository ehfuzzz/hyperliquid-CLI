//using version 2.33 not the latest one
use clap::{Arg, Command};

pub fn command() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("A CLI bot to interact with the hyperliquid exchange")
        .subcommand(
            Command::new("set")
                .about("Sets the default values for the bot in the exchange")
                .subcommand(
                    Command::new("dl")
                        .about("Sets the default leverage")
                        .arg(
                            Arg::new("leverage")
                                .index(1)
                                .required(true)
                                .help("Default leverage value")
                        )
                )
            )
        .subcommand(
            Command::new("tp")
                .about("Takes profit on open order as a market order")
                .arg(
                    Arg::new("size")
                        .required(true)
                        .index(1)
                        .help("% of order to tp")
                )
                .arg(
                    Arg::new("asset")
                        .required(true)
                        .index(2)
                        .help(
                            "Asset symbol e.g ETH, SOL, BTC ... optional if default asset is provided"
                        )
                )
                .arg(
                    Arg::new("tp")
                        .required(true)
                        .index(3)
                        .help(
                            "Take profit price or %/$ gain in asset before tp or %$ gain in pnl before tp"
                        )
                )
        )
        .subcommand(
            Command::new("sl")
                .about("Stop loss on an open order as market order")
                .arg(
                    Arg::new("size")
                        .required(true)
                        .index(1)
                        .help("% of order to sl")
                )
                .arg(
                    Arg::new("asset")
                        .required(true)
                        .index(2)
                        .help(
                            "asset symbol e.g ETH, SOL, BTC .., optional if default asset is provided"
                        )
                )
                .arg(
                    Arg::new("sl")
                        .required(true)
                        .index(3)
                        .help(
                            "Stop loss price or %/$ loss in asset before sl or %$ loss in pnl before sl"
                        )
                )
        )
        .subcommand(
            Command::new("buy")
                .about("Buys an asset at market or limit price")
                .arg(
                    Arg::new("size")
                        .help("size of the order e.g, $100 ")
                        .long("size")
                )
                .arg(
                    Arg::new("asset")
                        .help(
                            "Asset symbol e.g ETH, SOL, BTC, optional if default asset is defined"
                        )
                        .long("asset")
                )
                .arg(
                    Arg::new("price")
                        .help("Limit price e.g ., @1900")
                        .long("price")
                )
                .arg(
                    Arg::new("tp")
                        .help("Take profit value")
                        .long("tp")
                )
                .arg(
                    Arg::new("sl")
                        .help("Stop loss value")
                        .long("sl")
                )
        )
        .subcommand(
            Command::new("sell")
                .about("Sells an asset at market or limit price")
                .arg(
                    Arg::new("size")
                        .help("size of the order e.g ., $100 ")
                        .long("size")
                )
                .arg(
                    Arg::new("asset")
                        .help(
                            "Asset symbol e.g ETH, SOL, BTC, optional if default asset is defined"
                        )
                        .long("asset")
                )
                .arg(
                    Arg::new("price")
                        .help("Limit price e.g ,. @1900")
                        .long("price")
                )
                .arg(
                    Arg::new("tp")
                        .help("Take profit value")
                        .long("tp")
                )
                .arg(
                    Arg::new("sl")
                        .help("Stop loss value")
                        .long("sl")
                )
        )
        .subcommand(
            Command::new("twap")
                .about("Divides the total order size by the number of intervals. After the time between intervals, each piece of the divided order will be bought at market")
                .subcommand(
                    Command::new("buy")
                        .about("twap buy")
                        .arg(
                            Arg::new("size")
                                .required(true)
                                .index(1)
                                .help("Total order size")
                        )
                        .arg(
                            Arg::new("asset")
                                .required(true)
                                .index(2)
                                .help("asset to be traded")
                        )
                        .arg(
                            Arg::new("interval")
                                .required(true)
                                .index(3)
                                .help(
                                    "Time between intervals in minutes, number of intervals e.g 5,10"
                                )
                        )
                )
                .subcommand(
                    Command::new("sell")
                        .about("twap sell")
                        .arg(
                            Arg::new("size")
                                .required(true)
                                .index(1)
                                .help("Total order size")
                        )
                        .arg(
                            Arg::new("asset")
                                .required(true)
                                .index(2)
                                .help("asset to be traded")
                        )
                        .arg(
                            Arg::new("interval")
                                .required(true)
                                .index(3)
                                .help(
                                    "Time between intervals in minutes, number of intervals e.g 5,10"
                                )
                        )
                )
        )
        .subcommand(
            Command::new("view")
                .about("Handles the view commands")
                .subcommand(
                    Command::new("upnl").about("view unrealized pnl")
                )
                .subcommand(
                    Command::new("wallet")
                        .about("view wallet balance")
                        .arg(
                            Arg::new("balance")
                                .required(true)
                                .index(1)
                                .help("argument to complete the view wallet balance command")
                            
                        )
                )
                .subcommand(
                    Command::new("unfilled")
                        .about("view unfilled orders")
                        .arg(
                            Arg::new("orders")
                                .required(true)
                                .index(1)
                                .help("argument to complete the view unfilled orders command")
                            
                        )
                )
                .subcommand(
                    Command::new("open")
                        .about("view open positions")
                        .arg(
                            Arg::new("positions")
                                .required(true)
                                .index(1)
                                .help("argument to complete the view open positions command")
                            
                        )
                )
        )
        .subcommand(
            Command::new("pair")
                .about("Takes 50% of order size and longs Asset X and takes another 50% of order size and shorts Asset Y.")
                .subcommand(
                    Command::new("buy")
                        .about("pair to buy")
                        .arg(
                            Arg::new("size")
                                .required(true)
                                .index(1)
                                .help("Order Size")
                        )
                        .arg(
                            Arg::new("pair")
                                .required(true)
                                .index(2)
                                .help("Asset X/Asset Y e.g BTC/SOL")
                        )
                        .arg(
                            Arg::new("price")
                                .required(false)
                                .long("price")
                                .help("Limit price if Commandlicable")
                        )
                        .arg(
                            Arg::new("sl")
                                .required(false)
                                .long("sl")
                                .help("Stop loss")
                        )
                        .arg(
                            Arg::new("tp")
                                .required(false)
                                .long("tp")
                                .help("Take profit")
                        )
                )
                .subcommand(
                    Command::new("sell")
                        .about("pair to sell")
                        .arg(
                            Arg::new("size")
                                .required(true)
                                .index(1)
                                .help("Order Size")
                        )
                        .arg(
                            Arg::new("pair")
                                .required(true)
                                .index(2)
                                .help("Asset X/Asset Y e.g BTC/SOL")
                        )
                        .arg(
                            Arg::new("price")
                                .long("price")
                                .required(false)
                                .help("Limit price if Commandlicable")
                        )
                        .arg(
                            Arg::new("sl")
                                .required(false)
                                .long("sl")
                                .help("stop loss")
                        )
                        .arg(
                            Arg::new("tp")
                                .required(false)
                                .long("tp")
                                .help("Take profit")
                        )
                )
        )
        .subcommand(
            Command::new("scale")
                .about("Divides the total order size by the number of intervals. After the time between intervals, each piece of the divided order will be bought at market")
                .subcommand(
                    Command::new("buy")
                        .about("scale buy")
                        .arg(
                            Arg::new("size_per_interval")
                                .required(true)
                                .index(1)
                                .help(
                                    "total order size/number of intervals"
                                )
                        )
                        .arg(
                            Arg::new("asset")
                                .required(true)
                                .index(2)
                                .help("asset e.g ETH, SOL, BTC")
                        )
                        .arg(
                            Arg::new("lower")
                                .required(true)
                                .index(3)
                                .help("lower price bracket")
                        )
                        .arg(
                            Arg::new("upper")
                                .required(true)
                                .index(4)
                                .help("upper price bracket")
                        )
                )
                .subcommand(
                    Command::new("sell")
                        .about("Divides the total order size by the number of intervals. After the time between intervals, each piece of the divided order will be bought at market")
                        .arg(
                            Arg::new("size_per_interval")
                                .required(true)
                                .index(1)
                                .help(
                                    "total order size/number of intervals"
                                )
                        )
                        .arg(
                            Arg::new("asset")
                                .required(true)
                                .index(2)
                                .help("asset e.g ETH, SOL, BTC")
                        )
                        .arg(
                            Arg::new("lower")
                                .required(true)
                                .index(3)
                                .help("Lower price bracket")
                        )
                        .arg(
                            Arg::new("upper")
                                .required(true)
                                .index(4)
                                .help("Upper price bracket")
                        )
                )
        )
}
