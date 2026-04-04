use ab_glyph::point;
use anyhow::Result;
use anyhow::anyhow;
use clap::Parser;
use std::fs;
use tiny_skia::Color;

use kaya_lib::parser::parse;
mod arrow;
mod canvas;
mod draw;
mod draw_state;
mod render;
mod style;

use crate::canvas::Canvas;
use crate::render::{RenderState, render_program};
use crate::style::standard_style;

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

        // Start with measurement, empty canvas
        let mut canvas = Canvas::new(1, 1)?;
        canvas.load_font(
            "mono",
            include_bytes!("../fonts/DejaVu/DejaVuSansMono-Bold.ttf"),
        )?;
        canvas.load_font("serif", include_bytes!("../fonts/Lato/Lato-Regular.ttf"))?;
        canvas.load_font("serif_bold", include_bytes!("../fonts/Lato/Lato-Bold.ttf"))?;
        let mut rs = RenderState::default();
        rs.style = standard_style()?;
        let mut v = render_program(&program, &mut rs, &canvas)?;
        let bb = v.bounding_box(&canvas)?;
        // Translate to 0, 0
        v.translate(point(-bb.min.x, -bb.min.y));
        let w = bb.max.x - bb.min.x;
        let h = bb.max.y - bb.min.y;
        // Now we know size, recreate canvas at right size with fonts
        canvas = Canvas::new(w.ceil() as u32, h.ceil() as u32)?;
        canvas
            .pixmap
            .fill(Color::from_rgba8(0x19, 0x19, 0x19, 0xff));
        canvas.load_font(
            "mono",
            include_bytes!("../fonts/DejaVu/DejaVuSansMono-Bold.ttf"),
        )?;
        canvas.load_font("serif", include_bytes!("../fonts/Lato/Lato-Regular.ttf"))?;
        canvas.load_font("serif_bold", include_bytes!("../fonts/Lato/Lato-Bold.ttf"))?;
        v.draw(&mut canvas)?;

        canvas.save(&output_filename)?;
    }
    Ok(())
}
