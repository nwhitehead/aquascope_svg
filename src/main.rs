use clap::Parser;

#[derive(Parser)]
#[command(name = "aquascope_svg")]
#[command(about = "A tool for working with aquascope SVGs", long_about = None)]
struct Args {
    #[arg(help = "Input filename")]
    input: String,
}

fn main() {
    let args = Args::parse();
    println!("{}", args.input);
}
