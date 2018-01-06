
#[macro_use]
extern crate serde_derive;

use std::error::Error;
use std::env;

mod tax;

use tax::*;

struct CLIArguments {
    in_file: String,
    out_file: String
}

fn read_arguments() -> Result<CLIArguments, Box<Error>> {
    let mut args = env::args();
    let in_file = args.nth(1).expect("Expected an input file name");
    let out_file = args.next().unwrap_or(String::from("./out.csv"));

    Ok(CLIArguments { in_file: in_file, out_file: out_file })
}

fn main() {
    let cli_args = read_arguments().expect("No input filename given");
    let (profits, balances) = calculate_profits_balances(cli_args.in_file);

    for (currency, entries) in balances.iter() {
        let total_in_currency: f64 = entries.into_iter().map(|e| e.amount).sum();
        println!("{}: {}", currency, total_in_currency);
    }

    println!("out file: {}", cli_args.out_file);
    write_profits_to_file(&profits, &cli_args.out_file).expect("Can't write to file.");
}
