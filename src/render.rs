use crate::states::{Def, Location, Program, Region, Step, Value};
use serde_json::json;
use handlebars::Handlebars;
use anyhow::Result;
use std::rc::Rc;
use std::cell::RefCell;

pub enum Format {
    Svg,
    Html,
}

const CSS_STYLE: &[u8] = include_bytes!("./style.css");
const LEADER_LINE_JS: &[u8] = include_bytes!("./leader-line-v1.0.7.min.js");
const INDEX_HBS: &[u8] = include_bytes!("./index.hbs");
const SVG_HBS: &[u8] = include_bytes!("./svg.hbs");

struct RenderState {
    id_prefix: String,
    step_index: usize,
    arrows: Rc<RefCell<Vec<(String, String)>>>,
}

impl RenderState {
    fn new(step_index: usize) -> Self {
        Self {
            id_prefix: format!("L{}", step_index),
            step_index,
            arrows: Rc::new(RefCell::new(vec![])),
        }
    }
    fn extend_id(&self, txt: &str) -> Self {
        Self {
            id_prefix: format!("{}.{}", &self.id_prefix, txt),
            step_index: self.step_index,
            arrows: self.arrows.clone(),
        }
    }
    fn with_step_index(&self, step_index: usize) -> Self {
        Self {
            id_prefix: format!("L{}", step_index),
            step_index: step_index,
            arrows: self.arrows.clone(),
        }
    }
    fn add_arrow(&mut self, src: &str, dest: &str) {
        self.arrows.borrow_mut().push((src.into(), dest.into()));
    }
}

pub fn render(prg: &Program, format: Format, inline_js: bool) -> Result<String> {
    let (prg, arrows) = render_program(&prg)?;
    let leader = if inline_js {
        &format!("<script>{}</script>", String::from_utf8(LEADER_LINE_JS.to_vec())?)
    } else {
        r#"<script src="https://cdn.jsdelivr.net/npm/leader-line@1.0.7/leader-line.min.js"></script>"#
    };
    let css_style = String::from_utf8(CSS_STYLE.to_vec())?;
    let index_hbs = String::from_utf8(INDEX_HBS.to_vec())?;
    let svg_hbs = String::from_utf8(SVG_HBS.to_vec())?;
    let reg = Handlebars::new();

    let mut arrow_txt = String::new();
    for (src, dst) in arrows {
        arrow_txt.push_str(&format!(
            "new LeaderLine(document.getElementById('{}'), document.getElementById('{}'));\n", src, dst));
    }
    let output = match format {
        Format::Html => reg.render_template(&index_hbs, &json!({
            "style": css_style,
            "content": prg,
            "script": leader,
            "arrows": &arrow_txt,
        }))?,
        Format::Svg => reg.render_template(&svg_hbs, &json!({"style": css_style, "content": prg, "script": leader}))?,
    };
    Ok(output)
}

fn render_program(prg: &Program) -> Result<(String, Vec<(String, String)>)> {
    let mut res = String::new();
    res.push_str("<div class=\"program\">");
    let state = RenderState::new(0);
    for (idx, step) in prg.0.iter().enumerate() {
        let mut st = state.with_step_index(idx);
        let piece = render_step(&step, &mut st)?;
        res.push_str(&piece);
    }
    res.push_str("</div>");
    let arrows = state.arrows.borrow().clone();
    Ok((res, arrows))
}

fn render_step(step: &Step, state: &mut RenderState) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"step\">");
    res.push_str(&format!("<span class=\"header\">{}</span>", &step.label));
    for location in &step.locations {
        let piece = render_location(&location, state)?;
        res.push_str(&piece);
    }
    res.push_str("</div>");
    Ok(res)
}

fn render_location(loc: &Location, state: &mut RenderState) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"location\">");
    res.push_str(&format!("<span class=\"header\">{}</span>", &loc.name));
    // A location either has definitions itself (and no regions) OR it has regions and no definitions
    assert!(loc.definitions.is_empty() || loc.regions.is_empty());
    if !loc.definitions.is_empty() {
        let piece = render_definitions(&loc.definitions, state)?;
        res.push_str(&piece);
    } else {
        for region in &loc.regions {
            let piece = render_region(&region, state)?;
            res.push_str(&piece);
        }
    }
    res.push_str("</div>");
    Ok(res)
}

fn render_region(region: &Region, state: &mut RenderState) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"region\">");
    res.push_str(&format!("<span class=\"header\">{}</span>", &region.name));
    let pieces = render_definitions(&region.definitions, state)?;
    res.push_str(&pieces);
    res.push_str("</div>");
    Ok(res)
}

fn render_definitions(definitions: &[Def], state: &mut RenderState) -> Result<String> {
    let mut res = String::new();
    for definition in definitions {
        let piece = render_definition(&definition, state)?;
        res.push_str(&piece);
    }
    Ok(res)
}

fn render_definition(definition: &Def, state: &mut RenderState) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"definition\">");
    res.push_str(&format!(
        "<span class=\"label\">{}</span>",
        &definition.label
    ));
    res.push_str(&"<span class=\"separator\">:</span>");
    let mut st = state.extend_id(&definition.label);
    let v = render_value(
        &definition.value,
        &mut st
    )?;
    res.push_str(&v);
    res.push_str("</div>");
    Ok(res)
}

fn render_value(value: &Value, state: &mut RenderState) -> Result<String> {
    match value {
        Value::Number(v) => Ok(format!(
            "<span id=\"{}\" class=\"value number\">{}</span>",
            &state.id_prefix, v
        )),
        Value::Array(v) => {
            let mut res = String::new();
            res.push_str(&format!(
                "<div id=\"{}\" class=\"value array\">",
                &state.id_prefix
            ));
            res.push_str(&render_values("array_child", &v, state)?);
            res.push_str("</div>");
            Ok(res)
        }
        Value::Tuple(v) => {
            let mut res = String::new();
            res.push_str(&format!(
                "<div id=\"{}\" class=\"value tuple\">",
                &state.id_prefix
            ));
            res.push_str(&render_values("tuple_child", &v, state)?);
            res.push_str("</div>");
            Ok(res)
        }
        Value::Char(v) => Ok(format!(
            "<span id=\"{}\" class=\"value char\">'{}'</span>",
            &state.id_prefix, v
        )),
        Value::Struct(v) => render_struct(&v.name, &v.fields, state),
        Value::Pointer(v) => {
            let mut dst = String::new();
            dst.push_str(&format!("L{}.{}", state.step_index, &v.name));
            for selector in &v.selectors {
                dst.push_str(&format!(".{}", selector));
            }
            let src = state.id_prefix.clone();
            state.add_arrow(&src, &dst);
            Ok(format!(
                "<span id=\"{}\" class=\"value pointer\">●</span>",
                &state.id_prefix,
            ))
        }
        Value::Invalid => Ok(format!(
            "<span id=\"{}\" class=\"value invalid\">*</span>",
            &state.id_prefix
        )),
    }
}

fn render_values(
    inner_tag: &str,
    values: &[Value],
    state: &mut RenderState,
) -> Result<String> {
    let mut res = String::new();
    for (idx, value) in values.into_iter().enumerate() {
        let mut state_p = state.extend_id(&format!("{}", idx));
        let piece = render_value(&value, &mut state_p)?;
        res.push_str(&format!("<div class=\"{}\">", inner_tag));
        res.push_str(&piece);
        res.push_str("</div>");
    }
    Ok(res)
}

fn render_struct(
    name: &str,
    fields: &[(String, Value)],
    state: &mut RenderState,
) -> Result<String> {
    let mut res = String::new();
    res.push_str(&format!(
        "<div id=\"{}\" class=\"value struct\">",
        &state.id_prefix
    ));
    res.push_str(&format!("<span class=\"name\">{}</span>", &name));
    for (idx, (label, value)) in fields.into_iter().enumerate() {
        let v = render_field(
            &label,
            &value,
            &mut state.extend_id(&format!("{}", idx))
        )?;
        res.push_str(&v);
    }
    res.push_str("</div>");
    Ok(res)
}

fn render_field(label: &str, value: &Value, state: &mut RenderState) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"field\">");
    res.push_str(&format!("<span class=\"label\">{}</span>", &label));
    res.push_str(&"<span class=\"separator\">:</span>");
    let v = render_value(&value, state)?;
    res.push_str(&v);
    res.push_str("</div>");
    Ok(res)
}
