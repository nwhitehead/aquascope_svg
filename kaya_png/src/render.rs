use ab_glyph::{Point, Rect, point};
use anyhow::{Result, bail};
use std::collections::HashMap;
use tiny_skia::{Color, ColorU8};

use crate::canvas::Canvas;
use crate::draw::{Drawable, FormulaType, GArray, GLine, GPadding, GSpace, GText, border, compute_align, hstack};
use crate::draw_state::DrawState;
use crate::style::Styling;

use kaya_lib::states::{Def, Location, Program, Ptr, Region, Step, Value};

#[derive(Clone, Debug, Default)]
pub struct RenderState {
    locations: HashMap<String, Rect>,
    style: Styling,
}

fn max_height(values: &Vec<Box<dyn Drawable>>, canvas: &Canvas) -> Result<f32> {
    let mut res: f32 = 0.0;
    for value in values {
        let bb = value.bounding_box(&canvas)?;
        let h = bb.max.y - bb.min.y;
        res = res.max(h);
    }
    Ok(res)
}

fn render_value_array(
    a: &Vec<Value>,
    render_state: &mut RenderState,
    canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    // Draw all the parts separately
    let mut a_draws: Vec<Box<dyn Drawable>> = vec![];
    for x in a {
        let draw = render_value(&x, render_state, canvas)?;
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
    let h = max_height(&a_draws, &canvas)?;
    let sep_margin = style.get_number_or("value.array.separator.vmargin", 5.0);
    // intersperse vertical lines
    ds.stroke_color = style.get_color_or("value.array.separator.color", color("#000")?);
    let sep = GLine::new(point(0.0, 0.0), point(0.0, h - sep_margin), ds.clone());
    let sep_padding = (
        style.get_number_or("value.array.separator.padding.left", 5.0),
        style.get_number_or("value.array.separator.padding.top", 5.0),
        style.get_number_or("value.array.separator.padding.right", 5.0),
        style.get_number_or("value.array.separator.padding.bottom", 5.0),
    );
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
    ds.padding.0 = style.get_number_or("value.array.padding.left", 5.0);
    ds.padding.1 = style.get_number_or("value.array.padding.top", 5.0);
    ds.padding.2 = style.get_number_or("value.array.padding.right", 5.0);
    ds.padding.3 = style.get_number_or("value.array.padding.bottom", 5.0);
    ds.stroke_color = style.get_color_or("value.array.border.color", color("#000")?);
    ds.stroke.width = style.get_number_or("value.array.border.width", 4.0);
    let radius = style.get_number_or("value.array.border.radius", 5.0);
    let radius_nw = style.get_number_or("value.array.border.radius.nw", radius);
    let radius_ne = style.get_number_or("value.array.border.radius.ne", radius);
    let radius_sw = style.get_number_or("value.array.border.radius.sw", radius);
    let radius_se = style.get_number_or("value.array.border.radius.se", radius);
    ds.border_radius = (radius_nw, radius_ne, radius_se, radius_sw);
    let res = border(Box::new(stk), &canvas, ds)?;
    return Ok(res);
}

fn render_value_tuple(
    a: &Vec<Value>,
    render_state: &mut RenderState,
    canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    // Draw all the parts separately
    let mut a_draws: Vec<Box<dyn Drawable>> = vec![];
    for x in a {
        let draw = render_value(&x, render_state, canvas)?;
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
    let h = max_height(&a_draws, &canvas)?;
    let sep_margin = style.get_number_or("value.tuple.separator.vmargin", 5.0);
    // intersperse vertical lines
    ds.stroke_color = style.get_color_or("value.tuple.separator.color", color("#000")?);
    let sep = GLine::new(point(0.0, 0.0), point(0.0, h - sep_margin), ds.clone());
    let sep_padding = (
        style.get_number_or("value.tuple.separator.padding.left", 5.0),
        style.get_number_or("value.tuple.separator.padding.top", 5.0),
        style.get_number_or("value.tuple.separator.padding.right", 5.0),
        style.get_number_or("value.tuple.separator.padding.bottom", 5.0),
    );
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
    ds.padding.0 = style.get_number_or("value.tuple.padding.left", 5.0);
    ds.padding.1 = style.get_number_or("value.tuple.padding.top", 5.0);
    ds.padding.2 = style.get_number_or("value.tuple.padding.right", 5.0);
    ds.padding.3 = style.get_number_or("value.tuple.padding.bottom", 5.0);
    ds.stroke_color = style.get_color_or("value.tuple.border.color", color("#000")?);
    ds.stroke.width = style.get_number_or("value.tuple.border.width", 4.0);
    let radius = style.get_number_or("value.tuple.border.radius", 5.0);
    let radius_nw = style.get_number_or("value.tuple.border.radius.nw", radius);
    let radius_ne = style.get_number_or("value.tuple.border.radius.ne", radius);
    let radius_sw = style.get_number_or("value.tuple.border.radius.sw", radius);
    let radius_se = style.get_number_or("value.tuple.border.radius.se", radius);
    ds.border_radius = (radius_nw, radius_ne, radius_se, radius_sw);
    let res = border(Box::new(stk), &canvas, ds)?;
    return Ok(res);
}

fn render_def(
    def: &Def,
    render_state: &mut RenderState,
    canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    let style = &render_state.style;
    let mut ds = DrawState::default();
    ds.font = style.get_string_or("def.label.font", "mono");
    ds.font_size = style.get_number_or("def.label.font_size", 24.0);
    ds.text_color = style.get_color_or("def.label.color", color("#000")?);

    let pad = style.get_number_or("def.padding", 5.0);
    let left = style.get_number_or("def.padding.left", pad);
    let top = style.get_number_or("def.padding.top", pad);
    let right = style.get_number_or("def.padding.right", pad);
    let bottom = style.get_number_or("def.padding.bottom", pad);
    let padding = (left, top, right, bottom);

    let mut g_label = GText::new(&def.label, point(0.0, 0.0), ds.clone());

    ds.font = style.get_string_or("def.separator.font", "mono");
    ds.font_size = style.get_number_or("def.separator.font_size", 24.0);
    ds.text_color = style.get_color_or("def.separator.color", color("#000")?);
    let sep_text = style.get_string_or("def.separator.text", ":");
    let g_separator = GText::new(&sep_text, point(0.0, 0.0), ds.clone());

    let sep_pad = style.get_number_or("def.separator.padding", 0.0);
    let sep_padding = (
        style.get_number_or("def.separator.padding.left", sep_pad),
        style.get_number_or("def.separator.padding.top", sep_pad),
        style.get_number_or("def.separator.padding.right", sep_pad),
        style.get_number_or("def.separator.padding.bottom", sep_pad),
    );
    let g_padded_sep = GPadding::new(Box::new(g_separator), sep_padding);

    // Make sure final drawable has x=0 as the dividing line for separator
    // (so we can align them later)
    // Instead of just doing hstack_... we compute the translation and use parts of it
    // Move label left to align right side to separator
    let label_bb = g_label.bounding_box(canvas)?;
    let sep_bb = g_padded_sep.bounding_box(canvas)?;
    let p = compute_align(&label_bb, &sep_bb, FormulaType::Sequenced, FormulaType::Centered);
    g_label.translate(point(-p.x, -p.y));
    let mut left = GArray::new();
    left.push(Box::new(g_label));
    left.push(Box::new(g_padded_sep));

    let left_pad = style.get_number_or("def.left.padding", 0.0);
    let left_padding = (
        style.get_number_or("def.left.padding.left", left_pad),
        style.get_number_or("def.left.padding.top", left_pad),
        style.get_number_or("def.left.padding.right", left_pad),
        style.get_number_or("def.left.padding.bottom", left_pad),
    );
    let g_left = GPadding::new(Box::new(left), left_padding);
    let left_bb = g_left.bounding_box(canvas)?;

    let mut g_value = render_value(&def.value, render_state, canvas)?;

    // Now align the value to right of separator, centered vertically
    let value_bb = g_value.bounding_box(canvas)?;
    println!("left_bb = {:?}", &left_bb);
    println!("value_bb = {:?}", &value_bb);
    let p = compute_align(&left_bb, &value_bb, FormulaType::Sequenced, FormulaType::Centered);
    g_value.translate(p);

    let mut g_array = GArray::new();
    g_array.push(Box::new(g_left));
    g_array.push(g_value);
    //let padded_gtxt = GPadding::new(Box::new(gtxt), padding);
    Ok(Box::new(g_array))
}

// fn render_value_struct(
//     named_struct: &NamedStruct,
//     render_state: &mut RenderState,
//     canvas: &Canvas,
// ) -> Result<Box<dyn Drawable>> {
//     // Draw all the parts separately
//     let mut field_draws: Vec<Box<dyn Drawable>> = vec![];
//     for x in named_struct.fields {
//         let draw = render_value(&x, render_state, canvas)?;
//         a_draws.push(draw);
//     }
//     let style = &render_state.style;
//     if a_draws.is_empty() {
//         a_draws.push(
//             GSpace::new(
//                 style.get_number_or("value.tuple.empty.w", 5.0),
//                 style.get_number_or("value.tuple.empty.h", 5.0),
//             )
//             .clone_box(),
//         );
//     }
//     let mut ds = DrawState::default();
//     // Now measure the height for divider lines
//     let h = max_height(&a_draws, &canvas)?;
//     let sep_margin = style.get_number_or("value.tuple.separator.vmargin", 5.0);
//     // intersperse vertical lines
//     ds.stroke_color = style.get_color_or("value.tuple.separator.color", color("#000")?);
//     let sep = GLine::new(point(0.0, 0.0), point(0.0, h - sep_margin), ds.clone());
//     let sep_padding = (
//         style.get_number_or("value.tuple.separator.padding.left", 5.0),
//         style.get_number_or("value.tuple.separator.padding.top", 5.0),
//         style.get_number_or("value.tuple.separator.padding.right", 5.0),
//         style.get_number_or("value.tuple.separator.padding.bottom", 5.0),
//     );
//     let padded_sep = GPadding::new(Box::new(sep), sep_padding);
//     let mut a_draws_sep: Vec<Box<dyn Drawable>> = vec![];
//     let mut any_elems_yet = false;
//     for x in a_draws {
//         if any_elems_yet {
//             a_draws_sep.push(padded_sep.clone_box());
//         } else {
//             any_elems_yet = true;
//         }
//         a_draws_sep.push(x);
//     }
//     let stk = hstack(a_draws_sep, canvas)?;
//     ds.padding.0 = style.get_number_or("value.tuple.padding.left", 5.0);
//     ds.padding.1 = style.get_number_or("value.tuple.padding.top", 5.0);
//     ds.padding.2 = style.get_number_or("value.tuple.padding.right", 5.0);
//     ds.padding.3 = style.get_number_or("value.tuple.padding.bottom", 5.0);
//     ds.stroke_color = style.get_color_or("value.tuple.border.color", color("#000")?);
//     ds.stroke.width = style.get_number_or("value.tuple.border.width", 4.0);
//     let radius = style.get_number_or("value.tuple.border.radius", 5.0);
//     let radius_nw = style.get_number_or("value.tuple.border.radius.nw", radius);
//     let radius_ne = style.get_number_or("value.tuple.border.radius.ne", radius);
//     let radius_sw = style.get_number_or("value.tuple.border.radius.sw", radius);
//     let radius_se = style.get_number_or("value.tuple.border.radius.se", radius);
//     ds.border_radius = (radius_nw, radius_ne, radius_se, radius_sw);
//     let res = border(Box::new(stk), &canvas, ds)?;
//     return Ok(res);
// }

fn render_value_number(
    v: f64,
    render_state: &mut RenderState,
    canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    let style = &render_state.style;
    let mut ds = DrawState::default();
    ds.font = style.get_string_or("value.number.font", "mono");
    ds.text_color = style.get_color_or("value.number.color", color("#000")?);
    ds.font_size = style.get_number_or("value.number.font_size", 24.0);
    let left = style.get_number_or("value.number.padding.left", 5.0);
    let top = style.get_number_or("value.number.padding.top", 5.0);
    let right = style.get_number_or("value.number.padding.right", 5.0);
    let bottom = style.get_number_or("value.number.padding.bottom", 5.0);
    let padding = (left, top, right, bottom);
    let text = format!("{}", v);
    let gtxt = GText::new(&text, point(0.0, 0.0), ds);
    let padded_gtxt = GPadding::new(Box::new(gtxt), padding);
    Ok(Box::new(padded_gtxt))
}

fn render_value_char(
    c: char,
    render_state: &mut RenderState,
    canvas: &Canvas,
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
    p: &Ptr,
    render_state: &mut RenderState,
    canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    let style = &render_state.style;
    let mut ds = DrawState::default();
    ds.font = style.get_string_or("value.pointer.font", "mono");
    ds.text_color = style.get_color_or("value.pointer.color", color("#000")?);
    ds.font_size = style.get_number_or("value.pointer.font_size", 24.0);
    // ✕✖✗✘×•●○◯42
    let text = "●";
    Ok(Box::new(GText::new(&text, point(0.0, 0.0), ds)))
}

pub fn render_value(
    value: &Value,
    render_state: &mut RenderState,
    canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    match value {
        Value::Number(v) => Ok(render_value_number(*v, render_state, &canvas)?),
        Value::Char(c) => Ok(render_value_char(*c, render_state, &canvas)?),
        Value::Pointer(p) => Ok(render_value_pointer(p, render_state, &canvas)?),
        Value::Array(a) => Ok(render_value_array(&a, render_state, &canvas)?),
        Value::Tuple(a) => Ok(render_value_tuple(&a, render_state, &canvas)?),
        _ => panic!("not handled"),
    }
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

        let mut v = render_value(&Value::Number(42.0), &mut rs, &canvas)?;
        v.translate(point(400.0, 400.0));
        v.draw(&mut canvas)?;
        v.translate(point(10.0, 5.0));
        v.draw(&mut canvas)?;
        v.translate(point(10.0, 5.0));
        v.draw(&mut canvas)?;

        rs.style
            .add_color("value.number.color", color("#cfa9bc80")?);
        let mut v2 = render_value(&Value::Number(67.0), &mut rs, &canvas)?;
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
        let mut canvas = Canvas::new(800, 800)?;
        canvas
            .pixmap
            .fill(Color::from_rgba8(0x19, 0x19, 0x19, 0xff));
        canvas.load_font(
            "mono",
            include_bytes!("../fonts/DejaVu/DejaVuSansMono-Bold.ttf"),
        )?;
        canvas.load_font("serif", include_bytes!("../fonts/Lato/Lato-Regular.ttf"))?;

        let mut rs = RenderState::default();
        rs.style.add_string("value.number.font", "mono");
        rs.style.add_number("value.number.font_size", 23.0);
        rs.style.add_color("value.number.color", color("#bccfa9")?);
        rs.style.add_number("value.number.padding.left", 5.0);
        rs.style.add_number("value.number.padding.top", 5.0);
        rs.style.add_number("value.number.padding.right", 5.0);
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
        rs.style.add_number("def.separator.padding.right", 3.0);
        rs.style.add_number("def.left.padding.bottom", 3.0);


        let mut v = render_value(&Value::Number(42.0), &mut rs, &canvas)?;
        v.translate(point(200.0, 200.0));
        v.draw(&mut canvas)?;

        let mut v = render_value(&Value::Char('H'), &mut rs, &canvas)?;
        v.translate(point(250.0, 200.0));
        v.draw(&mut canvas)?;

        let mut v = render_value(
            &Value::Pointer(Ptr {
                name: "".to_string(),
                selectors: vec![],
                borrow: 0,
                help: vec![],
            }),
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
            &mut rs,
            &canvas,
        )?;
        v.translate(point(350.0, 200.0));
        v.draw(&mut canvas)?;

        let mut v = render_def(
            &Def{
                label: "a".to_string(),
                value: Value::Number(7.0),
            },
            &mut rs,
            &canvas,
        )?;
        v.translate(point(200.0, 300.0));
        v.draw(&mut canvas)?;

        canvas.save("test_render_value.png")?;

        Ok(())
    }
}
