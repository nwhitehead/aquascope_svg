mod parser;
mod states;

use anyhow::Result;
use clap::Parser;
use std::fs;

#[derive(Debug, Parser)]
#[command(name = "render_states")]
#[command(about = "A tool for rendering STATES diagrams", long_about = None)]
struct Args {
    #[arg(
        help = "Parse the input and show debug parsing output to stdout",
        long,
        default_value_t = false
    )]
    show_parse: bool,
    #[arg(help = "Output filename", long)]
    output: Option<String>,
    #[arg(help = "Input filename")]
    input: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let contents = fs::read_to_string(&args.input)?;
    let program = parser::parse(&contents)?;
    if args.show_parse {
        println!("{:#?}", program);
        return Ok(());
    }
    println!("Not implemented yet");
    Ok(())
}
