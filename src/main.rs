use clap::Parser;
use serde_json::Value;
use std::fs;

#[derive(Parser)]
#[command(name = "aquascope_svg")]
#[command(about = "A tool for converting Aquascope JSON to SVG", long_about = None)]
struct Args {
    #[arg(help = "Input filename")]
    input: String,
}

fn main() {
    let args = Args::parse();
    let content = fs::read_to_string(&args.input).expect("Failed to read input file");
    let json: Value = serde_json::from_str(&content).expect("Failed to parse JSON");
    println!("{:?}", json);
}
