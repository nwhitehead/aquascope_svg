use crate::states::{Def, Location, NamedStruct, Program, Ptr, Region, Step, Value};

use anyhow::Result;

pub fn render(prg: &Program) -> Result<String> {
    let output = render_program(&prg)?;
    Ok(output)
}

fn render_program(prg: &Program) -> Result<String> {
    let steps = &prg.0;
    let pieces: Result<Vec<String>> = steps
        .into_iter()
        .map(|s| render_step(&s))
        .collect();
    let res = pieces?.join("");
    Ok(res)
}

fn render_step(step: &Step) -> Result<String> {
    Ok("<step>".into())
}