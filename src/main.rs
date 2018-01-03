extern crate csv;
extern crate chrono;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::env;
use std::error::Error;
use std::fs::File;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use chrono::prelude::*;

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

pub mod transaction_types {
    pub const BUY: &'static str = "Buy";
    pub const SELL: &'static str = "Sell";
    pub const TX: &'static str = "Tx";
}

fn read_arguments() -> Result<String, Box<Error>> {
    match env::args().nth(1) {
        None => Err(From::from("expected a file name as argument")),
        Some(file_path) => Ok(file_path)
    }
}

fn read_transactions() -> Result<Vec<Transaction>, Box<Error>> {
    let file_path = read_arguments()?;
    let file = File::open(file_path)?;
    let mut reader = csv::Reader::from_reader(file);
    let mut transactions = Vec::new();
    for transaction in reader.deserialize() {
        transactions.push(transaction?);
    }
    Ok(transactions)
}

fn process_buy(balances: &mut HashMap<String, f64>, transaction: Transaction) {
    let currency = transaction.currency;
    let total = transaction.total;
    let ticker = transaction.ticker;
    let amount = transaction.amount;
    assert!(amount > 0_f64);
    // Deduct total value
    match balances.entry(currency) {
        Entry::Vacant(entry) => {
            entry.insert(-total);
        },
        Entry::Occupied(mut entry) => {
            let mut value = entry.get_mut();
            *value = *value - total;
        }
    }
    // Add purchased amount to balances.
    match balances.entry(ticker) {
        Entry::Vacant(entry) => {
            entry.insert(amount);
        },
        Entry::Occupied(mut entry) => {
            let mut value = entry.get_mut();
            *value = *value + amount;
        }
    }
}

fn process_sell(balances: &mut HashMap<String, f64>, transaction: Transaction) {
    // Add currency to balance.
    match balances.entry(transaction.currency) {
        Entry::Vacant(entry) => {
            entry.insert(transaction.subtotal);
        },
        Entry::Occupied(mut entry) => {
            let mut value = entry.get_mut();
            *value = *value + transaction.subtotal;
        }
    }
    // Deduct sold asset.
    match balances.entry(transaction.ticker) {
        Entry::Vacant(entry) => panic!("Selling {} which is not in balances!", entry.key()),
        Entry::Occupied(mut entry) => {
            let mut value = entry.get_mut();
            *value = *value - transaction.amount;
        }
    }
}

fn process_transaction(balances: &mut HashMap<String, f64>, transaction: Transaction) {
    // Remove transaction fee from balances.
    match balances.entry(transaction.ticker) {
        Entry::Vacant(entry) => panic!("Transacting in {} you don't own.", entry.key()),
        Entry::Occupied(mut entry) => {
            let mut value = entry.get_mut();
            *value = *value - transaction.fees;
        }
    }
}

fn calculate_balance(transactions: Vec<Transaction>) -> HashMap<String, f64> {
    let balances = transactions.into_iter().fold(HashMap::new(), |mut balances, transaction| {
        match transaction.transaction_type.as_ref() {
            transaction_types::BUY => process_buy(&mut balances, transaction),
            transaction_types::SELL => process_sell(&mut balances, transaction),
            transaction_types::TX => process_transaction(&mut balances, transaction),
            _ => panic!("Unkown transaction type encountered!")
        }
        balances
    });
    balances
}

fn main() {
    let mut transactions = read_transactions().expect("Can't read transactions");
    transactions.sort_by(|t1, t2| t1.timestamp.cmp(&t2.timestamp) );
    let balances = calculate_balance(transactions);
    balances.iter().for_each(|currency_balance| {
        let currency = currency_balance.0;
        let balance = currency_balance.1;
        println!("{}: {}", currency, balance);
    });
}
