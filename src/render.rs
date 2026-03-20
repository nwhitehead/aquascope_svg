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
    Ok(format!(r#"<?xml version="1.0" standalone="yes"?>
<svg width="4in" height="3in"
 xmlns = 'http://www.w3.org/2000/svg'>
  <desc>This example uses the 'switch' element to provide a
        fallback graphical representation of an paragraph, if
        XMHTML is not supported.</desc>
  <!-- The 'switch' element will process the first child element
       whose testing attributes evaluate to true.-->
  <switch>
    <!-- Process the embedded XHTML if the requiredExtensions attribute
         evaluates to true (i.e., the user agent supports XHTML
         embedded within SVG). -->
    <foreignObject width="100" height="50"
                   requiredExtensions="http://example.com/SVGExtensions/EmbeddedXHTML">
      <!-- XHTML content goes here -->
      <body xmlns="http://www.w3.org/1999/xhtml">
        <p>Here is a paragraph that requires word wrap</p>
      </body>
    </foreignObject>
    <!-- Else, process the following alternate SVG.
         Note that there are no testing attributes on the 'text' element.
         If no testing attributes are provided, it is as if there
         were testing attributes and they evaluated to true.-->
    <text font-size="10" font-family="Verdana">
      <tspan x="10" y="10">Here is a paragraph that</tspan>
      <tspan x="10" y="20">requires word wrap.</tspan>
    </text>
  </switch>
</svg>
"#/*, res*/))
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
