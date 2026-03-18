use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "render_states")]
#[command(about = "A tool for rendering STATES diagrams", long_about = None)]
struct Args {
    #[arg(help = "Input filename")]
    input: String,
    #[arg(help = "Output filenames", long)]
    output: String,
}

fn main() {
    let args = Args::parse();
    println!("args = {:?}", args);
}