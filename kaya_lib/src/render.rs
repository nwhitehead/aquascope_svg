use crate::states::{Def, Location, Program, Region, Step, Value};
use anyhow::Result;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cell::RefCell;
use std::rc::Rc;

pub enum Theme {
    Dark,
    Light,
    TransparentDark,
    TransparentLight,
    TransparentNoColor,
}

pub const DEFAULT_GRAVITY: f32 = 80.0;
pub const CSS_STYLE: &[u8] = include_bytes!("./style.css");
pub const LEADER_LINE_JS: &[u8] = include_bytes!("./leader-line-v1.0.7.min.js");
pub const INDEX_HBS: &[u8] = include_bytes!("./index.hbs");
pub const NUM_ARROW_COLORS: usize = 6;
const DEBUG_ARROWS: bool = false;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ArrowInfo {
    src: String,
    dst: String,
    src_help: String,
    dst_help: String,
    src_gravity: f32,
    dst_gravity: f32,
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
        let mut src_gravity = 0.0;
        let mut dst_gravity = 0.0;
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
                ".g0" => {
                    src_gravity = 0.0;
                    dst_gravity = 0.0;
                }
                ".g1" => {
                    src_gravity = 1.0;
                    dst_gravity = 1.0;
                }
                ".g2" => {
                    src_gravity = 2.0;
                    dst_gravity = 2.0;
                }
                ".g3" => {
                    src_gravity = 3.0;
                    dst_gravity = 3.0;
                }
                ".g4" => {
                    src_gravity = 4.0;
                    dst_gravity = 4.0;
                }
                ".g5" => {
                    src_gravity = 5.0;
                    dst_gravity = 5.0;
                }
                ".g6" => {
                    src_gravity = 6.0;
                    dst_gravity = 6.0;
                }
                ".g7" => {
                    src_gravity = 7.0;
                    dst_gravity = 7.0;
                }
                ".g8" => {
                    src_gravity = 8.0;
                    dst_gravity = 8.0;
                }
                ".g9" => {
                    src_gravity = 9.0;
                    dst_gravity = 9.0;
                }
                ".sg0" => src_gravity = 0.0,
                ".sg1" => src_gravity = 1.0,
                ".sg2" => src_gravity = 2.0,
                ".sg3" => src_gravity = 3.0,
                ".sg4" => src_gravity = 4.0,
                ".sg5" => src_gravity = 5.0,
                ".sg6" => src_gravity = 6.0,
                ".sg7" => src_gravity = 7.0,
                ".sg8" => src_gravity = 8.0,
                ".sg9" => src_gravity = 9.0,
                ".dg0" => dst_gravity = 0.0,
                ".dg1" => dst_gravity = 1.0,
                ".dg2" => dst_gravity = 2.0,
                ".dg3" => dst_gravity = 3.0,
                ".dg4" => dst_gravity = 4.0,
                ".dg5" => dst_gravity = 5.0,
                ".dg6" => dst_gravity = 6.0,
                ".dg7" => dst_gravity = 7.0,
                ".dg8" => dst_gravity = 8.0,
                ".dg9" => dst_gravity = 9.0,
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

fn socket_gravity_to_option(x: f32) -> Option<f32> {
    if x == 0.0 {
        return None;
    }
    Some(20.0 * x - 10.0)
}

pub fn arrow_options(info: &ArrowInfo, idx: usize) -> serde_json::Value {
    let ArrowInfo {
        src: _,
        dst: _,
        src_help,
        dst_help,
        src_gravity,
        dst_gravity,
        path,
        color,
    } = info;
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
    let start_socket_gravity = socket_gravity_to_option(*src_gravity);
    let end_socket_gravity = socket_gravity_to_option(*dst_gravity);
    let color_txt = match color {
        None => format!("{}", idx % NUM_ARROW_COLORS),
        Some(c) => format!("{}", (*c as usize) % NUM_ARROW_COLORS),
    };
    json!({
        "startSocket": start_socket,
        "endSocket": end_socket,
        "startSocketGravity": start_socket_gravity,
        "endSocketGravity": end_socket_gravity,
        "color": color_txt,
        "path": path,
    })
}

pub fn render_arrow(info: &ArrowInfo, idx: usize) -> String {
    let ArrowInfo {
        src,
        dst,
        src_help,
        dst_help,
        src_gravity,
        dst_gravity,
        path,
        color,
    } = info;
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
    let start_socket_gravity = socket_gravity_to_option(*src_gravity).unwrap_or(DEFAULT_GRAVITY);
    let end_socket_gravity = socket_gravity_to_option(*dst_gravity).unwrap_or(DEFAULT_GRAVITY);
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
        start_socket, end_socket, start_socket_gravity, end_socket_gravity, color_txt, path
    );
    if src != dst {
        format!(
            "new LeaderLine(
                document.getElementById('{}'),
                document.getElementById('{}'),
                {}
            );",
            src, dst, inner
        )
    } else {
        format!(
            "new LeaderLine(
                document.getElementById('{}').getElementsByClassName('dummy')[0],
                document.getElementById('{}'),
                {}
            );",
            src, dst, inner
        )
    }
}

pub fn render_arrows(arrows: Vec<ArrowInfo>) -> Result<String> {
    let mut arrow_txt = String::new();
    for (idx, arrow_info) in arrows.iter().enumerate() {
        arrow_txt.push_str(&render_arrow(arrow_info, idx));
    }
    Ok(arrow_txt)
}

pub fn render_parts(prg: &Program, show_heap: bool) -> Result<(String, String)> {
    let (prg, arrows) = render_program(prg, !show_heap)?;
    if DEBUG_ARROWS {
        println!("arrows = {:?}", &arrows);
    }
    let arrow_txt = render_arrows(arrows)?;
    Ok((prg, arrow_txt))
}

pub fn render(prg: &Program, show_heap: bool) -> Result<String> {
    let leader = String::from_utf8(LEADER_LINE_JS.to_vec())?;
    let css_style = String::from_utf8(CSS_STYLE.to_vec())?;
    let index_hbs = String::from_utf8(INDEX_HBS.to_vec())?;
    let reg = Handlebars::new();

    let (prg_txt, arrow_txt) = render_parts(prg, show_heap)?;

    let output = reg.render_template(
        &index_hbs,
        &json!({
            "style": css_style,
            "content": prg_txt,
            "script": leader,
            "arrows": arrow_txt,
        }),
    )?;
    Ok(output)
}

pub fn render_program(prg: &Program, hide_heap: bool) -> Result<(String, Vec<ArrowInfo>)> {
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
