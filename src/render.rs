use crate::states::{Def, Location, NamedStruct, Program, Ptr, Region, Step, Value};

use anyhow::Result;

pub enum Format {
    Svg,
    Html,
}

pub fn render(prg: &Program, format: Format) -> Result<String> {
    let output = render_program(&prg, format)?;
    Ok(output)
}

fn render_program(prg: &Program, format: Format) -> Result<String> {
    let mut res = String::new();
    for step in &prg.0 {
        let piece = render_step(&step)?;
        res.push_str(&piece);
    }
    let output = match format {
        Format::Html => format!(r#"
<!DOCTYPE html>
<html>
    {}
</html>
"#, res),
        Format::Svg => format!(r#"
<svg viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg">
  <style>
    div {{
      color: white;
      font: 18px serif;
      height: 100%;
      overflow: auto;
    }}
  </style>
  <foreignObject x="0" y="0" width="200" height="200">
    <div xmlns="http://www.w3.org/1999/xhtml">
      <pre>{}</pre>
    </div>
  </foreignObject>
</svg>
"#, res),
    };
    Ok(output)
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
