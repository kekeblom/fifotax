extern crate csv;
extern crate chrono;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::env;
use std::error::Error;
use std::fs::File;
use csv::StringRecord;
use chrono::prelude::*;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Transaction {
    timestamp: DateTime<Utc>,
    ticker: String,
    transaction_type: String,
    amount: f64,
    subtotal: f64,
    fees: f64,
    total: f64,
    currency: String,
    price_per_coin: f64,
    id: String
}

impl Transaction {
}

fn read_arguments() -> Result<String, Box<Error>> {
    match env::args().nth(1) {
        None => Err(From::from("expected a file name as argument")),
        Some(file_path) => Ok(file_path)
    }
}

fn read_transactions() -> Result<(), Box<Error>> {
    let file_path = read_arguments()?;
    let file = File::open(file_path)?;
    let mut reader = csv::Reader::from_reader(file);


    for record in  reader.deserialize() {
        let transaction: Transaction = record?;
        println!("{:?}", transaction);
    }
    Ok(())
}

fn main() {
    read_transactions().unwrap();
}
