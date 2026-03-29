use anyhow::Result;
use anyhow::anyhow;
use clap::Parser;

use kaya_lib::parser::parse;

#[derive(Debug, Parser)]
#[command(name = "render_states")]
#[command(about = "A tool for rendering Kaya diagrams", long_about = None)]
struct Args {
    #[arg(
        help = "Parse the input and show debug parsing output to stdout",
        long,
        default_value_t = false
    )]
    show_parse: bool,
    #[arg(
        help = "Set scale factor for PNG output, 1.0 is web standard 96 DPI, 3.125 is 300 DPI",
        long
    )]
    output_png_scale: Option<f64>,
    #[arg(
        help = "Show labels starting with H (heap) (default is to hide)",
        long,
        default_value_t = false
    )]
    show_heap: bool,
    #[arg(
        help = "Output filename(s), required, (use - for stdout), (must be PNG)",
        long
    )]
    output: Vec<String>,
    #[arg(help = "Input filename(s)")]
    input: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.input.is_empty() {
        return Err(anyhow!("At least one input filename is required."));
    }
    Ok(())
}
