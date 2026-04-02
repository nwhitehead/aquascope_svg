use ab_glyph::{Rect, point};
use anyhow::{Result, bail};
use std::collections::HashMap;
use tiny_skia::{Color, ColorU8};

use crate::canvas::Canvas;
use crate::draw::{
    Drawable, FormulaType, GArray, GLine, GPadding, GSpace, GTagged, GText, border, compute_align, hstack,
    hstack_top, vstack, vstack_left, vstack_none,
};
use crate::draw_state::DrawState;
use crate::style::Styling;

use kaya_lib::states::{Def, Location, NamedStruct, Program, Ptr, Region, Step, Value};

#[derive(Clone, Debug)]
pub struct RenderState {
    locations: HashMap<String, Rect>,
    style: Styling,
    skip_heap: bool,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            locations: Default::default(),
            style: Default::default(),
            skip_heap: true,
        }
    }
}

fn max_height(values: &Vec<Box<dyn Drawable>>, canvas: &Canvas) -> Result<f32> {
    let mut res: f32 = 0.0;
    for value in values {
        let bb = value.bounding_box(canvas)?;
        let h = bb.max.y - bb.min.y;
        res = res.max(h);
    }
    Ok(res)
}

fn render_value_array(
    a: &Vec<Value>,
    prefix: &str,
    render_state: &mut RenderState,
    canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    // Draw all the parts separately
    let mut a_draws: Vec<Box<dyn Drawable>> = vec![];
    for (idx, x) in a.iter().enumerate() {
        let draw = render_value(x, &format!("{}.{}", prefix, idx), render_state, canvas)?;
        a_draws.push(draw);
    }
    let style = &render_state.style;
    if a_draws.is_empty() {
        a_draws.push(
            GSpace::new(
                style.get_number_or("value.array.empty.w", 5.0),
                style.get_number_or("value.array.empty.h", 5.0),
            )
            .clone_box(),
        );
    }
    let mut ds = DrawState::default();
    // Now measure the height for divider lines
    let h = max_height(&a_draws, canvas)?;
    let sep_margin = style.get_number_or("value.array.separator.vmargin", 5.0);
    // intersperse vertical lines
    ds.stroke_color = style.get_color_or("value.array.separator.color", color("#000")?);
    let sep = GLine::new(point(0.0, 0.0), point(0.0, h - sep_margin), ds.clone());
    let sep_padding = style.get_padding("value.array.separator.padding", 5.0);
    let padded_sep = GPadding::new(Box::new(sep), sep_padding);
    let mut a_draws_sep: Vec<Box<dyn Drawable>> = vec![];
    let mut any_elems_yet = false;
    for x in a_draws {
        if any_elems_yet {
            a_draws_sep.push(padded_sep.clone_box());
        } else {
            any_elems_yet = true;
        }
        a_draws_sep.push(x);
    }
    let stk = hstack(a_draws_sep, canvas)?;
    ds.padding = style.get_padding("value.array.padding", 5.0);
    ds.stroke_color = style.get_color_or("value.array.border.color", color("#000")?);
    ds.stroke.width = style.get_number_or("value.array.border.width", 4.0);
    ds.border_radius = style.get_radius("value.array.border.radius", 5.0);
    let res = border(Box::new(stk), canvas, ds)?;
    Ok(res)
}

fn render_value_tuple(
    a: &Vec<Value>,
    prefix: &str,
    render_state: &mut RenderState,
    canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    // Draw all the parts separately
    let mut a_draws: Vec<Box<dyn Drawable>> = vec![];
    for (idx, x) in a.into_iter().enumerate() {
        let draw = render_value(x, &format!("{}.{}", prefix, idx), render_state, canvas)?;
        a_draws.push(draw);
    }
    let style = &render_state.style;
    if a_draws.is_empty() {
        a_draws.push(
            GSpace::new(
                style.get_number_or("value.tuple.empty.w", 5.0),
                style.get_number_or("value.tuple.empty.h", 5.0),
            )
            .clone_box(),
        );
    }
    let mut ds = DrawState::default();
    // Now measure the height for divider lines
    let h = max_height(&a_draws, canvas)?;
    let sep_margin = style.get_number_or("value.tuple.separator.vmargin", 5.0);
    // intersperse vertical lines
    ds.stroke_color = style.get_color_or("value.tuple.separator.color", color("#000")?);
    let sep = GLine::new(point(0.0, 0.0), point(0.0, h - sep_margin), ds.clone());
    let sep_padding = style.get_padding("value.tuple.separator.padding", 5.0);
    let padded_sep = GPadding::new(Box::new(sep), sep_padding);
    let mut a_draws_sep: Vec<Box<dyn Drawable>> = vec![];
    let mut any_elems_yet = false;
    for x in a_draws {
        if any_elems_yet {
            a_draws_sep.push(padded_sep.clone_box());
        } else {
            any_elems_yet = true;
        }
        a_draws_sep.push(x);
    }
    let stk = hstack(a_draws_sep, canvas)?;
    ds.padding = style.get_padding("value.tuple.padding", 5.0);
    ds.stroke_color = style.get_color_or("value.tuple.border.color", color("#000")?);
    ds.stroke.width = style.get_number_or("value.tuple.border.width", 4.0);
    ds.border_radius = style.get_radius("value.tuple.border.radius", 5.0);
    let res = border(Box::new(stk), canvas, ds)?;
    Ok(res)
}

fn render_def(
    def: &Def,
    prefix: &str,
    render_state: &mut RenderState,
    canvas: &Canvas,
    skip_heap: bool,
) -> Result<Box<dyn Drawable>> {
    let style = &render_state.style;
    let mut ds = DrawState::default();
    ds.font = style.get_string_or("def.label.font", "mono");
    ds.font_size = style.get_number_or("def.label.font_size", 24.0);
    ds.text_color = style.get_color_or("def.label.color", color("#000")?);

    let mut g_label = GText::new(&def.label, point(0.0, 0.0), ds.clone());

    ds.font = style.get_string_or("def.separator.font", "mono");
    ds.font_size = style.get_number_or("def.separator.font_size", 24.0);
    ds.text_color = style.get_color_or("def.separator.color", color("#000")?);
    let sep_text = style.get_string_or("def.separator.text", ":");
    let g_separator = GText::new(&sep_text, point(0.0, 0.0), ds.clone());

    let sep_padding = style.get_padding("def.separator.padding", 0.0);
    let g_padded_sep = GPadding::new(Box::new(g_separator), sep_padding);

    // Make sure final drawable has x=0 as the dividing line for separator
    // (so we can align them later)
    // Instead of just doing hstack_... we compute the translation and use parts of it
    // Move label left to align right side to separator
    let label_bb = g_label.bounding_box(canvas)?;
    let sep_bb = g_padded_sep.bounding_box(canvas)?;
    let p = compute_align(
        &label_bb,
        &sep_bb,
        FormulaType::Sequenced,
        FormulaType::Centered,
    );
    g_label.translate(point(-p.x, -p.y));
    let mut left = GArray::new();
    left.push(Box::new(g_label));
    left.push(Box::new(g_padded_sep));

    let left_padding = style.get_padding("def.left.padding", 0.0);
    let g_left = GPadding::new(Box::new(left), left_padding);
    let left_bb = g_left.bounding_box(canvas)?;

    ds.padding = style.get_padding("def.value.padding", 0.0);
    ds.margin = style.get_padding("def.value.margin", 0.0);
    ds.stroke_color = style.get_color_or("def.value.border.color", color("#000")?);
    ds.stroke.width = style.get_number_or("def.value.border.width", 4.0);
    ds.border_radius = style.get_radius("def.value.border.radius", 5.0);
    let prefix = &format!("{}:{}", prefix, def.label);
    let g_value = render_value(&def.value, prefix, render_state, canvas)?;
    let mut g_border_value = border(g_value, canvas, ds.clone())?;

    // Now align the value to right of separator, centered vertically
    let value_bb = g_border_value.bounding_box(canvas)?;
    let p = compute_align(
        &left_bb,
        &value_bb,
        FormulaType::Sequenced,
        FormulaType::Centered,
    );
    g_border_value.translate(p);

    let mut g_array = GArray::new();
    // if skip_heap is off, always show label
    // if skip_heap is on, show label if it doesn't start with H
    if !skip_heap || !def.label.starts_with("H") {
        g_array.push(Box::new(g_left));
    }
    g_array.push(g_border_value);
    Ok(Box::new(g_array))
}

fn render_value_struct(
    named_struct: &NamedStruct,
    prefix: &str,
    render_state: &mut RenderState,
    canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    // Draw all the inner values separately first (to measure for sep line height)
    let mut v_draws: Vec<Box<dyn Drawable>> = vec![];
    for (idx, p) in named_struct.fields.clone().into_iter().enumerate() {
        let label = &p.0;
        let value = &p.1;
        let prefix = &format!("{}.{}", prefix, idx);
        let draw = render_value(value, prefix, render_state, canvas)?;
        v_draws.push(draw);
    }
    // Now measure the height for divider lines
    let h = max_height(&v_draws, canvas)?;

    let mut ds = DrawState::default();
    let style = &render_state.style;
    ds.font = style.get_string_or("value.struct.name.font", "mono");
    ds.font_size = style.get_number_or("value.struct.name.font_size", 24.0);
    ds.text_color = style.get_color_or("value.struct.name.color", color("#000")?);

    // intersperse vertical lines
    let div_margin = style.get_number_or("value.struct.divider.vmargin", 5.0);
    ds.stroke_color = style.get_color_or("value.struct.divider.color", color("#000")?);
    let div = GLine::new(point(0.0, 0.0), point(0.0, h - div_margin), ds.clone());
    let div_padding = style.get_padding("value.struct.divider.padding", 5.0);
    let padded_div = GPadding::new(Box::new(div), div_padding);

    let g_name = GText::new(&named_struct.name, point(0.0, 0.0), ds.clone());

    let mut body: Vec<Box<dyn Drawable>> = vec![];

    for (i, p) in named_struct.fields.iter().enumerate() {
        let label = &p.0;
        let value = &p.1;
        ds.font = style.get_string_or("value.struct.label.font", "mono");
        ds.font_size = style.get_number_or("value.struct.label.font_size", 24.0);
        ds.text_color = style.get_color_or("value.struct.label.color", color("#000")?);
        let g_label = GText::new(&label, point(0.0, 0.0), ds.clone());

        ds.font = style.get_string_or("value.struct.separator.font", "mono");
        ds.font_size = style.get_number_or("value.struct.separator.font_size", 24.0);
        ds.text_color = style.get_color_or("value.struct.separator.color", color("#000")?);
        let sep_text = style.get_string_or("value.struct.separator.text", ":");
        let g_separator = GText::new(&sep_text, point(0.0, 0.0), ds.clone());

        let sep_padding = style.get_padding("value.struct.separator.padding", 0.0);
        let g_padded_sep = GPadding::new(Box::new(g_separator), sep_padding);

        let mut left: Vec<Box<dyn Drawable>> = vec![];
        left.push(Box::new(g_label));
        left.push(Box::new(g_padded_sep));
        let g_left = hstack(left, canvas)?;
        let left_padding = style.get_padding("value.struct.left.padding", 0.0);
        let g_padded_left = GPadding::new(Box::new(g_left), left_padding);

        if i > 0 {
            // Add dividing line after 1st element before each field
            body.push(padded_div.clone_box());
        }
        body.push(Box::new(g_padded_left));
        body.push(v_draws[i].clone_box());
    }
    let g_body = hstack(body, canvas)?;

    ds.padding = style.get_padding("value.struct.padding", 0.0);
    ds.margin = style.get_padding("value.struct.margin", 0.0);
    ds.stroke_color = style.get_color_or("value.struct.border.color", color("#000")?);
    ds.stroke.width = style.get_number_or("value.struct.border.width", 4.0);
    ds.border_radius = style.get_radius("value.struct.border.radius", 5.0);
    let g_border_body = border(Box::new(g_body), canvas, ds.clone())?;

    let g_array = hstack(vec![Box::new(g_name), g_border_body], canvas)?;
    Ok(Box::new(g_array))
}

fn render_value_invalid(
    prefix: &str,
    render_state: &mut RenderState,
    canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    let style = &render_state.style;
    let mut ds = DrawState::default();
    ds.font = style.get_string_or("value.invalid.font", "mono");
    ds.text_color = style.get_color_or("value.invalid.color", color("#000")?);
    ds.font_size = style.get_number_or("value.invalid.font_size", 24.0);
    let padding = style.get_padding("value.invalid.padding", 5.0);
    let text = format!("×");
    let gtxt = GText::new(&text, point(0.0, 0.0), ds);
    let padded_gtxt = GPadding::new(Box::new(gtxt), padding);
    Ok(Box::new(padded_gtxt))
}

fn render_value_number(
    v: f64,
    prefix: &str,
    render_state: &mut RenderState,
    _canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    let style = &render_state.style;
    let mut ds = DrawState::default();
    ds.font = style.get_string_or("value.number.font", "mono");
    ds.text_color = style.get_color_or("value.number.color", color("#000")?);
    ds.font_size = style.get_number_or("value.number.font_size", 24.0);
    let padding = style.get_padding("value.number.padding", 5.0);
    let text = format!("{}", v);
    let gtxt = GText::new(&text, point(0.0, 0.0), ds);
    let padded_gtxt = GPadding::new(Box::new(gtxt), padding);
    Ok(Box::new(padded_gtxt))
}

fn render_value_char(
    c: char,
    prefix: &str,
    render_state: &mut RenderState,
    _canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    let style = &render_state.style;
    let mut ds = DrawState::default();
    ds.font = style.get_string_or("value.char.font", "mono");
    ds.text_color = style.get_color_or("value.char.color", color("#000")?);
    ds.font_size = style.get_number_or("value.char.font_size", 24.0);
    let text = format!("'{}'", c);
    Ok(Box::new(GText::new(&text, point(0.0, 0.0), ds)))
}

fn render_value_pointer(
    _p: &Ptr,
    prefix: &str,
    render_state: &mut RenderState,
    _canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    let style = &render_state.style;
    let mut ds = DrawState::default();
    ds.font = style.get_string_or("value.pointer.font", "mono");
    ds.text_color = style.get_color_or("value.pointer.color", color("#000")?);
    ds.font_size = style.get_number_or("value.pointer.font_size", 24.0);
    // ✕✖✗✘×•●○◯42
    let text = "●";
    Ok(Box::new(GText::new(text, point(0.0, 0.0), ds)))
}

pub fn render_value(
    value: &Value,
    prefix: &str,
    render_state: &mut RenderState,
    canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    let item = match value {
        Value::Number(v) => render_value_number(*v, prefix, render_state, canvas)?,
        Value::Char(c) => render_value_char(*c, prefix, render_state, canvas)?,
        Value::Pointer(p) => render_value_pointer(p, prefix, render_state, canvas)?,
        Value::Array(a) => render_value_array(a, prefix, render_state, canvas)?,
        Value::Tuple(a) => render_value_tuple(a, prefix, render_state, canvas)?,
        Value::Struct(a) => render_value_struct(a, prefix, render_state, canvas)?,
        Value::Invalid => render_value_invalid(prefix, render_state, canvas)?,
        _ => panic!("not handled"),
    };
    let tagged = GTagged::new(item, prefix);
    Ok(Box::new(tagged))
}

pub fn render_region(
    value: &Region,
    prefix: &str,
    render_state: &mut RenderState,
    canvas: &Canvas,
    skip_heap: bool,
) -> Result<Box<dyn Drawable>> {
    // Header
    let style = &render_state.style;
    let mut ds = DrawState::default();
    ds.font = style.get_string_or("region.header.font", "serif");
    ds.text_color = style.get_color_or("region.header.color", color("#000")?);
    ds.font_size = style.get_number_or("region.header.font_size", 24.0);
    let header_padding = style.get_padding("region.header.padding", 5.0);
    let region_padding = style.get_padding("region.padding", 5.0);
    let text = format!("{}", value.name);
    let gtxt = GText::new(&text, point(0.0, 0.0), ds);
    let padded_gtxt = GPadding::new(Box::new(gtxt), header_padding);
    // Body
    let mut body: Vec<Box<dyn Drawable>> = vec![];
    for def in &value.definitions {
        let g_def = render_def(&def, prefix, render_state, canvas, skip_heap)?;
        body.push(g_def);
    }
    // stack vertically without moving horizontally (to keep : aligned)
    let g_body = vstack_none(body, canvas)?;
    let g_final = vstack_left(vec![Box::new(padded_gtxt), Box::new(g_body)], canvas)?;
    let g_padded_final = GPadding::new(Box::new(g_final), region_padding);
    Ok(Box::new(g_padded_final))
}

pub fn render_location(
    value: &Location,
    prefix: &str,
    render_state: &mut RenderState,
    canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    if !value.definitions.is_empty() {
        let region = Region {
            name: "".to_string(),
            definitions: value.definitions.clone(),
        };
        return render_location(
            &Location {
                name: value.name.clone(),
                definitions: vec![],
                regions: vec![region],
            },
            prefix,
            render_state,
            canvas,
        );
    }

    let mut skip_heap = render_state.skip_heap;
    if value.name.starts_with("Stack") {
        skip_heap = false;
    }

    // Header
    let style = &render_state.style;
    let mut ds = DrawState::default();
    ds.font = style.get_string_or("location.header.font", "serif");
    ds.text_color = style.get_color_or("location.header.color", color("#000")?);
    ds.font_size = style.get_number_or("location.header.font_size", 24.0);
    let header_padding = style.get_padding("location.header.padding", 5.0);
    let region_padding = style.get_padding("location.padding", 5.0);
    let text = format!("{}", value.name);
    let gtxt = GText::new(&text, point(0.0, 0.0), ds);
    let padded_gtxt = GPadding::new(Box::new(gtxt), header_padding);

    // Body
    let style = &render_state.style;
    let mut body: Vec<Box<dyn Drawable>> = vec![];
    for region in &value.regions {
        let g_region = render_region(&region, prefix, render_state, canvas, skip_heap)?;
        body.push(g_region);
    }
    let g_body = vstack_left(body, canvas)?;
    let g_final = vstack_left(vec![Box::new(padded_gtxt), Box::new(g_body)], canvas)?;

    Ok(Box::new(g_final))
}

pub fn render_step(
    value: &Step,
    render_state: &mut RenderState,
    canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {

    // Label
    let style = &render_state.style;
    let mut ds = DrawState::default();
    ds.font = style.get_string_or("step.header.font", "serif");
    ds.text_color = style.get_color_or("step.header.color", color("#000")?);
    ds.font_size = style.get_number_or("step.header.font_size", 24.0);
    let padding = style.get_padding("step.header.padding", 5.0);
    let text = format!("{}", value.label);
    let g_text = GText::new(&text, point(0.0, 0.0), ds.clone());
    let g_padded_text = GPadding::new(Box::new(g_text), padding);

    // draw border on right side sized properly
    ds.stroke_color = style.get_color_or("step.separator.color", color("#000")?);
    let h = style.get_number_or("step.separator.size", 5.0);
    let sep = GLine::new(point(0.0, 0.0), point(0.0, h), ds.clone());
    let sep_padding = style.get_padding("step.separator.padding", 5.0);
    let g_padded_sep = GPadding::new(Box::new(sep), sep_padding);

    let style = &render_state.style;
    let mut body: Vec<Box<dyn Drawable>> = vec![];
    body.push(Box::new(g_padded_text));
    body.push(Box::new(g_padded_sep));
    let gap = style.get_number_or("step.location.gap", 5.0);
    for (idx, location) in value.locations.iter().enumerate() {
        let g_location = render_location(&location, &format!("{}", value.label), render_state, canvas)?;
        if idx > 0 {
            body.push(Box::new(GSpace::new(gap, 0.0)));
        }
        body.push(g_location);
    }
    let g_body = hstack_top(body, canvas)?;

    let style = &render_state.style;
    ds.padding = style.get_padding("step.padding", 5.0);
    ds.margin = style.get_padding("step.margin", 5.0);
    ds.stroke_color = style.get_color_or("step.border.color", color("#000")?);
    ds.stroke.width = style.get_number_or("step.border.width", 4.0);
    ds.border_radius = style.get_radius("step.border.radius", 5.0);
    let res = border(Box::new(g_body), canvas, ds)?;

    Ok(res)
}

pub fn render_program(
    value: &Program,
    render_state: &mut RenderState,
    canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {

    let style = &render_state.style;
    let mut body: Vec<Box<dyn Drawable>> = vec![];
    let gap = style.get_number_or("program.step.gap", 5.0);
    for (idx, step) in value.0.iter().enumerate() {
        let g_location = render_step(&step, render_state, canvas)?;
        if idx > 0 {
            body.push(Box::new(GSpace::new(0.0, gap)));
        }
        body.push(g_location);
    }
    let g_body = vstack_left(body, canvas)?;
    Ok(Box::new(g_body))
}

fn color(txt: &str) -> Result<ColorU8> {
    let r;
    let g;
    let b;
    let mut a = 255;
    if !txt.starts_with('#') {
        bail!("colors must start with #");
    }
    let txt = txt.trim_start_matches('#');
    match txt.len() {
        3 => {
            r = u8::from_str_radix(&txt[0..1], 16)? * (0x10 + 0x01);
            g = u8::from_str_radix(&txt[1..2], 16)? * (0x10 + 0x01);
            b = u8::from_str_radix(&txt[2..3], 16)? * (0x10 + 0x01);
        }
        4 => {
            r = u8::from_str_radix(&txt[0..1], 16)? * (0x10 + 0x01);
            g = u8::from_str_radix(&txt[1..2], 16)? * (0x10 + 0x01);
            b = u8::from_str_radix(&txt[2..3], 16)? * (0x10 + 0x01);
            a = u8::from_str_radix(&txt[3..4], 16)? * (0x10 + 0x01);
        }
        6 => {
            r = u8::from_str_radix(&txt[0..2], 16)?;
            g = u8::from_str_radix(&txt[2..4], 16)?;
            b = u8::from_str_radix(&txt[4..6], 16)?;
        }
        8 => {
            r = u8::from_str_radix(&txt[0..2], 16)?;
            g = u8::from_str_radix(&txt[2..4], 16)?;
            b = u8::from_str_radix(&txt[4..6], 16)?;
            a = u8::from_str_radix(&txt[6..8], 16)?;
        }
        _ => bail!("unknown color length"),
    }
    Ok(ColorU8::from_rgba(r, g, b, a))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_color() -> Result<()> {
        assert_eq!(color("#000")?, ColorU8::from_rgba(0, 0, 0, 0xff));
        assert_eq!(color("#f00")?, ColorU8::from_rgba(0xff, 0, 0, 0xff));
        assert_eq!(color("#080")?, ColorU8::from_rgba(0, 0x88, 0, 0xff));
        assert_eq!(color("#00a")?, ColorU8::from_rgba(0, 0, 0xaa, 0xff));
        assert_eq!(color("#0008")?, ColorU8::from_rgba(0, 0, 0, 0x88));
        assert_eq!(color("#1234")?, ColorU8::from_rgba(0x11, 0x22, 0x33, 0x44));
        assert_eq!(
            color("#123456")?,
            ColorU8::from_rgba(0x12, 0x34, 0x56, 0xff)
        );
        assert_eq!(
            color("#12345678")?,
            ColorU8::from_rgba(0x12, 0x34, 0x56, 0x78)
        );
        assert!(color("000").is_err());
        assert!(color("#12345").is_err());
        assert!(color("#0g3456").is_err());
        assert!(color("#fffffffff").is_err());
        Ok(())
    }

    #[test]
    pub fn test_render_alpha() -> Result<()> {
        let mut canvas = Canvas::new(800, 800)?;
        canvas
            .pixmap
            .fill(Color::from_rgba(0.2, 0.1, 0.3, 1.0).unwrap());

        canvas.load_font(
            "mono",
            include_bytes!("../fonts/DejaVu/DejaVuSansMono-Bold.ttf"),
        )?;
        canvas.load_font("serif", include_bytes!("../fonts/Lato/Lato-Regular.ttf"))?;

        let mut rs = RenderState::default();
        rs.style.add_string("value.number.font", "mono");
        rs.style.add_number("value.number.font_size", 48.0);
        rs.style
            .add_color("value.number.color", color("#bccfa980")?);

        let mut v = render_value(&Value::Number(42.0), "", &mut rs, &canvas)?;
        v.translate(point(400.0, 400.0));
        v.draw(&mut canvas)?;
        v.translate(point(10.0, 5.0));
        v.draw(&mut canvas)?;
        v.translate(point(10.0, 5.0));
        v.draw(&mut canvas)?;

        rs.style
            .add_color("value.number.color", color("#cfa9bc80")?);
        let mut v2 = render_value(&Value::Number(67.0), "", &mut rs, &canvas)?;
        v2.translate(point(400.0, 430.0));
        v2.draw(&mut canvas)?;
        v2.translate(point(10.0, -7.0));
        v2.draw(&mut canvas)?;
        v2.translate(point(10.0, -7.0));
        v2.draw(&mut canvas)?;

        canvas.save("test_render_alpha.png")?;

        Ok(())
    }

    #[test]
    pub fn test_render_value() -> Result<()> {
        let mut canvas = Canvas::new(2048, 2048)?;
        canvas
            .pixmap
            .fill(Color::from_rgba8(0x19, 0x19, 0x19, 0xff));
        canvas.load_font(
            "mono",
            include_bytes!("../fonts/DejaVu/DejaVuSansMono-Bold.ttf"),
        )?;
        canvas.load_font("serif", include_bytes!("../fonts/Lato/Lato-Regular.ttf"))?;
        canvas.load_font("serif_bold", include_bytes!("../fonts/Lato/Lato-Bold.ttf"))?;

        let mut rs = RenderState::default();
        rs.style.add_string("value.number.font", "mono");
        rs.style.add_number("value.number.font_size", 23.0);
        rs.style.add_color("value.number.color", color("#bccfa9")?);
        rs.style.add_number("value.number.padding", 5.0);
        rs.style.add_number("value.number.padding.bottom", 8.0);
        rs.style.add_string("value.char.font", "mono");
        rs.style.add_number("value.char.font_size", 23.0);
        rs.style.add_color("value.char.color", color("#bf947a")?);
        rs.style.add_string("value.pointer.font", "mono");
        rs.style.add_number("value.pointer.font_size", 23.0);
        rs.style.add_color("value.pointer.color", color("#ccc")?);

        rs.style.add_number("value.array.empty.w", 0.0);
        rs.style.add_number("value.array.empty.h", 20.0);
        rs.style
            .add_color("value.array.separator.color", color("#7197d580")?);
        rs.style.add_number("value.array.separator.vmargin", 5.0);
        rs.style
            .add_number("value.array.separator.padding.left", 10.0);
        rs.style
            .add_number("value.array.separator.padding.top", 5.0);
        rs.style
            .add_number("value.array.separator.padding.right", 10.0);
        rs.style
            .add_number("value.array.separator.padding.bottom", 5.0);
        rs.style.add_number("value.array.padding.left", 10.0);
        rs.style.add_number("value.array.padding.top", 2.0);
        rs.style.add_number("value.array.padding.right", 10.0);
        rs.style.add_number("value.array.padding.bottom", 2.0);
        rs.style
            .add_color("value.array.border.color", color("#7197d5")?);
        rs.style.add_number("value.array.border.width", 1.5);
        rs.style.add_number("value.array.border.radius", 5.0);
        rs.style.add_number("value.tuple.empty.w", 0.0);
        rs.style.add_number("value.tuple.empty.h", 20.0);

        rs.style
            .add_color("value.tuple.separator.color", color("#b785c080")?);
        rs.style.add_number("value.tuple.separator.vmargin", 5.0);
        rs.style
            .add_number("value.tuple.separator.padding.left", 10.0);
        rs.style
            .add_number("value.tuple.separator.padding.top", 5.0);
        rs.style
            .add_number("value.tuple.separator.padding.right", 10.0);
        rs.style
            .add_number("value.tuple.separator.padding.bottom", 5.0);
        rs.style.add_number("value.tuple.padding.left", 10.0);
        rs.style.add_number("value.tuple.padding.top", 2.0);
        rs.style.add_number("value.tuple.padding.right", 10.0);
        rs.style.add_number("value.tuple.padding.bottom", 2.0);
        rs.style
            .add_color("value.tuple.border.color", color("#b785c0")?);
        rs.style.add_number("value.tuple.border.width", 1.5);
        rs.style.add_number("value.tuple.border.radius", 5.0);
        rs.style.add_number("value.tuple.border.radius.nw", 0.0);
        rs.style.add_number("value.tuple.border.radius.se", 0.0);

        rs.style.add_string("def.label.font", "mono");
        rs.style.add_number("def.label.font_size", 23.0);
        rs.style.add_color("def.label.color", color("#b2d9fd")?);
        rs.style.add_string("def.separator.font", "mono");
        rs.style.add_number("def.separator.font_size", 23.0);
        rs.style.add_color("def.separator.color", color("#ccc")?);
        rs.style.add_string("def.separator.text", ":");
        rs.style.add_number("def.separator.padding.left", 3.0);
        rs.style.add_number("def.separator.padding.right", 5.0);
        rs.style.add_number("def.left.padding.bottom", 3.0);
        rs.style.add_number("def.value.padding", 8.0);
        rs.style.add_number("def.value.margin", 0.0);
        rs.style.add_number("def.value.margin.top", 3.0);
        rs.style.add_number("def.value.margin.bottom", 3.0);
        rs.style
            .add_color("def.value.border.color", color("#282828")?);
        rs.style.add_number("def.value.border.width", 1.5);
        rs.style.add_number("def.value.border.radius", 5.0);

        rs.style.add_string("value.struct.name.font", "mono");
        rs.style.add_number("value.struct.name.font_size", 23.0);
        rs.style
            .add_color("value.struct.name.color", color("#7fc8b0")?);
        rs.style.add_string("value.struct.label.font", "mono");
        rs.style.add_number("value.struct.label.font_size", 23.0);
        rs.style
            .add_color("value.struct.label.color", color("#7fc8b0")?);
        rs.style.add_string("value.struct.separator.font", "mono");
        rs.style
            .add_number("value.struct.separator.font_size", 23.0);
        rs.style
            .add_color("value.struct.separator.color", color("#ccc")?);
        rs.style.add_string("value.struct.separator.text", ":");
        rs.style
            .add_number("value.struct.separator.padding.left", 3.0);
        rs.style
            .add_number("value.struct.separator.padding.right", 3.0);
        rs.style.add_number("value.struct.padding", 10.0);
        rs.style.add_number("value.struct.margin.left", 10.0);
        rs.style
            .add_color("value.struct.border.color", color("#789a56")?);
        rs.style.add_number("value.struct.border.width", 1.5);
        rs.style.add_number("value.struct.border.radius", 5.0);
        rs.style.add_number("value.struct.left.padding.bottom", 3.0);

        rs.style.add_number("value.struct.divider.vmargin", 0.0);
        rs.style.add_number("value.struct.divider.padding", 0.0);
        rs.style
            .add_number("value.struct.divider.padding.left", 7.0);
        rs.style
            .add_number("value.struct.divider.padding.right", 12.0);
        rs.style
            .add_color("value.struct.divider.color", color("#789a5680")?);

        rs.style.add_string("value.invalid.font", "mono");
        rs.style.add_number("value.invalid.font_size", 48.0);
        rs.style.add_color("value.invalid.color", color("#e44")?);
        rs.style.add_number("value.invalid.padding", 0.0);
        rs.style.add_number("value.invalid.padding.bottom", 10.0);

        rs.style.add_string("region.header.font", "serif");
        rs.style.add_number("region.header.font_size", 23.0);
        rs.style.add_color("region.header.color", color("#ccc")?);
        rs.style.add_number("region.header.padding", 0.0);
        rs.style.add_number("region.header.padding.top", 10.0);
        rs.style.add_number("region.header.padding.bottom", 10.0);
        rs.style.add_number("region.padding", 0.0);

        rs.style.add_number("location.region.gap", 25.0);
        rs.style.add_string("location.header.font", "serif_bold");
        rs.style.add_number("location.header.font_size", 28.0);
        rs.style.add_color("location.header.color", color("#ccc")?);
        rs.style.add_number("location.header.padding", 0.0);

        rs.style.add_string("step.header.font", "serif_bold");
        rs.style.add_number("step.header.font_size", 28.0);
        rs.style.add_color("step.header.color", color("#dbdeab")?);
        rs.style.add_number("step.header.padding", 3.0);
        rs.style.add_number("step.header.padding.right", 20.0);
        rs.style.add_color("step.separator.color", color("#404040")?);
        rs.style.add_number("step.separator.size", 26.0);
        rs.style.add_number("step.separator.padding", 5.0);
        rs.style.add_number("step.separator.padding.right", 25.0);
        rs.style.add_number("step.location.gap", 40.0);
        rs.style.add_number("step.padding", 20.0);
        rs.style.add_number("step.padding.left", 40.0);
        rs.style.add_number("step.padding.right", 40.0);
        rs.style.add_number("step.margin", 5.0);
        rs.style.add_color("step.border.color", color("#404040")?);
        rs.style.add_number("step.border.width", 1.5);
        rs.style.add_number("step.border.radius", 5.0);
        rs.style.add_number("program.step.gap", 5.0);

        let mut v = render_value(&Value::Number(42.0), "", &mut rs, &canvas)?;
        v.translate(point(200.0, 200.0));
        v.draw(&mut canvas)?;

        let mut v = render_value(&Value::Char('H'), "", &mut rs, &canvas)?;
        v.translate(point(250.0, 200.0));
        v.draw(&mut canvas)?;

        let mut v = render_value(
            &Value::Pointer(Ptr {
                name: "".to_string(),
                selectors: vec![],
                borrow: 0,
                help: vec![],
            }),
            "",
            &mut rs,
            &canvas,
        )?;
        v.translate(point(300.0, 200.0));
        v.draw(&mut canvas)?;

        let mut v = render_value(
            &Value::Array(vec![
                Value::Number(42.0),
                Value::Number(67.0),
                Value::Tuple(vec![]),
                Value::Tuple(vec![
                    Value::Char('C'),
                    Value::Number(4.0),
                    Value::Array(vec![]),
                ]),
            ]),
            "",
            &mut rs,
            &canvas,
        )?;
        v.translate(point(350.0, 200.0));
        v.draw(&mut canvas)?;

        let mut v = render_def(
            &Def {
                label: "a".to_string(),
                value: Value::Array(vec![Value::Number(42.0), Value::Invalid]),
            },
            "",
            &mut rs,
            &canvas,
            true,
        )?;
        v.translate(point(200.0, 300.0));
        v.draw(&mut canvas)?;

        let mut v = render_def(
            &Def {
                label: "x".to_string(),
                value: Value::Struct(NamedStruct {
                    name: "Rect".to_string(),
                    fields: vec![
                        ("pos".to_string(), Value::Number(42.0)),
                        ("w".to_string(), Value::Number(3.0)),
                    ],
                }),
            },
            "",
            &mut rs,
            &canvas,
            true,
        )?;
        v.translate(point(200.0, 380.0));
        v.draw(&mut canvas)?;

        let mut v = render_program(
            &Program(
                vec![
                    Step {
                        label: "L0".to_string(),
                        locations: vec![
                            Location {
                                name: "Stack".to_string(),
                                definitions: vec![],
                                regions: vec![
                                    Region {
                                        name: "main".to_string(),
                                        definitions: vec![
                                            Def {
                                                label: "x".to_string(),
                                                value: Value::Struct(NamedStruct {
                                                    name: "Rect".to_string(),
                                                    fields: vec![
                                                        ("pos".to_string(), Value::Number(42.0)),
                                                        ("w".to_string(), Value::Number(3.0)),
                                                    ],
                                                }),
                                            },
                                            Def {
                                                label: "y2".to_string(),
                                                value: Value::Array(vec![Value::Number(42.0), Value::Invalid]),
                                            },
                                            Def {
                                                label: "H0".to_string(),
                                                value: Value::Invalid,
                                            },
                                        ],
                                    },
                                    Region {
                                        name: "main::f".to_string(),
                                        definitions: vec![
                                            Def {
                                                label: "x".to_string(),
                                                value: Value::Number(42.0),
                                            },
                                            Def {
                                                label: "y".to_string(),
                                                value: Value::Number(2.0),
                                            },
                                        ],
                                    },
                                ],
                            },
                            Location {
                                name: "Heap".to_string(),
                                definitions: vec![
                                    Def {
                                        label: "H0".to_string(),
                                        value: Value::Number(42.0),
                                    },
                                    Def {
                                        label: "y".to_string(),
                                        value: Value::Number(2.0),
                                    },
                                ],
                                regions: vec![]
                            },                    
                        ],
                    },
                    Step {
                        label: "L1".to_string(),
                        locations: vec![
                            Location {
                                name: "Stack".to_string(),
                                definitions: vec![],
                                regions: vec![
                                    Region {
                                        name: "main".to_string(),
                                        definitions: vec![
                                            Def {
                                                label: "x".to_string(),
                                                value: Value::Struct(NamedStruct {
                                                    name: "Rect".to_string(),
                                                    fields: vec![
                                                        ("pos".to_string(), Value::Number(42.0)),
                                                        ("w".to_string(), Value::Number(3.0)),
                                                    ],
                                                }),
                                            },
                                            Def {
                                                label: "y2".to_string(),
                                                value: Value::Array(vec![Value::Number(42.0), Value::Invalid]),
                                            },
                                            Def {
                                                label: "H0".to_string(),
                                                value: Value::Invalid,
                                            },
                                        ],
                                    },
                                ],
                            },
                            Location {
                                name: "Heap".to_string(),
                                definitions: vec![
                                ],
                                regions: vec![]
                            },
                        ],
                    },
                ],
            ),
            &mut rs,
            &canvas,
        )?;
        v.translate(point(600.0, 500.0));
        v.draw(&mut canvas)?;

        canvas.save("test_render_value.png")?;

        Ok(())
    }
}
