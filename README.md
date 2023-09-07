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





