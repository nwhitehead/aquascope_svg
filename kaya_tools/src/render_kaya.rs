use anyhow::Result;
use anyhow::anyhow;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use clap::Parser;
use headless_chrome::Browser;
use headless_chrome::browser::tab::Tab;
use headless_chrome::protocol::cdp::DOM::RGBA;
use headless_chrome::protocol::cdp::Page;
use headless_chrome::types::Bounds;
use kaya_lib::parser::parse;
use kaya_lib::render::render;
use std::fs;
use std::sync::Arc;

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
        help = "Output filename(s), required, (use - for stdout), (HTML/PNG allowed)",
        long
    )]
    output: Vec<String>,
    #[arg(help = "Input filename(s)")]
    input: Vec<String>,
}

#[derive(Clone)]
struct Headless {
    _browser: Browser,
    tab: Arc<Tab>,
}

impl Headless {
    fn new() -> Result<Self> {
        let browser = Browser::default()?;
        let tab = browser.new_tab()?;
        tab.set_bounds(Bounds::Normal {
            left: None,
            top: None,
            width: Some(2048.0),
            height: Some(2048.0),
        })?;
        // Set background color to transparent so transparency in body shows up in png if desired
        tab.set_background_color(RGBA {
            r: 0,
            g: 0,
            b: 0,
            a: Some(0.0),
        })?;
        Ok(Self {
            _browser: browser,
            tab,
        })
    }
    fn save_png_from(&mut self, content: String, filename: String, scale: f64) -> Result<()> {
        // Create data uri that base64 encodes full page
        let data_url = format!(
            "data:text/html;charset=utf-8;base64,{}",
            STANDARD.encode(&content)
        );
        // Go to uri and wait for it to be ready
        self.tab.navigate_to(data_url.as_str())?;
        self.tab.wait_for_element("div.program")?;
        // Get viewport for diagram
        let element = self.tab.find_element("div.program")?;
        element.scroll_into_view()?;
        let box_model = element.get_box_model()?;
        let mut viewport = box_model.margin_viewport();
        // Set scale which adjusts DPI
        viewport.scale = scale;
        let png_data = self.tab.capture_screenshot(
            Page::CaptureScreenshotFormatOption::Png,
            Some(100),
            Some(viewport),
            true,
        )?;
        std::fs::write(filename, png_data)?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.input.is_empty() {
        return Err(anyhow!("At least one input filename is required."));
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

    if args.input.len() != args.output.len() {
        return Err(anyhow!(
            "Number of inputs and outputs must match. Got {} input{} and {} output{}.",
            args.input.len(),
            if args.input.len() != 1 { "s" } else { "" },
            args.output.len(),
            if args.output.len() != 1 { "s" } else { "" },
        ));
    }
    let mut headless: Option<Headless> = None;
    for (input_filename, output_filename) in std::iter::zip(args.input, args.output) {
        let output_html = output_filename.ends_with(".html") || output_filename.ends_with(".HTML");
        let output_png = output_filename.ends_with(".png");
        if !output_html && !output_png {
            return Err(anyhow!(
                "Not a valid output format for filename: {}",
                output_filename
            ));
        }
        let contents = fs::read_to_string(&input_filename)
            .map_err(|err| anyhow!("{}, could not read input filename: {}", err, input_filename))?;
        let program = parse(&contents)?;
        let output = render(&program, args.show_heap)?;
        if output_png {
            let mut h = match headless {
                Some(hd) => hd,
                None => Headless::new()?,
            };
            h.save_png_from(
                output,
                output_filename,
                args.output_png_scale.unwrap_or(1.0),
            )?;
            headless = Some(h.clone());
        } else if output_filename != "-" {
            fs::write(output_filename, output)?;
        } else {
            println!("{}", output);
        }
    }
    Ok(())
}
