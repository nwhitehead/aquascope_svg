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

/// Trait that drawable objects have
trait Drawable {
    fn translate(&mut self, tx: f32, ty: f32);
    fn bounding_box(&self) -> Rect;
    fn draw(&self, doc: Document) -> Document;
}

/// A drawing is any item that can be present in the final SVG
#[derive(Clone)]
struct GBox {
    r: Rect,
}

impl GBox {
    fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            r: Rect { x, y, w, h}
        }
    }
}

fn translate(r: &Rect, tx: f32, ty: f32) -> Rect {
    Rect {
        x: r.x + tx,
        y: r.y + ty,
        w: r.w,
        h: r.h,
    }
}

impl Drawable for GBox {
    fn translate(&mut self, tx: f32, ty: f32) {
        self.r = translate(&self.r, tx, ty);
    }
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
    fn new() -> Self {
        Self {
            items: vec![],
        }
    }
}

impl Drawable for GArray {
    fn translate(&mut self, tx: f32, ty: f32) {
        for item in &mut self.items {
            item.translate(tx, ty);
        }
    }

    fn bounding_box(&self) -> Rect {
        let mut x: f32 = 1000.0;
        let mut y: f32 = 1000.0;
        let mut x2: f32 = -1000.0;
        let mut y2: f32 = -1000.0;
        for item in &self.items {
            let bb = item.bounding_box();
            x = x.min(bb.x);
            y = y.min(bb.y);
            x2 = x2.max(bb.x + bb.w);
            y2 = y2.max(bb.y + bb.h);
        }
        Rect { x, y, w: x2 - x, h: y2 - y }
    }
    fn draw(&self, doc: Document) -> Document {
        let mut d = doc;
        for item in &self.items {
            d = item.draw(d);
        }
        d
    }
}

fn hstack(left: Box<dyn Drawable>, right: Box<dyn Drawable>) -> Box<dyn Drawable> {
    let left_bb = left.bounding_box();
    let mut c = GArray::new();
    c.push(left);
    Box::new(GBox::new(0.0, 0.0, 1.0, 1.0))
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
    let bx = GBox::new(0.0, 0.0, 40.0, 40.0);
    let bx2 = GBox::new(20.0, 20.0, 40.0, 40.0);
    let hs = hstack(Box::new(bx.clone()), Box::new(bx2.clone()));
    let mut c = GArray::new();
    c.push(Box::new(bx));
    c.push(Box::new(bx2));
    let document = c.draw(Document::new())
        .set("viewBox", view_box(outline(c.bounding_box(), 10.0)));

    svg::save("image.svg", &document).unwrap();
}
