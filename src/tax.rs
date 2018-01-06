
extern crate csv;
extern crate chrono;

use std::error::Error;
use std::fs::File;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use self::chrono::prelude::*;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Transaction {
    timestamp: DateTime<Utc>,
    ticker: String, // Id of currency purchased.
    transaction_type: String, // "Buy" or "Sell".
    amount: f64, // Amount purchased.
    subtotal: f64, // Amount * price per coin.
    fees: f64, // How much was payed in fees.
    total: f64, // Subtotal + fees.
    currency: String, // What currency the transaction was payed in.
    price_per_coin: f64, // Unit price of the purchased currency.
    id: String
}

impl Clone for Transaction {
    fn clone(&self) -> Transaction {
        Transaction {
            timestamp: self.timestamp.clone(),
            ticker: self.ticker.clone(),
            transaction_type: self.transaction_type.clone(),
            amount: self.amount,
            subtotal: self.subtotal,
            fees: self.fees,
            total: self.total,
            currency: self.currency.clone(),
            price_per_coin: self.price_per_coin.clone(),
            id: self.id.clone().clone()
        }
    }
}

pub mod transaction_types {
    pub const BUY: &'static str = "Buy";
    pub const SELL: &'static str = "Sell";
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct BalanceEntry {
    pub acquire_date: DateTime<Utc>,
    pub amount: f64,
    pub currency: String,
    pub cost: f64
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct RealizedProfit {
    pub date: DateTime<Utc>,
    pub currency: String,
    pub amount_sold: f64,
    pub eur_total: f64,
    pub cost: f64
}

impl RealizedProfit {
    #[allow(dead_code)]
    pub fn profit(&self) -> f64 {
        self.eur_total - self.cost
    }
}

fn read_transactions(file_path: &str) -> Result<Vec<Transaction>, Box<Error>> {
    let file = File::open(file_path)?;
    let mut reader = csv::Reader::from_reader(file);
    let mut transactions = Vec::new();
    for transaction in reader.deserialize() {
        transactions.push(transaction?);
    }
    Ok(transactions)
}

fn buy_cost(balances: &mut HashMap<String, Vec<BalanceEntry>>, transaction: &Transaction) -> f64 {
    let cost: f64;
    if transaction.currency == "EUR" {
        cost = transaction.total;
    } else {
        cost = deduct_balance(balances, &transaction.currency, transaction.total);
    }
    cost
}

fn adjust_balance_buy(balances: &mut HashMap<String, Vec<BalanceEntry>>, transaction: &Transaction) {
    let cost = buy_cost(balances, transaction);
    let balance_entry = BalanceEntry {
        acquire_date: transaction.timestamp.clone(),
        amount: transaction.amount,
        currency: transaction.ticker.clone(),
        cost: cost
    };
    match balances.entry(transaction.ticker.clone()) {
        Entry::Occupied(mut entry) => {
            let balances = entry.get_mut();
            balances.push(balance_entry);
        },
        Entry::Vacant(entry) => {
            entry.insert(vec![balance_entry]);
        }
    }
}

fn deduct_balance(balances: &mut HashMap<String, Vec<BalanceEntry>>, currency_spent: &String, spent: f64) -> f64 {
    let balance = balances.get_mut(currency_spent).unwrap();

    let total_balance_in_currency: f64 = balance.into_iter().map(|b| b.amount).sum();
    assert!(total_balance_in_currency >= spent, "We have a problem.");

    let mut amount_spent = spent;
    let mut total_cost = 0_f64;
    while amount_spent != 0_f64 {
        let balance_entry = balance.iter_mut().find(|b| {
            b.amount > 0_f64
        }).unwrap();
        if balance_entry.amount >= amount_spent {
            let to_deduct = amount_spent;
            let sold_fraction: f64 = to_deduct / balance_entry.amount;

            total_cost += sold_fraction * balance_entry.cost;
            balance_entry.cost = balance_entry.cost * (1_f64 - sold_fraction);
            // We deduct the amount we spent.
            balance_entry.amount = balance_entry.amount - amount_spent;

            amount_spent = 0_f64;
        } else {
            let to_deduct = balance_entry.amount;
            total_cost += balance_entry.cost;
            balance_entry.amount = 0_f64;
            balance_entry.cost = 0_f64;
            amount_spent -= to_deduct;
        }
    }
    total_cost
}

fn adjust_balance_sell(balances: &mut HashMap<String, Vec<BalanceEntry>>, transaction: &Transaction) {
    let amount_sold = transaction.amount;
    let amount_owned = balances.get(&transaction.ticker).unwrap().iter().map(|b| b.amount).sum();
    assert!(amount_sold <= amount_owned, "We have a problem");
    let amount_earned = transaction.subtotal;
    let currency_earned = transaction.currency.clone();
    let currency_sold = &transaction.ticker;
    let purchase_cost = deduct_balance(balances, currency_sold, amount_sold);
    let new_balance = BalanceEntry {
        acquire_date: transaction.timestamp.clone(),
        amount: amount_earned,
        currency: currency_earned,
        cost: purchase_cost + transaction.fees
    };
    balances.get_mut(&new_balance.currency).unwrap().push(new_balance);
}

fn calculate_profit(balances: &mut HashMap<String, Vec<BalanceEntry>>, transaction: &Transaction) -> RealizedProfit {
    let to_sell = transaction.amount;
    let acquisition_cost = deduct_balance(balances, &transaction.ticker, to_sell);

    RealizedProfit {
        date: transaction.timestamp.clone(),
        amount_sold: transaction.amount,
        currency: transaction.ticker.clone(),
        eur_total: transaction.subtotal,
        cost: acquisition_cost + transaction.fees
    }
}

fn profits_balances(transactions: &Vec<Transaction>) -> (Vec<RealizedProfit>, HashMap<String, Vec<BalanceEntry>>) {
    let mut balances: HashMap<String, Vec<BalanceEntry>> = HashMap::new();
    let mut profits: Vec<RealizedProfit> = Vec::new();
    for transaction in transactions.iter() {
        if transaction.transaction_type == transaction_types::BUY {
            adjust_balance_buy(&mut balances, transaction);
        } else if transaction.transaction_type == transaction_types::SELL {
            if transaction.currency == "EUR" {
                let realized_profit = calculate_profit(&mut balances, transaction);
                profits.push(realized_profit);
            } else {
                adjust_balance_sell(&mut balances, transaction);
            }
        }
    }
    (profits, balances)
}

pub fn calculate_profits_balances(file_path: &str) -> (Vec<RealizedProfit>, HashMap<String, Vec<BalanceEntry>>) {
    let mut transactions = read_transactions(file_path).expect("Can't read transactions");
    transactions.sort_by(|t1, t2| t1.timestamp.cmp(&t2.timestamp) );
    profits_balances(&transactions)
}

pub fn write_profits_to_file(profits: &Vec<RealizedProfit>, out_file: &str) -> Result<(), Box<Error>> {
    let file = File::create(out_file)?;
    let mut writer = csv::Writer::from_writer(file);
    for profit in profits.iter() {
        writer.serialize(profit)?
    }
    Ok(())
}

pub fn write_balances_to_file(balances: &HashMap<String, Vec<BalanceEntry>>, file: &str) -> Result<(), Box<Error>> {
    let file = File::create(file)?;
    let mut writer = csv::Writer::from_writer(file);
    let mut non_zero_balances: Vec<&BalanceEntry> = Vec::new();
    for balance in balances.values() {
        let mut non_zero = balance.iter().filter(|b| b.amount != 0_f64).collect::<Vec<_>>();
        non_zero_balances.append(&mut non_zero);
    }
    non_zero_balances.sort_by(|b1, b2| b1.acquire_date.cmp(&b2.acquire_date));
    for entry in non_zero_balances.iter() {
        writer.serialize(entry)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_sell_transactions() {
        let transactions = vec![Transaction {
            timestamp: Utc::now(),
            ticker: String::from("BTC"),
            transaction_type: String::from(transaction_types::BUY),
            amount: 1000_f64,
            subtotal: 1000_f64,
            fees: 1_f64,
            total: 1001_f64,
            currency: String::from("EUR"),
            price_per_coin: 1_f64,
            id: String::from("1")
        }];
        let (profits, balances) = profits_balances(&transactions);
        assert!(profits.len() == 0);
        let btc_balance = balances.get("BTC").unwrap();
        assert!(btc_balance.len() == 1);
        assert!(btc_balance[0].amount == 1000_f64);
    }

    #[test]
    fn two_purchases_one_sale() {
        let transactions = vec![Transaction {
            timestamp: Utc::now(),
            ticker: String::from("BTC"),
            transaction_type: String::from(transaction_types::BUY),
            amount: 1000_f64,
            subtotal: 500_f64,
            fees: 0_f64,
            total: 500_f64,
            currency: String::from("EUR"),
            price_per_coin: 0.5_f64,
            id: String::from("1")
        }, Transaction {
            timestamp: Utc::now(),
            ticker: String::from("BTC"),
            transaction_type: String::from(transaction_types::BUY),
            amount: 500_f64,
            subtotal: 500_f64,
            fees: 0_f64,
            total: 500_f64,
            currency: String::from("EUR"),
            price_per_coin: 1_f64,
            id: String::from("1")
        }, Transaction {
            timestamp: Utc::now(),
            ticker: String::from("BTC"),
            transaction_type: String::from(transaction_types::SELL),
            amount: 1200_f64,
            subtotal: 1800_f64,
            fees: 0_f64,
            total: 18000_f64,
            currency: String::from("EUR"),
            price_per_coin: 1_f64,
            id: String::from("1")
        }];
        let (profits, balances) = profits_balances(&transactions);
        assert!(profits.len() == 1);
        let btc_balance = balances.get("BTC").unwrap();
        let sum: f64 = btc_balance.iter().map(|b| b.amount).sum();
        assert_eq!(sum, 300_f64);
        assert_eq!(btc_balance[1].cost, 300_f64);
        let profit = &profits[0];
        assert_eq!(profit.eur_total, 1800_f64);
        assert_eq!(profit.cost, 500_f64 + 200_f64);
        println!("profit: {}", profit.profit());
        assert_eq!(profit.profit(), 1100_f64);
    }

    #[test]
    fn test_deduct_balance() {
        let btc = String::from("BTC");
        let btc_balances = vec![BalanceEntry {
                acquire_date: Utc::now(),
                currency: btc.clone(),
                amount: 1000_f64,
                cost: 300_f64
            }, BalanceEntry {
                acquire_date: Utc::now(),
                currency: btc.clone(),
                amount: 200_f64,
                cost: 300_f64
            }];
        let mut balances = HashMap::new();
        balances.insert(btc.clone(), btc_balances);
        let cost = deduct_balance(&mut balances, &btc, 1100_f64);
        assert_eq!(cost, 450_f64);
    }

}
