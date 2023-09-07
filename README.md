# hyperliquid

A bot for interacting with hyperliquid exchange.

Here is a list of the commands and the flags they support:

## Commands

#### Set Leverage

`cargo run -- set dl <leverage>` - sets the leverage for all the available assets on hyperliquid

leverage:[_required_] is a number between 1 and 100

example: `cargo run -- set dl 10`

#### Take Profit

`cargo run -- tp <size> <asset> <tp>` - sets a take profit order for the specified asset

size:[_required_] is the percentage of the order size

asset:[_required_] is the asset to set the take profit order for

tp:[_required_] is the take profit percentage or gain in asset before tp order is triggered

example: `cargo run -- tp 10% btc 5` - sets a take profit order for 10% of the order size for btc with a take profit of 5%

example: `cargo run -- tp 30% btc +$300` - sets a take profit order for 30% of the order size for btc with a take profit of $300

example `cargo run -- tp 100% sol +300pnl` - sets a take profit order for 100% of the order size for sol with a take profit of 300pnl

example `cargo run  --tp 50% sol +10%pnl` - sets a take profit order for 50% of the order size for sol with a take profit of 10% of the order size

#### Stop Loss

`cargo run -- sl <size> <asset> <sl>` - sets a stop loss order for the specified asset

size:[_required_] is the percentage of the order size

asset:[_required_] is the asset to set the stop loss order for

sl:[_required_] is the stop loss percentage or loss in asset before sl order is triggered

example: `cargo run -- sl 10% btc 5` - sets a stop loss order for 10% of the order size for btc with a stop loss of 5%

example: `cargo run -- sl 30% btc -$300` - sets a stop loss order for 30% of the order size for btc with a stop loss of $300

example `cargo run -- sl 100% sol -300pnl` - sets a stop loss order for 100% of the order size for sol with a stop loss of 300pnl

example `cargo run  --sl 50% sol -10%pnl` - sets a stop loss order for 50% of the order size for sol with a stop loss of 10% of the order size

#### Buy

`cargo run -- buy <size> <asset> <price>` - places a buy order for the specified asset

example: `cargo run -- buy` - places a buy order for 100% of the order size for btc at the current price

example: `cargo run -- buy $100` - places a buy order for $100 worth of btc at the current price

example: `cargo run -- buy eth` - places a buy order for 100% of the order size for eth at the current price

example: `cargo run --buy @1900` - places a buy order for 100% of the order size for btc at the price of $1900

example: `cargo run --buy $100 eth @1900` - places a buy order for $100 worth of eth at the price of $1900

example: `cargo run -- buy  $100 tp 1990 sl 1800` - places a buy order for $100 worth of btc at the current price with a take profit of $1990 and a stop loss of $1800

example: `cargo run -- buy  $100 eth tp +10% sl -1%pnl` - places a buy order for $100 worth of eth at the current price with a take profit of 10% and a stop loss of 1% of the order size

#### Sell

`cargo run -- sell <size> <asset> <price>` - places a sell order for the specified asset

example: `cargo run -- sell` - places a sell order for 100% of the order size for btc at the current price

example: `cargo run -- sell $100` - places a sell order for $100 worth of btc at the current price

example: `cargo run -- sell eth` - places a sell order for 100% of the order size for eth at the current price

example: `cargo run --sell @1900` - places a sell order for 100% of the order size for btc at the price of $1900

example: `cargo run --sell $100 eth @1900` - places a sell order for $100 worth of eth at the price of $1900

example: `cargo run -- sell  $100 tp 1990 sl 1800` - places a sell order for $100 worth of btc at the current price with a take profit of $1990 and a stop loss of $1800

example: `cargo run -- sell  $100 eth tp +10% sl -1%pnl` - places a sell order for $100 worth of eth at the current price with a take profit of 10% and a stop loss of 1% of the order size

#### Twap Buy

`cargo run -- twap buy <size> <asset> <time between interval in mins, number of intervals>` - Divides the total order size by the number of intervals. After the time between intervals, each piece of the divided order will be bought at market

example: `cargo run --twap buy 100 eth 5,10` - places a twap buy order for 100 usd. The order will be divided into 10 pieces and each piece will be bought every 5 minutes.






