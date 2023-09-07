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

example: `cargo run -- tp 10% btc 5.5`
