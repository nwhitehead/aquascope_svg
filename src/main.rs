use clap::Parser;
use std::fs;
use svg::save;
use std::collections::HashMap;

mod mtrace;
mod svg_draw;
use mtrace::{AbbreviatedMValue, MPath, MTrace, MValue, MValueAdt, MValuePointer, CharRange, CharPos, MMemorySegment, MPathSegment};
use svg_draw::{box_around, hstack_spacers, render, stack, text, text_in_box};

#[derive(Parser)]
#[command(name = "aquascope_svg")]
#[command(about = "A tool for converting Aquascope JSON to SVG", long_about = None)]
struct Args {
    #[arg(help = "Input filename")]
    input: String,
}

fn test_svg() {
    let d = 10.0;
    let ds = 10.0;
    let sep_height = 10.0;
    let arr0 = text("0");
    let arr1 = text("1");
    let arr2 = text("2");
    let arr = hstack_spacers(
        vec![Box::new(arr0), Box::new(arr1), Box::new(arr2)],
        ds,
        sep_height,
    );
    let box_arr = box_around(&arr, ds);
    let box_arr_coll = stack(vec![Box::new(arr), Box::new(box_arr)]);
    let t1 = text_in_box("x".into(), d);
    let t2 = text_in_box("0".into(), d);
    let spaced = hstack_spacers(vec![Box::new(t1), Box::new(t2)], ds, sep_height);
    let document = render(&box_arr_coll);

    save("image.svg", &document).unwrap();
}

fn values_display_array(v: &Vec<MValue>) -> String {
    let inner = v
        .iter()
        .map(|x| value_display(&x))
        .collect::<Vec<String>>()
        .join(", ");
    format!("[{}]", inner)
}

fn values_display_tuple(v: &Vec<MValue>) -> String {
    let inner = v
        .iter()
        .map(|x| value_display(&x))
        .collect::<Vec<String>>()
        .join(", ");
    format!("[{}]", inner)
}

fn abbrv_values_display(v: &AbbreviatedMValue) -> String {
    match v {
        AbbreviatedMValue::All { value } => values_display_array(value),
        _ => panic!("Illegal AbbreviatedMValue: Only"),
    }
}

fn escape(x: &str) -> String {
    if x.contains('{') || x.contains('}') {
        format!("`{}`", x)
    } else {
        format!("{}", x)
    }
}

fn adt_name(v: &MValueAdt) -> String {
    if let Some(variant) = &v.variant {
        format!("{}::{}", escape(&v.name), escape(&variant))
    } else {
        format!("{}", escape(&v.name))
    }
}

fn adt_fields(f: &Vec<(String, MValue)>) -> String {
    let inner = f
        .iter()
        .map(|(n, v)| format!("{}: {}", n, value_display(&v)))
        .collect::<Vec<String>>()
        .join(", ");
    format!("{{{}}}", inner)
}

// TODO: handle subslice, I don't know how that works
// TODO: use field names instead of numbers (how?)
fn ptr_tail(v: &Vec<MPathSegment>) -> String {
    v
        .iter()
        .map(|x| {
            let v = match x {
                MPathSegment::Field { value } => value,
                MPathSegment::Index { value } => value,
                _ => panic!("Unhandled subslice pointer specification"),
            };
            format!(".{}", v)
        })
        .collect::<Vec<String>>()
        .join("")
}

// TODO: handle shadowing for stack stuff by adding ticks, currently IGNORED
fn ptr_display(p: &MValuePointer) -> String {
    match &p.path.segment {
        MMemorySegment::Stack { value } => format!("{}{}", value.local, ptr_tail(&p.path.parts)),
        MMemorySegment::Heap { value } => format!("H{}{}", value.index, ptr_tail(&p.path.parts)),
    }
}

fn value_display(v: &MValue) -> String {
    match v {
        MValue::Bool { value } => format!("{:?}", value),
        MValue::Char { value } => format!(
            "'{}'",
            std::char::from_u32(*value).expect("Characters must be UTF-8")
        ),
        MValue::Uint { value } => format!("{:?}", value),
        MValue::Int { value } => format!("{:?}", value),
        MValue::Float { value } => format!("{:?}", value),
        MValue::Array { value } => abbrv_values_display(&value),
        MValue::Tuple { value } => values_display_tuple(&value),
        MValue::Unallocated { value } => "*".into(),
        //MValue::Pointer { value } => format!("ptr({:?})", value),
        MValue::Pointer { value } => format!("ptr({})", ptr_display(&value)),
        MValue::Adt { value } => format!("{}{}", adt_name(value), adt_fields(&value.fields)),
        _ => format!("VALUE[{:?}]", &v).into(),
    }
}

fn strip_off(v: &MValue, name: &str) -> MValue {
    match v {
        MValue::Adt { value } => {
            if value.name == name {
                value.fields[0].1.clone()
            } else {
                v.clone()
            }
        }
        _ => v.clone(),
    }
}

fn strip_off_mult(v: &MValue, names: Vec<&str>) -> MValue {
    let mut res = v.clone();
    let mut all_matched = true;
    for name in names {
        match res {
            MValue::Adt { ref value } => {
                if value.name == name {
                    res = value.fields[0].1.clone();
                }
            }
            _ => all_matched = false,
        }
    }
    // Make subs all or nothing
    if all_matched { res } else { v.clone() }
}

fn strip_off_mult_rec(v: &MValue, names: Vec<&str>) -> MValue {
    match v {
        MValue::Tuple { value } => MValue::Tuple {
            value: value
                .iter()
                .map(|x| strip_off_mult_rec(x, names.clone()))
                .collect(),
        },
        MValue::Array { value } => MValue::Array {
            value: match value {
                AbbreviatedMValue::All { value } => AbbreviatedMValue::All {
                    value: value
                        .iter()
                        .map(|x| strip_off_mult_rec(x, names.clone()))
                        .collect(),
                },
                _ => panic!("Illegal AbbreviatedMValue: Only"),
            },
        },
        _ => strip_off_mult(&v, names),
    }
}

fn simplify_box(v: &MValue) -> MValue {
    strip_off_mult_rec(&v, vec!["Box", "Unique", "NonNull"])
}

fn simplify_vec(v: &MValue) -> MValue {
    strip_off_mult_rec(
        &v,
        vec!["Vec", "RawVec", "RawVecInner", "Unique", "NonNull"],
    )
}

fn simplify_string(v: &MValue) -> MValue {
    strip_off_mult_rec(&v, vec!["String", "Vec", "RawVec", "RawVecInner", "Unique", "NonNull"])
}

fn extract_pointers(v: &MValue, out: &mut Vec<MValuePointer>) {
    match v {
        MValue::Tuple { value } => value
                .iter()
                .map(|x| extract_pointers(x, out))
                .collect(),
        MValue::Array { value } => match value {
            AbbreviatedMValue::All { value } => value
                .iter()
                .map(|x| extract_pointers(x, out))
                .collect(),
            _ => panic!("Illegal AbbreviatedMValue: Only"),
        },
        MValue::Pointer { value } => out.push(value.clone()),
        _ => (),
    }
}

fn tag_code(txt: &str, loc: &CharPos, tag: &str) -> String {
    let mut lines: Vec<String> = txt.split('\n').map(|x| x.to_string()).collect();
    let endloc = &loc;
    lines[endloc.line as usize].insert_str(endloc.column as usize, tag);
    lines.join("\n")
}

fn tag_code_multi(txt: &str, tags: Vec<(String, CharPos)>) -> String {
    let mut result = txt.to_string();
    for tag in tags.iter().rev() {
        result = tag_code(&result, &tag.1, &tag.0);
    }
    result
}

fn main() {
    let args = Args::parse();
    let content = fs::read_to_string(&args.input).expect("Failed to read input file");
    let json: MTrace = serde_json::from_str(&content).expect("Failed to parse JSON");
    // Get original code text
    let code = json.code;

    // Extract tags to put into code snippet
    let mut tags = vec![];
    for (step_idx, step) in json.steps.iter().enumerate() {
        for frame in &step.stack.frames {
            let tag = format!(" /* L{} */ ", step_idx);
            tags.push((tag, frame.location.end.clone()));
        }
    }

    // Show code tagged with labels
    println!("{}", tag_code_multi(&code, tags));

    for (step_idx, step) in json.steps.iter().enumerate() {
        println!("# L{}", step_idx);

        println!("## Stack");
        if step.stack.frames.len() == 0 {}
        for frame in &step.stack.frames {
            println!("### {}", frame.name);
            for local in &frame.locals {
                let simpl = simplify_string(&simplify_box(&simplify_vec(&local.value)));
                println!("{}: {}", local.name, value_display(&simpl));
            }
        }

        if step.heap.locations.len() > 0 {
            println!("## Heap");
        }
        for (heap_idx, heap_val) in step.heap.locations.iter().enumerate() {
            println!("Heap[{}]: {}", heap_idx, value_display(&heap_val));
        }
        // Add blank line before starting next Ln part
        println!("");
    }
}
