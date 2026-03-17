use clap::Parser;
use std::fs;
use svg::save;

mod mtrace;
mod svg_draw;
use mtrace::{AbbreviatedMValue, MTrace, MValue};
use svg_draw::{hstack_spacers, box_around, text_in_box, render, stack, text};

#[derive(Parser)]
#[command(name = "aquascope_svg")]
#[command(about = "A tool for converting Aquascope JSON to SVG", long_about = None)]
struct Args {
    #[arg(help = "Input filename")]
    input: String,
}

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

fn test_svg() {
    let d = 10.0;
    let ds = 10.0;
    let sep_height = 10.0;
    let arr0 = text("0");
    let arr1 = text("1");
    let arr2 = text("2");
    let arr = hstack_spacers(vec![Box::new(arr0), Box::new(arr1), Box::new(arr2)], ds, sep_height);
    let box_arr = box_around(&arr, ds);
    let box_arr_coll = stack(vec![Box::new(arr), Box::new(box_arr)]);
    let t1 = text_in_box("x".into(), d);
    let t2 = text_in_box("0".into(), d);
    let spaced = hstack_spacers(vec![Box::new(t1), Box::new(t2)], ds, sep_height);
    let document = render(&box_arr_coll);

    save("image.svg", &document).unwrap();
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
}
