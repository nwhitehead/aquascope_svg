#![allow(unused)]

use anyhow::{Result, bail};
use ab_glyph::{Point, Rect, point};
use tiny_skia::{Paint, PathBuilder, Transform};

use crate::canvas::Canvas;
use crate::draw::Drawable;
use crate::draw_state::DrawState;
use crate::style::color;

#[derive(Clone, Debug)]
pub struct Arrow {
    start: Point,
    start_control: Point,
    end: Point,
    end_control: Point,
    state: DrawState,
}

fn norm(p: Point) -> f32 {
    (p.x * p.x + p.y * p.y).sqrt()
}

fn scale(p: Point, s: f32) -> Point {
    point(p.x * s, p.y * s)
}

/// Decompose a direction vector into two vectors: along and parallel (unit length)
fn decomp(dir: Point) -> (Point, Point) {
    let norm = 1.0 / norm(dir);
    let parallel = point(dir.x * norm, dir.y * norm);
    let perp = point(parallel.y, -parallel.x);
    (parallel, perp)
}

impl Arrow {
    pub fn new(start: Point, start_control: Point, end: Point, end_control: Point, state: DrawState) -> Self {
        Self { start, start_control, end, end_control, state }
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
        let color = self.state.stroke_color;
        let mut paint = Paint::default();
        paint.set_color_rgba8(color.red(), color.green(), color.blue(), color.alpha());
        paint.anti_alias = true;
        let forward = self.end - self.end_control;
        let (par, perp) = decomp(forward);
        let head_length = 20.0;
        let arrow_width = 40.0;
        let arrow_dent_ratio = 0.1;
        let crossp = self.end - scale(par, head_length);
        // p1 and p2 are tips on sides
        let p1 = self.end + scale(par, -head_length) + scale(perp, arrow_width);
        let p2 = self.end + scale(par, -head_length) + scale(perp, -arrow_width);
        // p3 is middle dent part
        let p3 = self.end + scale(par, -head_length * ( 1.0 - arrow_dent_ratio));
        let Some(path) = ({
            let mut pb = PathBuilder::new();
            pb.move_to(self.start.x, self.start.y);
            pb.cubic_to(self.start_control.x, self.start_control.y, self.end_control.x, self.end_control.y, self.end.x, self.end.y);
            //pb.line_to(self.end.x, self.end.y);
            pb.finish()
        }) else {
            bail!("could not make path");
        };
        canvas.pixmap.stroke_path(
            &path,
            &paint,
            &self.state.stroke,
            Transform::identity(),
            None,
        );
        let mut stroke2 = self.state.stroke.clone();
        stroke2.width = 2.0;
        paint.set_color_rgba8(0, 0, 0, 255);
        let Some(path) = ({
            let mut pb = PathBuilder::new();
            pb.move_to(p1.x, p1.y);
            pb.line_to(self.end.x, self.end.y);
            pb.line_to(p2.x, p2.y);
            pb.line_to(p3.x, p3.y);
            pb.close();
            pb.finish()
        }) else {
            bail!("could not make path2");
        };
        canvas.pixmap.stroke_path(
            &path,
            &paint,
            &stroke2,
            Transform::identity(),
            None,
        );


        Ok(())
    }
    fn clone_box(&self) -> Box<dyn Drawable> {
        Box::new(self.clone())
    }
    fn get_tagged(&self, _id: &str) -> Option<Box<dyn Drawable>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::standard_style;
    use crate::draw::GBox;
    use tiny_skia::{Color, ColorU8};

    #[test]
    pub fn test_draw_arrow() -> Result<()> {
        let mut canvas = Canvas::new(800, 800)?;
        canvas
            .pixmap
            .fill(Color::from_rgba(0.2, 0.1, 0.3, 1.0).unwrap());

        canvas.load_font(
            "mono",
            include_bytes!("../fonts/DejaVu/DejaVuSansMono-Bold.ttf"),
        )?;
        canvas.load_font("serif", include_bytes!("../fonts/Lato/Lato-Regular.ttf"))?;

        let mut ds = DrawState::default();
        ds.stroke.width = 20.0;
        ds.stroke_color = color("#ff0")?;
    
        let arrow = Arrow::new(
            point(100.0, 100.0),
            point(175.0, 100.0),
            point(300.0, 200.0),
            point(250.0, 200.0),
            ds,
        );
        arrow.draw(&mut canvas)?;

        canvas.save("test_render_arrow.png")?;
        Ok(())
    }
}
