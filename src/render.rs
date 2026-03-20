use crate::states::{Def, Location, NamedStruct, Program, Ptr, Region, Step, Value};

use anyhow::Result;

pub fn render(prg: &Program) -> Result<String> {
    let output = render_program(&prg)?;
    Ok(output)
}

fn render_program(prg: &Program) -> Result<String> {
    let mut res = String::new();
    for step in &prg.0 {
        let piece = render_step(&step)?;
        res.push_str(&piece);
    }
    Ok(res)
}

fn render_step(step: &Step) -> Result<String> {
    let mut res = String::new();
    res.push_str(&step.label);
    res.push_str(" -> ");
    for location in &step.locations {
        let piece = render_location(&location)?;
        res.push_str(&piece);
    }
    Ok(res)
}

fn render_location(loc: &Location) -> Result<String> {
    let mut res = String::new();
    res.push_str(&loc.name);
    res.push_str(" -> ");
    // A location either has definitions itself (and no regions) OR it has regions and no definitions
    assert!(loc.definitions.is_empty() || loc.regions.is_empty());
    if !loc.definitions.is_empty() {
        let piece = render_definitions(&loc.definitions)?;
        res.push_str(&piece);
    } else {
        for region in &loc.regions {
            let piece = render_region(&region)?;
            res.push_str(&piece);
        }
    }
    Ok(res)
}

fn render_region(region: &Region) -> Result<String> {
    let mut res = String::new();
    res.push_str(&region.name);
    let pieces = render_definitions(&region.definitions)?;
    res.push_str(&pieces);
    Ok(res)
}

fn render_definitions(definitions: &[Def]) -> Result<String> {
    Ok("[defns]".into())
}
