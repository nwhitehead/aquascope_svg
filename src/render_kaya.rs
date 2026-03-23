mod parser;
mod render;
mod states;

use anyhow::anyhow;
use anyhow::Result;
use clap::Parser;
use parser::parse;
use render::{Format, render};
use std::fs;
use headless_chrome::Browser;
use headless_chrome::protocol::cdp::Page;
use headless_chrome::protocol::cdp::DOM::RGBA;
use headless_chrome::types::Bounds;
use base64::{engine::general_purpose::STANDARD, Engine as _};

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
    #[arg(help = "Set scale factor for PNG output, 1.0 is web standard 96 DPI, 3.125 is 300 DPI", long)]
    output_png_scale: Option<f64>,
    #[arg(
        help = "Show labels starting with H (heap) (default is to hide)",
        long,
        default_value_t = false
    )]
    show_heap: bool,
    #[arg(help = "Output filename, required, (use - for stdout), (HTML/SVG/PNG allowed)", long)]
    output: String,
    #[arg(help = "Input filename")]
    input: String,
}

fn save_png_from(content: String, filename: String, scale: f64) -> Result<()> {
    let browser = Browser::default()?;
    let tab = browser.new_tab()?;
    tab.set_bounds(Bounds::Normal {
        left: None,
        top: None,
        width: Some(2048.0),
        height: Some(2048.0),
    })?;
    // Create data uri that base64 encodes full page
    let data_url = format!("data:text/html;charset=utf-8;base64,{}", STANDARD.encode(&content));
    // Go to uri and wait for it to be ready
    tab.navigate_to(data_url.as_str())?
        .wait_until_navigated()?
        .wait_for_element("div.program")?;
    // Set background color to transparent so transparency in body shows up in png if desired
    tab.set_background_color(RGBA {
        r: 0,
        g: 0,
        b: 0,
        a: Some(0.0),
    })?;

    // Get viewport for diagram
    let element = tab.find_element("div.program")?;
    element.scroll_into_view()?;
    let box_model = element.get_box_model()?;
    let mut viewport = box_model.margin_viewport();
    // Set scale which adjusts DPI
    viewport.scale = scale;
    let png_data = tab.capture_screenshot(
        Page::CaptureScreenshotFormatOption::Png,
        Some(100),
        Some(viewport),
        true)?;
    std::fs::write(filename, png_data)?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let output_html = args.output.ends_with(".html") || args.output.ends_with(".HTML");
    let output_png = args.output.ends_with(".png");
    let output_svg = args.output.ends_with(".svg");
    if !output_html && !output_png && !output_svg {
        return Err(anyhow!("Not a valid output format."));
    }
    let format = if output_html || output_png {
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
    let output = render(&program, format, args.show_heap)?;
    if output_png {
        save_png_from(output, args.output, args.output_png_scale.unwrap_or(1.0))?;
    } else {
        if args.output != "-" {
            fs::write(args.output, output)?;
        } else {
            println!("{}", output);
        }
    }
    Ok(())
}
