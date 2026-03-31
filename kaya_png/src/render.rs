use ab_glyph::{Point, Rect, point};
use anyhow::{Result, bail};
use std::collections::HashMap;
use tiny_skia::ColorU8;

use crate::canvas::Canvas;
use crate::draw::{Drawable, GText};
use crate::draw_state::DrawState;
use crate::style::Styling;

use kaya_lib::states::{Def, Location, Program, Region, Step, Value};

#[derive(Clone, Debug, Default)]
pub struct RenderState {
    locations: HashMap<String, Rect>,
    style: Styling,
}

pub fn render_value(value: &Value, render_state: &mut RenderState) -> Box<dyn Drawable> {
    let style = &render_state.style;
    let mut ds = DrawState::default();
    let mono = "mono";
    let black = ColorU8::from_rgba(0, 0, 0, 255);
    match value {
        Value::Number(v) => {
            ds.font = style.get_string_or("value.number.font", mono);
            ds.text_color = style.get_color_or("value.number.color", black);
            ds.font_size = style.get_number_or("value.number.font_size", 24.0);
            let text = format!("{}", v);
            return Box::new(GText::new(&text, point(0.0, 0.0), ds));
        },
        _ => panic!("not handled"),
    }

    // Ok(format!(
    //     "<span id=\"{}\" class=\"value number\">{}</span>",
    //     &state.id_prefix, v
    // )),

    // render_state.locations.insert(
    //     "d2".to_string(),
    //     Rect {
    //         min: point(0.0, 0.0),
    //         max: point(0.0, 0.0),
    //     },
    // );
    // let mut s = DrawState {
    //     ..Default::default()
    // };
    // s.font = "mono".to_string();
    // s.stroke_color = ColorU8::from_rgba(128, 0, 128, 255);
    // s.stroke.width = 2.0;
    // s.border_radius = (5.0, 5.0, 5.0, 5.0);
    // s.border_clip = (false, false, false, false);
    // s.padding = (60.0, 30.0, 60.0, 30.0);
    // s.margin = (40.0, 10.0, 40.0, 10.0);
    panic!("unreachable");
}

fn color(txt: &str) -> Result<ColorU8> {
    let mut r = 255;
    let mut g = 255;
    let mut b = 255;
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
        },
        4 => {
            r = u8::from_str_radix(&txt[0..1], 16)? * (0x10 + 0x01);
            g = u8::from_str_radix(&txt[1..2], 16)? * (0x10 + 0x01);
            b = u8::from_str_radix(&txt[2..3], 16)? * (0x10 + 0x01);
            a = u8::from_str_radix(&txt[3..4], 16)? * (0x10 + 0x01);
        },
        6 => {
            r = u8::from_str_radix(&txt[0..2], 16)?;
            g = u8::from_str_radix(&txt[2..4], 16)?;
            b = u8::from_str_radix(&txt[4..6], 16)?;
        },
        8 => {
            r = u8::from_str_radix(&txt[0..2], 16)?;
            g = u8::from_str_radix(&txt[2..4], 16)?;
            b = u8::from_str_radix(&txt[4..6], 16)?;
            a = u8::from_str_radix(&txt[6..8], 16)?;
        },
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
        assert_eq!(color("#123456")?, ColorU8::from_rgba(0x12, 0x34, 0x56, 0xff));
        assert_eq!(color("#12345678")?, ColorU8::from_rgba(0x12, 0x34, 0x56, 0x78));
        assert!(color("000").is_err());
        assert!(color("#12345").is_err());
        assert!(color("#0g3456").is_err());
        assert!(color("#fffffffff").is_err());
        Ok(())
    }

    #[test]
    pub fn test_render_value() -> Result<()> {
        let mut canvas = Canvas::new(800, 800)?;

        canvas.load_font(
            "mono",
            include_bytes!("../fonts/DejaVu/DejaVuSansMono-Bold.ttf"),
        )?;
        canvas.load_font("serif", include_bytes!("../fonts/Lato/Lato-Regular.ttf"))?;

        let mut rs = RenderState::default();
        rs.style.add_string("value.number.font", "mono");
        rs.style.add_number("value.number.font_size", 48.0);
        rs.style.add_color("value.number.color", color("#f00")?);
        let mut v = render_value(&Value::Number(42.0), &mut rs);
        v.translate(point(400.0, 400.0));
        v.draw(&mut canvas)?;
        canvas.save("test_render_value.png")?;

        Ok(())
    }
}
