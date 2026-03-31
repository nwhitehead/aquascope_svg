use anyhow::Result;
use anyhow::anyhow;
use clap::Parser;
use std::fs;

use kaya_lib::parser::parse;
mod canvas;
mod draw;
mod draw_state;
mod render;
mod style;

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

    if args.input.len() != args.output.len() {
        return Err(anyhow!(
            // Be precise in our error message and plurals, come on people, we have the technology
            "Number of inputs and outputs must match. Got {} input{} and {} output{}.",
            args.input.len(),
            if args.input.len() != 1 { "s" } else { "" },
            args.output.len(),
            if args.output.len() != 1 { "s" } else { "" },
        ));
    }

    // Handle show-parse option, there are no output filenames for this case
    if args.show_parse {
        for filename in args.input {
            let contents = fs::read_to_string(&filename)
                .map_err(|err| anyhow!("{}, could not read input filename: {}", err, filename))?;
            let program = parse(&contents)?;
            println!("{:#?}", program);
        }
        return Ok(());
    }

    for (input_filename, output_filename) in std::iter::zip(args.input, args.output) {
        let output_png = output_filename.ends_with(".png");
        if !output_png {
            return Err(anyhow!(
                "Not a valid output format for filename, must end with .png: {}",
                output_filename
            ));
        }
        let contents = fs::read_to_string(&input_filename)
            .map_err(|err| anyhow!("{}, could not read input filename: {}", err, input_filename))?;
        let program = parse(&contents)?;

        // TODO
    }
    Ok(())
}
