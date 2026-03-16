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

#[derive(Clone)]
struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

pub trait Translatable {
    fn translate(self, tx: f32, ty: f32) -> Self;
}

/// Trait that drawable objects have
pub trait Drawable {
    fn bounding_box(&self) -> Rect;
    fn draw(&self, doc: Document) -> Document;
}

/// A drawing is any item that can be present in the final SVG
struct GBox {
    r: Rect,
}

impl Translatable for Rect {
    fn translate(self, tx: f32, ty: f32) -> Self {
        Rect {
            x: self.x + tx,
            y: self.y + ty,
            w: self.w,
            h: self.h,
        }
    }
}

impl Translatable for GBox {
    fn translate(self, tx: f32, ty: f32) -> Self {
        GBox {
            r: self.r.translate(tx, ty),
        }
    }
}

impl Drawable for GBox {
    fn bounding_box(&self) -> Rect {
        self.r.clone()
    }
    fn draw(&self, doc: Document) -> Document {
        let (x, y, w, h) = (self.r.x, self.r.y, self.r.w, self.r.h);
        let data = Data::new()
            .move_to((x, y))
            .line_by((w, 0))
            .line_by((0, h))
            .line_by((-w, 0))
            .close();

        let path = Path::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 3)
            .set("d", data);

        doc.add(path)
    }
}

/// Utility function for extracting viewBox numbers from Rect
fn view_box(r: Rect) -> (f32, f32, f32, f32) {
    (r.x, r.y, r.w, r.h)
}

/// Expand rect evenly on all sides by d
fn outline(rect: Rect, d: f32) -> Rect {
    Rect {
        x: rect.x - d,
        y: rect.y - d,
        w: rect.w + 2.0 * d,
        h: rect.h + 2.0 * d,
    }
}

struct GArray {
    items: Vec<Box<dyn Drawable>>,
}

impl GArray {
    fn push(&mut self, item: Box<dyn Drawable>) {
        self.items.push(item);
    }
}

impl Drawable for GArray {
    fn bounding_box(&self) -> Rect {
        Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 }
    }
    fn draw(&self, doc: Document) -> Document {
        doc
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
        r: Rect {
            x: 0.0,
            y: 0.0,
            w: 40.0,
            h: 40.0,
        },
    };
    let document = bx.draw(Document::new())
        .set("viewBox", view_box(outline(bx.bounding_box(), 10.0)));

    svg::save("image.svg", &document).unwrap();
}
