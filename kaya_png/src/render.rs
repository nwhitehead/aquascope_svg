use anyhow::{Result, bail};
use std::collections::HashMap;
use tiny_skia::{Point, Rect};

use crate::canvas::Canvas;
use crate::draw::GText;
use crate::draw_state::DrawState;
use crate::style::Styling;

use kaya_lib::states::{Def, Location, Program, Region, Step, Value};

#[derive(Clone, Debug)]
pub struct RenderState {
    locations: HashMap<String, Rect>,
}

pub fn render_value() {
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
}
