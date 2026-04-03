
use ab_glyph::point;

use crate::canvas::Canvas;

pub struct Arrow {
    start: Point,
    end: Point,
}


impl Arrow {
    pub fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }
    pub fn draw(&self, canvas: &mut Canvas) {

    }
}