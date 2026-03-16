use clap::Parser;
use std::fs;
use svg::{Document, Node};
use svg::node::element::Path;
use svg::node::element::path::Data;

mod mtrace;
use mtrace::{AbbreviatedMValue, MTrace, MValue};

#[derive(Parser)]
#[command(name = "aquascope_svg")]
#[command(about = "A tool for converting Aquascope JSON to SVG", long_about = None)]
struct Args {
    #[arg(help = "Input filename")]
    input: String,
}

/// Trait that drawable objects have
pub trait Drawable {
    fn bounding_box(&self) -> (f32, f32, f32, f32);
    fn draw(&self, doc: Document) -> Document;
}

/// A drawing is any item that can be present in the final SVG
struct GBox {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Drawable for GBox {
    fn bounding_box(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.w, self.h)
    }
    fn draw(&self, doc: Document) -> Document {
        let data = Data::new()
            .move_to((self.x, self.y))
            .line_by((self.x + self.w, self.y))
            .line_by((self.x + self.w, self.y + self.h))
            .line_by((self.x, self.y + self.h))
            .close();

        let path = Path::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 3)
            .set("d", data);

        doc.add(path)
    }
}
//fn node_of_value(value: &MValue) 

fn collect_leaves(value: &MValue) -> Vec<&MValue> {
    let mut leaves = Vec::new();
    match value {
        MValue::Bool { .. }
        | MValue::Char { .. }
        | MValue::Uint { .. }
        | MValue::Int { .. }
        | MValue::Float { .. }
        | MValue::Pointer { .. }
        | MValue::Unallocated { .. } => {
            leaves.push(value);
        }
        MValue::Tuple { value: vals } => {
            for v in vals {
                leaves.extend(collect_leaves(v));
            }
        }
        MValue::Array { value: arr } => match arr {
            AbbreviatedMValue::All { value: vals } => {
                for v in vals {
                    leaves.extend(collect_leaves(v));
                }
            }
            AbbreviatedMValue::Only {
                value: (vals, boxed),
            } => {
                for v in vals {
                    leaves.extend(collect_leaves(v));
                }
                leaves.extend(collect_leaves(boxed));
            }
        },
        MValue::Adt { value: adt } => {
            for (_, v) in &adt.fields {
                leaves.extend(collect_leaves(v));
            }
        }
    }
    leaves
}

fn main() {
    let args = Args::parse();
    let content = fs::read_to_string(&args.input).expect("Failed to read input file");
    let json: MTrace = serde_json::from_str(&content).expect("Failed to parse JSON");

    for (step_idx, step) in json.steps.iter().enumerate() {
        println!("=== Step {} ===", step_idx);

        for frame in &step.stack.frames {
            for local in &frame.locals {
                let leaves = collect_leaves(&local.value);
                println!("Local '{}': {:?}", local.name, leaves);
            }
        }

        for (heap_idx, heap_val) in step.heap.locations.iter().enumerate() {
            let leaves = collect_leaves(heap_val);
            println!("Heap[{}]: {:?}", heap_idx, leaves);
        }
    }
    let bx = GBox {
        x: -10.0,
        y: -10.0,
        w: 20.0,
        h: 20.0,
    };
    let document = bx.draw(Document::new())
        .set("viewBox", bx.bounding_box());

    svg::save("image.svg", &document).unwrap();
}
