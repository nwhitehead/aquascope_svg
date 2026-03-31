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

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut v = render_value(&Value::Number(42.0), &mut rs);
        v.translate(point(400.0, 400.0));
        v.draw(&mut canvas)?;
        canvas.save("test_render_value.png")?;

        Ok(())
    }
}
