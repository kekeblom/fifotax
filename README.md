# Usage

The project is written in the Rust programming language. You'll need to have rust installed https://www.rust-lang.org/en-US/install.html.

The input to the program is a csv file of your full transaction history. The format of the file is described below. Column names are case sensitive.

| Column  | Description |
| ------- | ----------- |
| Timestamp | The time at which the transaction was made. The format is ISO 8601 with a Z appended to the end. Example: "2014-02-21T15:43:30Z" |
| Ticker | The symbol of the currency that was bought or sold. E.g. "BTC" |
| TransactionType | "Buy" or "Sell". |
| Amount | The amount of currency that was bought or sold. If you bought 1 bitcoin the value is "1". |
| Subtotal | The price of the coins that were bought or sold. amount * price per coin. |
| Fees | The transaction fee. This will contribute towards the acquisition cost of the asset. |
| Total | Subtotal + Fees |
| Currency | What currency was used to buy the asset or if you sold, what currency you received in return. |
| PricePerCoin | The unit price of the coin transacted. |
| Id | Some arbitrary unique id for the transaction. |

The file `example.csv` shows an example of an input file.

Profits are calculated such that only when liquidating an asset to EUR will it be a taxable event. In case a cryptocurrency was bought using another cryptocurrency, the acquisition cost of the new currency is calculated in a first-in-first-out manner.

You can run the program with the command `cargo run --release transactions.csv`. It will output a file `out.csv` which will contain the realized profits. The output columns are:

| Column | Description |
| ------ | ----------- |
| Date | When the sale happened. |
| Currency | Which currency was sold. |
| AmountSold | How much currency was sold. |
| EurTotal | How much was received as part of the sale. |
| Cost | Calculated acquisition cost of all the currency sold. |

