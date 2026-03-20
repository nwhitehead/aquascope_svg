use crate::states::{Def, Location, NamedStruct, Program, Ptr, Region, Step, Value};

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
        Format::Html => format!(r#"
<!DOCTYPE html>
<html>
<head>
<style>{}</style>
</head>
<body>
{}
<script src="https://cdn.jsdelivr.net/npm/leader-line@1.0.7/leader-line.min.js"></script>
</body>
</html>
"#, CSS_STYLE, prg),
        Format::Svg => format!(r#"
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
"#, CSS_STYLE, prg),
    };
    Ok(output)
}

fn render_program(prg: &Program) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"program\">");
    for step in &prg.0 {
        let piece = render_step(&step)?;
        res.push_str(&piece);
    }
    res.push_str("</div>");
    Ok(res)
}

fn render_step(step: &Step) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"step\">");
    res.push_str(&format!("<span class=\"header\">{}</span>", &step.label));
    for location in &step.locations {
        let piece = render_location(&location)?;
        res.push_str(&piece);
    }
    res.push_str("</div>");
    Ok(res)
}

fn render_location(loc: &Location) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"location\">");
    res.push_str(&format!("<span class=\"header\">{}</span>", &loc.name));
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
    res.push_str("</div>");
    Ok(res)
}

fn render_region(region: &Region) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"region\">");
    res.push_str(&format!("<span class=\"header\">{}</span>", &region.name));
    let pieces = render_definitions(&region.definitions)?;
    res.push_str(&pieces);
    res.push_str("</div>");
    Ok(res)
}

fn render_definitions(definitions: &[Def]) -> Result<String> {
    let mut res = String::new();
    for definition in definitions {
        let piece = render_definition(&definition)?;
        res.push_str(&piece);
    }
    Ok(res)
}

fn render_definition(definition: &Def) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"definition\">");
    res.push_str(&format!("<span class=\"label\">{}</span>", &definition.label));
    res.push_str(&"<span class=\"separator\">:</span>");
    let v = render_value(&definition.value)?;
    res.push_str(&v);
    res.push_str("</div>");
    Ok(res)
}

fn render_value(value: &Value) -> Result<String> {
    match value {
        Value::Number(v) => Ok(format!("<span class=\"value number\">{}</span>", v)),
        Value::Array(v) => {
            let mut res = String::new();
            res.push_str("<div class=\"value array\">");
            res.push_str(&render_values("array_child", &v)?);
            res.push_str("</div>");
            Ok(res)
        },
        Value::Tuple(v) => {
            let mut res = String::new();
            res.push_str("<div class=\"value tuple\">");
            res.push_str(&render_values("tuple_child", &v)?);
            res.push_str("</div>");
            Ok(res)
        },
        Value::Char(v) => Ok(format!("<span class=\"value char\">'{}'</span>", v)),
        Value::Struct(v) => render_struct(&v.name, &v.fields),
        Value::Invalid => {
            Ok("<span class=\"value invalid\">*</span>".into())
        },
        _ => Ok("value".into()),
    }
}

fn render_values(inner_tag: &str, values: &[Value]) -> Result<String> {
    let mut res = String::new();
    for value in values {
        let piece = render_value(&value)?;
        res.push_str(&format!("<div class=\"{}\">", inner_tag));
        res.push_str(&piece);
        res.push_str("</div>");
    }
    Ok(res)
}

fn render_struct(name: &str, fields: &[(String, Value)]) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"value struct\">");
    res.push_str(&format!("<span class=\"name\">{}</span>", &name));
    for (label, value) in fields {
        let v = render_field(&label, &value)?;
        res.push_str(&v);
    }
    res.push_str("</div>");
    Ok(res)
}

fn render_field(label: &str, value: &Value) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"field\">");
    res.push_str(&format!("<span class=\"label\">{}</span>", &label));
    res.push_str(&"<span class=\"separator\">:</span>");
    let v = render_value(&value)?;
    res.push_str(&v);
    res.push_str("</div>");
    Ok(res)
}
