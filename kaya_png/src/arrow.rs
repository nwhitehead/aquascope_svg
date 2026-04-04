#![allow(unused)]

use anyhow::Result;
use ab_glyph::{Point, Rect, point};

use crate::canvas::Canvas;
use crate::draw::Drawable;
use crate::draw_state::DrawState;

#[derive(Clone, Debug)]
pub struct Arrow {
    start: Point,
    end: Point,
    state: DrawState,
}

impl Arrow {
    pub fn new(start: Point, end: Point, state: DrawState) -> Self {
        Self { start, end, state }
    }
}

impl Drawable for Arrow {
    fn translate(&mut self, t: Point) {
        self.start += t;
        self.end += t;
    }
    fn bounding_box(&self, _canvas: &Canvas) -> Result<Rect> {
        Ok(Rect {
            min: point(self.start.x.min(self.end.x), self.start.y.min(self.end.y)),
            max: point(self.start.x.max(self.end.x), self.start.y.max(self.end.y)),
        })
    }
    fn draw(&self, canvas: &mut Canvas) -> Result<()> {
        Ok(())
    }
    fn clone_box(&self) -> Box<dyn Drawable> {
        Box::new(self.clone())
    }
    fn get_tagged(&self, _id: &str) -> Option<Box<dyn Drawable>> {
        None
    }
}
