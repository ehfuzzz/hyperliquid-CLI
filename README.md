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

example `cargo run  -- tp 50% sol +10%pnl` - sets a take profit order for 50% of the order size for sol with a take profit of 10% of the order size

#### Stop Loss

`cargo run -- sl <size> <asset> <sl>` - sets a stop loss order for the specified asset

size:[_required_] is the percentage of the order size

asset:[_required_] is the asset to set the stop loss order for

sl:[_required_] is the stop loss percentage or loss in asset before sl order is triggered

example: `cargo run -- sl 10% btc 5` - sets a stop loss order for 10% of the order size for btc with a stop loss of 5%

example: `cargo run -- sl 30% btc -$300` - sets a stop loss order for 30% of the order size for btc with a stop loss of $300

example `cargo run -- sl 100% sol -300pnl` - sets a stop loss order for 100% of the order size for sol with a stop loss of 300pnl

example `cargo run  -- sl 50% sol -10%pnl` - sets a stop loss order for 50% of the order size for sol with a stop loss of 10% of the order size

#### Buy

`cargo run -- buy --size <size> --asset <asset> --price <price> --sl <sl> --tp <tp>` - places a buy order for the specified asset

size:[_optional_] is the order size. If not specified, the default order size in the config will be used

asset:[_optional_] is the asset to sell. If not specified, the default asset in the config will be used


price:[_optional_] is the price to buy the asset at. If not specified, the current market price will be used but if provided the a limit order will be placed at the specified price

sl:[_optional_] is the price to set the stop loss at. If not specified, the stop loss will not be set. Can be a percentage or a pnl amount or the exact price

tp:[_optional_] is the price to set the take profit at. If not specified, the take profit will not be set. Can be a percentage or a pnl amount or the exact price

example 1: `cargo run -- buy --size 100 --asset btc --price 1900 --sl 1800 --tp 2000` - places a buy order for 100 usd worth of btc at the price of $1900. The stop loss will be set at $1800 and the take profit will be set at $2000

example 2: `cargo run -- buy --size 100 --asset btc --price 1900` - places a buy order for 100 usd worth of btc at the price of $1900. The stop loss and take profit will not be set

example 3: `cargo run -- buy --size 100 --asset btc` - places a buy order for 100 usd worth of btc at the current market price. The stop loss and take profit will not be set

#### Sell

`cargo run -- sell --size <size> --asset <asset> --price <price> --sl <sl> --tp <tp>` - places a sell order for the specified asset

size:[_optional_] is the order size. If not specified, the default order size in the config will be used

asset:[_optional_] is the asset to sell. If not specified, the default asset in the config will be used

price:[_optional_] is the price to sell the asset at. If not specified, the current market price will be used but if provided the a limit order will be placed at the specified price

sl:[_optional_] is the price to set the stop loss at. If not specified, the stop loss will not be set. Can be a percentage or a pnl amount or the exact price

tp:[_optional_] is the price to set the take profit at. If not specified, the take profit will not be set. Can be a percentage or a pnl amount or the exact price

example 1: `cargo run -- sell --size 100 --asset btc --price 1900 --sl 1800 --tp 2000` - places a sell order for 100 usd worth of btc at the price of $1900. The stop loss will be set at $1800 and the take profit will be set at $2000

example 2: `cargo run -- sell --size 100 --asset btc --price 1900` - places a sell order for 100 usd worth of btc at the price of $1900. The stop loss and take profit will not be set

example 3: `cargo run -- sell --size 100 --asset btc` - places a sell order for 100 usd worth of btc at the current market price. The stop loss and take profit will not be set

#### Twap Buy

`cargo run -- twap buy <size> <asset> <time between interval in mins, number of intervals>` - Divides the total order size by the number of intervals. After the time between intervals, each piece of the divided order will be bought at market

example: `cargo run -- twap buy 100 eth 5,10` - places a twap buy order for 100 usd. The order will be divided into 10 pieces and each piece will be bought every 5 minutes.


#### Twap Sell

`cargo run -- twap sell <size> <asset> <time between interval in mins, number of intervals>` - Divides the total order size by the number of intervals. After the time between intervals, each piece of the divided order will be sold at market

example: `cargo run -- twap sell 100 eth 5,10` - places a twap sell order for 100 usd. The order will be divided into 10 pieces and each piece will be sold every 5 minutes.

#### Pair Buy
`cargo run -- pair buy <size> <pair> --price <price> --sl <sl> --tp <tp>` - Takes 50% of order size and longs Asset X and takes another 50% of order size and shorts Asset Y in a pair

size:[_required_] is the order size to be split equally between the long and short

pair:[_required_] is the pair to long and short i.e Asset X/Asset Y

price:[_optional_] is the ratio of the market price of Asset X/Asset Y to enter the trade at. If not specified, the current market price will be used but if provided the ratio will be used to calculate when to enter the trade

sl:[_optional_] is the ratio of the market price of Asset X/Asset Y to set the stop loss at. 

tp:[_optional_] is the ratio of the market price of Asset X/Asset Y to set the take profit at.

example 1: `cargo run -- pair buy 100 btc/eth --price 0.05 --sl 0.04 --tp 0.06` - Takes 50% of order size and longs btc and takes another 50% of order size and shorts eth in a pair. The ratio of btc/eth is 0.05 so the bot will enter the trade when the ratio is 0.05. The stop loss will be set at 0.04 and the take profit will be set at 0.06

example 2: `cargo run -- pair buy 100 btc/eth` - Takes 50% of order size and longs btc and takes another 50% of order size and shorts eth in a pair. The current market price of btc/eth will be used to enter the trade. The stop loss and take profit will not be set

example 3: `cargo run -- pair buy 100 btc/eth --price 0.05` - Takes 50% of order size and longs btc and takes another 50% of order size and shorts eth in a pair. The ratio of btc/eth is 0.05 so the bot will enter the trade when the ratio is 0.05. The stop loss and take profit will not be set

#### Pair Sell
`cargo run -- pair sell <size> <pair> --price <price> --sl <sl> --tp <tp>` - Takes 50% of order size and shorts Asset X and takes another 50% of order size and longs Asset Y in a pair

size:[_required_] is the order size to be split equally between the long and short

pair:[_required_] is the pair to long and short i.e Asset X/Asset Y

price:[_optional_] is the ratio of the market price of Asset X/Asset Y to enter the trade at. If not specified, the current market price will be used but if provided the ratio will be used to calculate when to enter the trade

sl:[_optional_] is the ratio of the market price of Asset X/Asset Y to set the stop loss at.

tp:[_optional_] is the ratio of the market price of Asset X/Asset Y to set the take profit at.

example 1: `cargo run -- pair sell 100 btc/eth --price 0.05 --sl 0.04 --tp 0.06` - Takes 50% of order size and shorts btc and takes another 50% of order size and longs eth in a pair. The ratio of btc/eth is 0.05 so the bot will enter the trade when the ratio is 0.05. The stop loss will be set at 0.04 and the take profit will be set at 0.06

example 2: `cargo run -- pair sell 100 btc/eth` - Takes 50% of order size and shorts btc and takes another 50% of order size and longs eth in a pair. The current market price of btc/eth will be used to enter the trade. The stop loss and take profit will not be set

example 3: `cargo run -- pair sell 100 btc/eth --price 0.05` - Takes 50% of order size and shorts btc and takes another 50% of order size and longs eth in a pair. The ratio of btc/eth is 0.05 so the bot will enter the trade when the ratio is 0.05. The stop loss and take profit will not be set

#### Scale Buy
`cargo run -- scale buy <size_per_interval> <asset> <lower> <upper> ` - Scales into a long position by placing limit orders at intervals between the lower and upper price

size_per_interval:[_required_] is a forward slash separated string of the size of each interval and the number of intervals i.e 100/10.

asset:[_required_] is the asset to scale buy

lower:[_required_] is the lower price to place the limit orders at

upper:[_required_] is the upper price to place the limit orders at

example: `cargo run -- scale buy 100/10 eth 1800 1890` - Scales into a long position by placing limit orders at intervals between 1800 and 1890. 100 eth will be divided over 10 orders (10eth per order). Since there are 10 number of intervals, there will be 10 buy orders placed between 1800 and 1890:

#### Scale Sell
`cargo run -- scale sell <size_per_interval> <asset> <lower> <upper> ` - Scales into a short position by placing limit orders at intervals between the lower and upper price

size_per_interval:[_required_] is a forward slash separated string of the size of each interval and the number of intervals i.e 100/10.

asset:[_required_] is the asset to scale sell

lower:[_required_] is the lower price to place the limit orders at

upper:[_required_] is the upper price to place the limit orders at

example: `cargo run -- scale sell 100/10 eth 1800 1890` - Scales into a short position by placing limit orders at intervals between 1800 and 1890. 100 eth will be divided over 10 orders (10eth per order). Since there are 10 number of intervals, there will be 10 sell orders placed between 1800 and 1890:

#### View pnl
`cargo run -- view upnl` - View the current unrealized pnl

#### View wallet balance
`cargo run -- view wallet balance` - View the current wallet balance

#### View Unfilled Orders
`cargo run -- view unfilled orders` - View the current unfilled orders

#### View Open Positions
`cargo run -- view open positions` - View the current open positions






