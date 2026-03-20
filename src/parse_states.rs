mod parser;
mod states;

use clap::Parser;
use std::fs;

#[derive(Debug, Parser)]
#[command(name = "parse_states")]
#[command(about = "A tool for parsing STATES diagram files", long_about = None)]
struct Args {
    #[arg(help = "Input filename")]
    input: String,
}

fn main() {
    let args = Args::parse();
    let contents = fs::read_to_string(&args.input).expect("Failed to read input file");
    let program = parser::parse(&contents).expect("Parse error");
    println!("{:#?}", program);
}
