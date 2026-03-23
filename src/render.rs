use crate::states::{Def, Location, Program, Region, Step, Value};
use anyhow::Result;
use handlebars::Handlebars;
use serde_json::json;
use std::cell::RefCell;
use std::rc::Rc;

pub enum Format {
    Svg,
    Html,
}

pub enum Theme {
    Dark,
    Light,
    TransparentDark,
    TransparentLight,
    TransparentNoColor,
}

const CSS_STYLE: &[u8] = include_bytes!("./style.css");
const LEADER_LINE_JS: &[u8] = include_bytes!("./leader-line-v1.0.7.min.js");
const INDEX_HBS: &[u8] = include_bytes!("./index.hbs");
const SVG_HBS: &[u8] = include_bytes!("./svg.hbs");
const NUM_ARROW_COLORS: usize = 6;
const DEBUG_ARROWS: bool = false;

#[derive(Clone, Debug)]
struct ArrowInfo {
    src: String,
    dst: String,
    src_help: String,
    dst_help: String,
    src_gravity: String,
    dst_gravity: String,
    path: String,
    color: Option<i32>,
}

#[derive(Clone, Debug)]
struct RenderState {
    id_prefix: String,
    step_index: usize,
    arrows: Rc<RefCell<Vec<ArrowInfo>>>,
}

fn chop_first(txt: &str) -> &str {
    let mut chars = txt.chars();
    chars.next();
    chars.as_str()
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
            step_index,
            arrows: self.arrows.clone(),
        }
    }
    fn add_arrow(&mut self, src: &str, dst: &str, help: &Vec<String>) {
        let mut src_help = String::new();
        let mut dst_help = String::new();
        let mut src_gravity = String::new();
        let mut dst_gravity = String::new();
        let mut path = String::new();
        let mut color = None;
        for h in help {
            match h.as_str() {
                ".sn" => src_help.push('n'),
                ".se" => src_help.push('e'),
                ".sw" => src_help.push('w'),
                ".ss" => src_help.push('s'),
                ".dn" => dst_help.push('n'),
                ".de" => dst_help.push('e'),
                ".dw" => dst_help.push('w'),
                ".ds" => dst_help.push('s'),
                ".straight" | ".arc" | ".fluid" | ".magnet" | ".grid" => {
                    path = chop_first(h.as_str()).to_string();
                }
                ".svl" | ".svlight" => src_gravity = "vlight".to_string(),
                ".sl" | ".slight" => src_gravity = "light".to_string(),
                ".sh" | ".sheavy" => src_gravity = "heavy".to_string(),
                ".svh" | ".svheavy" => src_gravity = "vheavy".to_string(),
                ".dvl" | ".dvlight" => dst_gravity = "vlight".to_string(),
                ".dl" | ".dlight" => dst_gravity = "light".to_string(),
                ".dh" | ".dheavy" => dst_gravity = "heavy".to_string(),
                ".dvh" | ".dvheavy" => dst_gravity = "vheavy".to_string(),
                ".c0" => color = Some(0),
                ".c1" => color = Some(1),
                ".c2" => color = Some(2),
                ".c3" => color = Some(3),
                ".c4" => color = Some(4),
                ".c5" => color = Some(5),
                _ => println!("WARNING: unknown ptr help found, {}", &h),
            }
        }
        self.arrows.borrow_mut().push(ArrowInfo {
            src: src.to_string(),
            dst: dst.to_string(),
            src_help,
            dst_help,
            src_gravity,
            dst_gravity,
            path,
            color,
        });
    }
}

fn socket_dir_to_option(x: &str) -> String {
    match x {
        "a" => "auto".to_string(),
        "n" => "top".to_string(),
        "s" => "bottom".to_string(),
        "w" => "left".to_string(),
        "e" => "right".to_string(),
        _ => "auto".to_string(),
    }
}

fn socket_gravity_to_option(x: &str) -> i32 {
    match x {
        "" => 120,
        "vlight" => 50,
        "light" => 80,
        "heavy" => 160,
        "vheavy" => 190,
        _ => panic!("Unknown gravity"),
    }
}

pub fn render(prg: &Program, format: Format, show_heap: bool) -> Result<String> {
    let (prg, arrows) = render_program(prg, !show_heap)?;
    if DEBUG_ARROWS {
        println!("arrows = {:?}", &arrows);
    }
    let leader = String::from_utf8(LEADER_LINE_JS.to_vec())?;
    let css_style = String::from_utf8(CSS_STYLE.to_vec())?;
    let index_hbs = String::from_utf8(INDEX_HBS.to_vec())?;
    let svg_hbs = String::from_utf8(SVG_HBS.to_vec())?;
    let reg = Handlebars::new();

    let mut arrow_txt = String::new();
    for (
        idx,
        ArrowInfo {
            src,
            dst,
            src_help,
            dst_help,
            src_gravity,
            dst_gravity,
            path,
            color,
        },
    ) in arrows.iter().enumerate()
    {
        let start_socket = socket_dir_to_option(src_help);
        let end_socket = match dst_help.as_str() {
            "a" => "auto",
            "n" => "top",
            "s" => "bottom",
            "w" => "left",
            "e" => "right",
            _ => "auto",
        }
        .to_string();
        let path = match path.as_str() {
            "" => "fluid",
            s => s,
        }
        .to_string();
        let start_socket_gravity = socket_gravity_to_option(src_gravity.as_str());
        let end_socket_gravity = socket_gravity_to_option(dst_gravity.as_str());
        // check for loops on one element, must be handled specially
        let color_txt = match color {
            None => format!("'var(--arrow{})'", idx % NUM_ARROW_COLORS),
            Some(c) => format!("'var(--arrow{})'", (*c as usize) % NUM_ARROW_COLORS),
        };
        let inner = &format!(
            r#"{{
                startSocket: '{}',
                endSocket: '{}',
                startSocketGravity: {},
                endSocketGravity: {},
                color: {},
                size: parseFloat(getCssVar('arrow_width')),
                endPlugSize: parseFloat(getCssVar('arrow_size')),
                outline: getCssVar('arrow_outline') !== "",
                outlineColor: 'var(--arrow_outline)',
                outlineSize: parseFloat(getCssVar('arrow_outline_size')),
                endPlugOutline: getCssVar('arrow_plug_outline') !== "",
                endPlugOutlineColor: 'var(--arrow_plug_outline)',
                endPlugOutlineSize: parseFloat(getCssVar('arrow_plug_outline_size')),
                path: '{}',
            }}"#,
            start_socket,
            end_socket,
            start_socket_gravity,
            end_socket_gravity,
            color_txt,
            path
        );
        if src != dst {
            arrow_txt.push_str(&format!(
                "new LeaderLine(
                    document.getElementById('{}'),
                    document.getElementById('{}'),
                    {}
                );", src, dst, inner));
        } else {
            arrow_txt.push_str(&format!(
                "new LeaderLine(
                  document.getElementById('{}').getElementsByClassName('dummy')[0],
                  document.getElementById('{}'),
                  {}
                );", src, dst, inner));
        }
    }
    let template = match format {
        Format::Html => &index_hbs,
        Format::Svg => &svg_hbs,
    };
    let output = reg.render_template(template,
        &json!({
            "style": css_style,
            "content": prg,
            "script": leader,
            "arrows": &arrow_txt,
        }),
    )?;
    Ok(output)
}

fn render_program(prg: &Program, hide_heap: bool) -> Result<(String, Vec<ArrowInfo>)> {
    let mut res = String::new();
    res.push_str("<div class=\"program\">");
    let state = RenderState::new(0);
    for (idx, step) in prg.0.iter().enumerate() {
        let mut st = state.with_step_index(idx);
        let piece = render_step(step, &mut st, hide_heap)?;
        res.push_str(&piece);
    }
    res.push_str("</div>");
    let arrows = state.arrows.borrow().clone();
    Ok((res, arrows))
}

fn render_step(step: &Step, state: &mut RenderState, hide_heap: bool) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"step\">");
    res.push_str(&format!("<span class=\"header\">{}</span>", &step.label));
    for location in &step.locations {
        let piece = render_location(location, state, hide_heap)?;
        res.push_str(&piece);
    }
    res.push_str("</div>");
    Ok(res)
}

fn render_location(loc: &Location, state: &mut RenderState, hide_labels: bool) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"location\">");
    res.push_str(&format!("<span class=\"header\">{}</span>", &loc.name));
    // A location either has definitions itself (and no regions) OR it has regions and no definitions
    assert!(loc.definitions.is_empty() || loc.regions.is_empty());
    if !loc.definitions.is_empty() {
        let piece = render_definitions(&loc.definitions, state, hide_labels)?;
        res.push_str(&piece);
    } else {
        for region in &loc.regions {
            let piece = render_region(region, state, hide_labels)?;
            res.push_str(&piece);
        }
    }
    res.push_str("</div>");
    Ok(res)
}

fn render_region(region: &Region, state: &mut RenderState, hide_labels: bool) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"region\">");
    res.push_str(&format!("<span class=\"header\">{}</span>", &region.name));
    let pieces = render_definitions(&region.definitions, state, hide_labels)?;
    res.push_str(&pieces);
    res.push_str("</div>");
    Ok(res)
}

fn render_definitions(
    definitions: &[Def],
    state: &mut RenderState,
    hide_labels: bool,
) -> Result<String> {
    let mut res = String::new();
    for definition in definitions {
        let piece = render_definition(definition, state, hide_labels)?;
        res.push_str(&piece);
    }
    Ok(res)
}

fn render_definition(
    definition: &Def,
    state: &mut RenderState,
    hide_label: bool,
) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"definition\">");
    if !hide_label || !definition.label.starts_with(['H']) {
        res.push_str(&format!(
            "<span class=\"label\">{}</span>",
            &definition.label
        ));
        res.push_str("<span class=\"separator\">:</span>");
    }
    let mut st = state.extend_id(&definition.label);
    let v = render_value(&definition.value, &mut st)?;
    res.push_str(&format!("<div class=\"defvalue\">{}</div>", &v));
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
            res.push_str(&render_values("array_child", v, state)?);
            res.push_str("</div>");
            Ok(res)
        }
        Value::Tuple(v) => {
            let mut res = String::new();
            res.push_str(&format!(
                "<div id=\"{}\" class=\"value tuple\">",
                &state.id_prefix
            ));
            res.push_str(&render_values("tuple_child", v, state)?);
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
            state.add_arrow(&src, &dst, &v.help);
            Ok(format!(
                "<span id=\"{}\" class=\"value pointer\">●<div class=\"dummy\"></div></span>",
                &state.id_prefix,
            ))
        }
        Value::Invalid => Ok(format!(
            "<span id=\"{}\" class=\"value invalid\">❌</span>",
            &state.id_prefix
        )),
    }
}

fn render_values(inner_tag: &str, values: &[Value], state: &mut RenderState) -> Result<String> {
    let mut res = String::new();
    for (idx, value) in values.iter().enumerate() {
        let mut state_p = state.extend_id(&format!("{}", idx));
        let piece = render_value(value, &mut state_p)?;
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
    for (idx, (label, value)) in fields.iter().enumerate() {
        let v = render_field(label, value, &mut state.extend_id(&format!("{}", idx)))?;
        res.push_str(&v);
    }
    res.push_str("</div>");
    Ok(res)
}

fn render_field(label: &str, value: &Value, state: &mut RenderState) -> Result<String> {
    let mut res = String::new();
    res.push_str("<div class=\"field\">");
    res.push_str(&format!("<span class=\"label\">{}</span>", &label));
    res.push_str("<span class=\"separator\">:</span>");
    let v = render_value(value, state)?;
    res.push_str(&v);
    res.push_str("</div>");
    Ok(res)
}
