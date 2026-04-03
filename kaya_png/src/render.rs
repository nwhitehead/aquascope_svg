#![allow(unused)]

use ab_glyph::point;
use anyhow::Result;

use crate::canvas::Canvas;
use crate::draw::{
    Drawable, FormulaType, GArray, GLine, GPadding, GSpace, GTagged, GText, border, compute_align,
    hstack, hstack_top, vstack_left, vstack_none,
};
use crate::draw_state::DrawState;
use crate::style::{Styling, color};

use kaya_lib::states::{Def, Location, NamedStruct, Program, Ptr, Region, Step, Value};

#[derive(Clone, Debug)]
pub struct RenderState {
    pub style: Styling,
    skip_heap: bool,
    ids: Vec<String>,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            style: Default::default(),
            skip_heap: true,
            ids: vec![],
        }
    }
}

impl RenderState {
    pub fn register(&mut self, id: &str) {
        self.ids.push(id.to_string());
    }
    pub fn ids(&self) -> Vec<String> {
        self.ids.clone()
    }
    pub fn clear_ids(&mut self) {
        self.ids = vec![];
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
    a: &[Value],
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
    a: &[Value],
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
    let mut ds = DrawState {
        font: style.get_string_or("def.label.font", "mono"),
        font_size: style.get_number_or("def.label.font_size", 24.0),
        text_color: style.get_color_or("def.label.color", color("#000")?),
        ..DrawState::default()
    };

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
        ds.font = style.get_string_or("value.struct.label.font", "mono");
        ds.font_size = style.get_number_or("value.struct.label.font_size", 24.0);
        ds.text_color = style.get_color_or("value.struct.label.color", color("#000")?);
        let g_label = GText::new(label, point(0.0, 0.0), ds.clone());

        ds.font = style.get_string_or("value.struct.separator.font", "mono");
        ds.font_size = style.get_number_or("value.struct.separator.font_size", 24.0);
        ds.text_color = style.get_color_or("value.struct.separator.color", color("#000")?);
        let sep_text = style.get_string_or("value.struct.separator.text", ":");
        let g_separator = GText::new(&sep_text, point(0.0, 0.0), ds.clone());

        let sep_padding = style.get_padding("value.struct.separator.padding", 0.0);
        let g_padded_sep = GPadding::new(Box::new(g_separator), sep_padding);

        let left: Vec<Box<dyn Drawable>> = vec![Box::new(g_label), Box::new(g_padded_sep)];
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
    render_state: &mut RenderState,
    _canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    let style = &render_state.style;
    let ds = DrawState {
        font: style.get_string_or("value.invalid.font", "mono"),
        text_color: style.get_color_or("value.invalid.color", color("#000")?),
        font_size: style.get_number_or("value.invalid.font_size", 24.0),
        ..DrawState::default()
    };
    let padding = style.get_padding("value.invalid.padding", 5.0);
    let text = "×".to_string();
    let gtxt = GText::new(&text, point(0.0, 0.0), ds);
    let padded_gtxt = GPadding::new(Box::new(gtxt), padding);
    Ok(Box::new(padded_gtxt))
}

fn render_value_number(
    v: f64,
    render_state: &mut RenderState,
    _canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    let style = &render_state.style;
    let ds = DrawState {
        font: style.get_string_or("value.number.font", "mono"),
        text_color: style.get_color_or("value.number.color", color("#000")?),
        font_size: style.get_number_or("value.number.font_size", 24.0),
        ..DrawState::default()
    };
    let padding = style.get_padding("value.number.padding", 5.0);
    let text = format!("{}", v);
    let gtxt = GText::new(&text, point(0.0, 0.0), ds);
    let padded_gtxt = GPadding::new(Box::new(gtxt), padding);
    Ok(Box::new(padded_gtxt))
}

fn render_value_char(
    c: char,
    render_state: &mut RenderState,
    _canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    let style = &render_state.style;
    let ds = DrawState {
        font: style.get_string_or("value.char.font", "mono"),
        text_color: style.get_color_or("value.char.color", color("#000")?),
        font_size: style.get_number_or("value.char.font_size", 24.0),
        ..DrawState::default()
    };
    let text = format!("'{}'", c);
    Ok(Box::new(GText::new(&text, point(0.0, 0.0), ds)))
}

fn render_value_pointer(
    _p: &Ptr,
    render_state: &mut RenderState,
    _canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    let style = &render_state.style;
    let ds = DrawState {
        font: style.get_string_or("value.pointer.font", "mono"),
        text_color: style.get_color_or("value.pointer.color", color("#000")?),
        font_size: style.get_number_or("value.pointer.font_size", 24.0),
        ..DrawState::default()
    };
    let padding = style.get_padding("value.pointer.padding", 0.0);
    // ✕✖✗✘×•●○◯42
    let text = "●";
    let g_txt = GText::new(text, point(0.0, 0.0), ds);
    let g_padded = GPadding::new(Box::new(g_txt), padding);
    Ok(Box::new(g_padded))
}

pub fn render_value(
    value: &Value,
    prefix: &str,
    render_state: &mut RenderState,
    canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    let item = match value {
        Value::Number(v) => render_value_number(*v, render_state, canvas)?,
        Value::Char(c) => render_value_char(*c, render_state, canvas)?,
        Value::Pointer(p) => render_value_pointer(p, render_state, canvas)?,
        Value::Array(a) => render_value_array(a, prefix, render_state, canvas)?,
        Value::Tuple(a) => render_value_tuple(a, prefix, render_state, canvas)?,
        Value::Struct(a) => render_value_struct(a, prefix, render_state, canvas)?,
        Value::Invalid => render_value_invalid(render_state, canvas)?,
    };
    let tagged = GTagged::new(item, prefix);
    render_state.register(prefix);
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
    let ds = DrawState {
        font: style.get_string_or("region.header.font", "serif"),
        text_color: style.get_color_or("region.header.color", color("#000")?),
        font_size: style.get_number_or("region.header.font_size", 24.0),
        ..DrawState::default()
    };
    let header_padding = style.get_padding("region.header.padding", 5.0);
    let region_padding = style.get_padding("region.padding", 5.0);
    let gtxt = GText::new(&value.name.to_string(), point(0.0, 0.0), ds);
    let padded_gtxt = GPadding::new(Box::new(gtxt), header_padding);
    // Body
    let mut body: Vec<Box<dyn Drawable>> = vec![];
    for def in &value.definitions {
        let g_def = render_def(def, prefix, render_state, canvas, skip_heap)?;
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
    let ds = DrawState {
        font: style.get_string_or("location.header.font", "serif"),
        text_color: style.get_color_or("location.header.color", color("#000")?),
        font_size: style.get_number_or("location.header.font_size", 24.0),
        ..DrawState::default()
    };
    let header_padding = style.get_padding("location.header.padding", 5.0);
    let gtxt = GText::new(&value.name.to_string(), point(0.0, 0.0), ds);
    let padded_gtxt = GPadding::new(Box::new(gtxt), header_padding);

    // Body
    let mut body: Vec<Box<dyn Drawable>> = vec![];
    for region in &value.regions {
        let g_region = render_region(region, prefix, render_state, canvas, skip_heap)?;
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
    let mut ds = DrawState {
        font: style.get_string_or("step.header.font", "serif"),
        text_color: style.get_color_or("step.header.color", color("#000")?),
        font_size: style.get_number_or("step.header.font_size", 24.0),
        stroke_color: style.get_color_or("step.separator.color", color("#000")?),
        ..DrawState::default()
    };
    let padding = style.get_padding("step.header.padding", 5.0);
    let g_text = GText::new(&value.label.to_string(), point(0.0, 0.0), ds.clone());
    let g_padded_text = GPadding::new(Box::new(g_text), padding);

    // draw border on right side sized properly
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
        let g_location = render_location(location, &value.label.to_string(), render_state, canvas)?;
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
        let g_location = render_step(step, render_state, canvas)?;
        if idx > 0 {
            body.push(Box::new(GSpace::new(0.0, gap)));
        }
        body.push(g_location);
    }
    let g_body = vstack_left(body, canvas)?;
    Ok(Box::new(g_body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::standard_style;
    use tiny_skia::{Color, ColorU8};

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
        rs.style = standard_style()?;

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

        rs.clear_ids();
        let mut v = render_program(
            &Program(vec![
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
                                            value: Value::Array(vec![
                                                Value::Number(42.0),
                                                Value::Invalid,
                                            ]),
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
                            regions: vec![],
                        },
                    ],
                },
                Step {
                    label: "L1".to_string(),
                    locations: vec![
                        Location {
                            name: "Stack".to_string(),
                            definitions: vec![],
                            regions: vec![Region {
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
                                        value: Value::Array(vec![
                                            Value::Number(42.0),
                                            Value::Invalid,
                                        ]),
                                    },
                                    Def {
                                        label: "H0".to_string(),
                                        value: Value::Invalid,
                                    },
                                ],
                            }],
                        },
                        Location {
                            name: "Heap".to_string(),
                            definitions: vec![],
                            regions: vec![],
                        },
                    ],
                },
            ]),
            &mut rs,
            &canvas,
        )?;
        v.translate(point(600.0, 500.0));
        // Show some rects
        println!("IDS: {:?}", rs.ids());
        for id in rs.ids() {
            let r = v.get_tagged(&id).unwrap().bounding_box(&canvas)?;
            println!("{} => {:?}", id, r);
        }
        v.draw(&mut canvas)?;

        canvas.save("test_render_value.png")?;

        Ok(())
    }
}
