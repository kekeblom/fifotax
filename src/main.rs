
#[macro_use]
extern crate serde_derive;

use std::error::Error;
use std::env;

mod tax;

use tax::*;

fn read_arguments() -> Result<String, Box<Error>> {
    match env::args().nth(1) {
        None => Err(From::from("expected a file name as argument")),
        Some(file_path) => Ok(file_path)
    }
}

fn main() {
    let file_path = read_arguments().expect("No input filename given");
    let (profits, balances) = calculate_profits_balances(file_path);

    for (currency, entries) in balances.iter() {
        let total_in_currency: f64 = entries.into_iter().map(|e| e.amount).sum();
        println!("{}: {}", currency, total_in_currency);
    }

    println!("Profits:");
    for profit in profits.into_iter() {
        println!("sold currency: {}, amount: {}, sell price: {}, cost: {}", profit.currency, profit.amount_sold, profit.eur_total, profit.cost);
    }
}
