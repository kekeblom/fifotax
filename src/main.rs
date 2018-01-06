
#[macro_use]
extern crate serde_derive;
extern crate clap;

mod tax;

use clap::{App, Arg, ArgMatches};
use tax::*;

fn read_arguments<'a>() -> ArgMatches<'a> {
    App::new("FIFOTax").version("0.1")
        .author("K. Blomqvist")
        .arg(Arg::with_name("INPUT_FILE")
            .help("The transaction history csv file to use.")
            .required(true)
            .index(1))
        .arg(Arg::with_name("OUT_FILE")
            .help("File to write the output to. optional, default: out.csv")
            .required(false)
            .default_value("./out.csv")
            .index(2))
        .arg(Arg::with_name("write-balances")
            .help("If used, writes balances to the file balances.csv")
            .short("b")
            .required(false))
        .get_matches()
}

fn main() {
    let cli_args = read_arguments();
    let (profits, balances) = calculate_profits_balances(cli_args.value_of("INPUT_FILE").expect("No input filename given."));

    for (currency, entries) in balances.iter() {
        let total_in_currency: f64 = entries.into_iter().map(|e| e.amount).sum();
        println!("{}: {}", currency, total_in_currency);
    }

    let out_file = cli_args.value_of("OUT_FILE").expect("No output filename givefilename given.");
    println!("out file: {}", out_file);
    write_profits_to_file(&profits, out_file).expect("Can't write to file.");

    let write_balances = cli_args.is_present("write-balances");
    if write_balances {
        write_balances_to_file(&balances, "./balances.csv").expect("Can't write balances");
    }
}
