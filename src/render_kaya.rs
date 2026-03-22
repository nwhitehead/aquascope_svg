mod parser;
mod render;
mod states;

use anyhow::Result;
use clap::Parser;
use parser::parse;
use render::{Format, render};
use std::fs;
use headless_chrome::Browser;
use headless_chrome::protocol::cdp::Page;

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
    #[arg(help = "Output an HTML fragment", long, default_value_t = false)]
    output_html: bool,
    #[arg(
        help = "Inline JS dependencies (default is to reference a CDN)",
        long,
        default_value_t = false
    )]
    #[arg(help = "Output diagram in PNG", long, default_value_t = false)]
    output_png: bool,
    #[arg(
        help = "Inline JS dependencies (default is to reference a CDN)",
        long,
        default_value_t = false
    )]
    inline_js: bool,
    #[arg(
        help = "Show labels starting with H (heap) (default is to hide)",
        long,
        default_value_t = false
    )]
    show_heap: bool,
    #[arg(help = "Output filename, required (use - for stdout)", long)]
    output: String,
    #[arg(help = "Input filename")]
    input: String,
}

fn save_png_from(content: String, filename: String) -> Result<()> {
    let browser = Browser::default()?;
    let tab = browser.new_tab()?;
    let data_url = format!("data:text/html;charset=utf-8;base64,{}", base64::encode(&content));
    tab.navigate_to(data_url.as_str())?;
    std::thread::sleep(std::time::Duration::from_secs(1)); // Wait
    let png_data = tab.capture_screenshot(
        Page::CaptureScreenshotFormatOption::Png,
        None,
        None,
        true)?;
    std::fs::write(filename, png_data)?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let format = if args.output_html || args.output_png {
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
    let output = render(&program, format, args.inline_js, args.show_heap)?;
    if args.output_png {
        save_png_from(output, args.output)?;
    } else {
        if args.output != "-" {
            fs::write(args.output, output)?;
        } else {
            println!("{}", output);
        }
    }
    Ok(())
}
