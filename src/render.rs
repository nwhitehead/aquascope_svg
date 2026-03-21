use crate::states::{Def, Location, Program, Region, Step, Value};

use anyhow::Result;

pub enum Format {
    Svg,
    Html,
}

const CSS_STYLE: &str = r#"
body {
    background-color: #000;
    color: white;
    font: 18px serif;
    height: 100%;
    overflow: auto;
}
div {
    display: inline-block;
    width: fit-content;
}
.program {
    display: flex;
    flex-direction: column;
    gap: 40px;
    background-color: #818;
}
.step {
    display: flex;
    flex-direction: row;
    gap: 20px;
    background-color: #811;
}
.step > .header {
    color: #ff0;
}
.location > .header {
    font-weight: bold;
}
.location {
   display: flex;
   flex-direction: column;
}
.region > .header {
    font-style: italic;
}
.region {
   display: flex;
   flex-direction: column;
}
.value.array {
    background-color: #00f;
    padding: 5px;
    display: inline-flex;
    align-items: center;
    gap: 10px;
    border: 1px solid #888;
    justify-content: start;
}
.value.struct {
    background-color: #0f0;
    padding: 5px;
    display: inline-flex;
    align-items: center;
    gap: 10px;
    border: 1px solid #888;
    justify-content: start;
}
.array_child + .array_child {
    border-left: solid 1px white;
}
"#;

pub fn render(prg: &Program, format: Format) -> Result<String> {
    let prg = render_program(&prg)?;
    let output = match format {
        Format::Html => format!(
            r#"
<!DOCTYPE html>
<html>
<head>
<style>{}</style>
</head>
<body>
{}
<script src="https://cdn.jsdelivr.net/npm/leader-line@1.0.7/leader-line.min.js"></script>
<script>
// Wait for HTML document to get ready
window.addEventListener('load', function() {{ // NOT `DOMContentLoaded`
  // Do something about HTML document
  var line = new LeaderLine(
    document.getElementById('L3.H0'),
    document.getElementById('L3.x.1')
  );
}});
</script>
</body>
</html>
"#,
            CSS_STYLE, prg
        ),
        Format::Svg => format!(
            r#"
<svg viewBox="0 0 2000 2000" xmlns="http://www.w3.org/2000/svg">
  <style>
{}
  </style>
  <foreignObject x="0" y="0" width="2000" height="2000">
    <div xmlns="http://www.w3.org/1999/xhtml">
      <pre>{}</pre>
    </div>
    <script>console.log('hi from svg');
    </script>
  </foreignObject>
</svg>
"#,
            CSS_STYLE, prg
        ),
    };
    Ok(output)
}

fn render_program(prg: &Program) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"program\">");
    for (idx, step) in prg.0.iter().enumerate() {
        let piece = render_step(&step, &format!("L{}", idx), idx)?;
        res.push_str(&piece);
    }
    res.push_str("</div>");
    Ok(res)
}

fn render_step(step: &Step, id_prefix: &str, step_index: usize) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"step\">");
    res.push_str(&format!("<span class=\"header\">{}</span>", &step.label));
    for location in &step.locations {
        let piece = render_location(&location, &id_prefix, step_index)?;
        res.push_str(&piece);
    }
    res.push_str("</div>");
    Ok(res)
}

fn render_location(loc: &Location, id_prefix: &str, step_index: usize) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"location\">");
    res.push_str(&format!("<span class=\"header\">{}</span>", &loc.name));
    // A location either has definitions itself (and no regions) OR it has regions and no definitions
    assert!(loc.definitions.is_empty() || loc.regions.is_empty());
    if !loc.definitions.is_empty() {
        let piece = render_definitions(&loc.definitions, &id_prefix, step_index)?;
        res.push_str(&piece);
    } else {
        for region in &loc.regions {
            let piece = render_region(&region, &id_prefix, step_index)?;
            res.push_str(&piece);
        }
    }
    res.push_str("</div>");
    Ok(res)
}

fn render_region(region: &Region, id_prefix: &str, step_index: usize) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"region\">");
    res.push_str(&format!("<span class=\"header\">{}</span>", &region.name));
    let pieces = render_definitions(&region.definitions, &id_prefix, step_index)?;
    res.push_str(&pieces);
    res.push_str("</div>");
    Ok(res)
}

fn render_definitions(definitions: &[Def], id_prefix: &str, step_index: usize) -> Result<String> {
    let mut res = String::new();
    for definition in definitions {
        let piece = render_definition(&definition, &id_prefix, step_index)?;
        res.push_str(&piece);
    }
    Ok(res)
}

fn render_definition(definition: &Def, id_prefix: &str, step_index: usize) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"definition\">");
    res.push_str(&format!(
        "<span class=\"label\">{}</span>",
        &definition.label
    ));
    res.push_str(&"<span class=\"separator\">:</span>");
    let v = render_value(
        &definition.value,
        &format!("{}.{}", &id_prefix, &definition.label),
        step_index,
    )?;
    res.push_str(&v);
    res.push_str("</div>");
    Ok(res)
}

fn render_value(value: &Value, id_prefix: &str, step_index: usize) -> Result<String> {
    match value {
        Value::Number(v) => Ok(format!(
            "<span id=\"{}\" class=\"value number\">{}</span>",
            &id_prefix, v
        )),
        Value::Array(v) => {
            let mut res = String::new();
            res.push_str(&format!(
                "<div id=\"{}\" class=\"value array\">",
                &id_prefix
            ));
            res.push_str(&render_values("array_child", &v, &id_prefix, step_index)?);
            res.push_str("</div>");
            Ok(res)
        }
        Value::Tuple(v) => {
            let mut res = String::new();
            res.push_str(&format!(
                "<div id=\"{}\" class=\"value tuple\">",
                &id_prefix
            ));
            res.push_str(&render_values("tuple_child", &v, &id_prefix, step_index)?);
            res.push_str("</div>");
            Ok(res)
        }
        Value::Char(v) => Ok(format!(
            "<span id=\"{}\" class=\"value char\">'{}'</span>",
            &id_prefix, v
        )),
        Value::Struct(v) => render_struct(&v.name, &v.fields, &id_prefix, step_index),
        Value::Pointer(v) => {
            let mut dst = String::new();
            dst.push_str(&format!("L{}.{}", step_index, &v.name));
            for selector in &v.selectors {
                dst.push_str(&format!(".{}", selector));
            }
            Ok(format!(
                "<span id=\"{}\" class=\"value pointer\">●{}</span>",
                &id_prefix, &dst
            ))
        }
        Value::Invalid => Ok(format!(
            "<span id=\"{}\" class=\"value invalid\">*</span>",
            &id_prefix
        )),
    }
}

fn render_values(
    inner_tag: &str,
    values: &[Value],
    id_prefix: &str,
    step_index: usize,
) -> Result<String> {
    let mut res = String::new();
    for (idx, value) in values.into_iter().enumerate() {
        let piece = render_value(&value, &format!("{}.{}", &id_prefix, idx), step_index)?;
        res.push_str(&format!("<div class=\"{}\">", inner_tag));
        res.push_str(&piece);
        res.push_str("</div>");
    }
    Ok(res)
}

fn render_struct(
    name: &str,
    fields: &[(String, Value)],
    id_prefix: &str,
    step_index: usize,
) -> Result<String> {
    let mut res = String::new();
    res.push_str(&format!(
        "<div id=\"{}\" class=\"value struct\">",
        &id_prefix
    ));
    res.push_str(&format!("<span class=\"name\">{}</span>", &name));
    for (idx, (label, value)) in fields.into_iter().enumerate() {
        let v = render_field(
            &label,
            &value,
            &format!("{}.{}", &id_prefix, idx),
            step_index,
        )?;
        res.push_str(&v);
    }
    res.push_str("</div>");
    Ok(res)
}

fn render_field(label: &str, value: &Value, id_prefix: &str, step_index: usize) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"field\">");
    res.push_str(&format!("<span class=\"label\">{}</span>", &label));
    res.push_str(&"<span class=\"separator\">:</span>");
    let v = render_value(&value, &id_prefix, step_index)?;
    res.push_str(&v);
    res.push_str("</div>");
    Ok(res)
}
