mod parser;
mod render;
mod states;

use anyhow::Result;
use clap::Parser;
use parser::parse;
use render::{Format, render};
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
    #[arg(help = "Output an HTML fragment", long, default_value_t = false)]
    output_html: bool,
    #[arg(help = "Output filename", long)]
    output: Option<String>,
    #[arg(help = "Input filename")]
    input: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let format = if args.output_html {
        Format::Html
    } else {
        Format::Svg
    };
    let contents = fs::read_to_string(&args.input)?;
    let program = parse(&contents)?;
    if args.show_parse {
        println!("{:#?}", program);
        return Ok(());
    }
    let output = render(&program, format)?;
    println!("{}", output);
    Ok(())
}
