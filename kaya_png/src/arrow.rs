#![allow(unused)]

use ab_glyph::{Point, Rect, point};
use anyhow::{Result, bail};
use tiny_skia::{ColorU8, FillRule, Paint, PathBuilder, Stroke, Transform};

use crate::canvas::Canvas;
use crate::draw::Drawable;
use crate::draw_state::DrawState;
use crate::style::color;

#[derive(Clone, Debug)]
pub struct FluidOptions {
    start_gravity: f32,
    end_gravity: f32,
}

#[derive(Clone, Debug)]
pub struct ArrowOutline {
    width: f32,
    color: ColorU8,
}

#[derive(Clone, Debug)]
pub enum ArrowType {
    Straight,
    Fluid(FluidOptions),
}

#[derive(Clone, Debug)]
pub struct ArrowOptions {
    width: f32,
    color: ColorU8,
    outline: Option<ArrowOutline>,
    head_width: f32,
    head_length: f32,
    dent_ratio: f32,
}

impl Default for ArrowOptions {
    fn default() -> Self {
        Self {
            width: 4.0,
            color: ColorU8::from_rgba(0, 0, 0, 255),
            outline: None,
            head_width: 10.0,
            head_length: 10.0,
            dent_ratio: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Arrow {
    start: Point,
    end: Point,
    start_dir: Point,
    end_dir: Point,
    arrow_type: ArrowType,
    options: ArrowOptions,
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
    pub fn new(
        start: Point,
        end: Point,
        start_dir: Point,
        end_dir: Point,
        arrow_type: ArrowType,
        options: ArrowOptions,
    ) -> Self {
        Self {
            start,
            end,
            start_dir,
            end_dir,
            arrow_type,
            options,
        }
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
        let ArrowType::Fluid(ref fluid_options) = self.arrow_type else {
            bail!("only fluid supported right now");
        };
        let color = self.options.color;
        let mut paint = Paint::default();
        paint.set_color_rgba8(color.red(), color.green(), color.blue(), color.alpha());
        paint.anti_alias = true;
        let (par, perp) = decomp(self.end_dir);
        let (par_src, perp_src) = decomp(self.start_dir);
        let head_length = self.options.head_length;
        let head_width = self.options.head_width;
        let arrow_dent_ratio = self.options.dent_ratio;
        let end_control = scale(par, -head_length - fluid_options.end_gravity);
        let start_control = scale(par_src, fluid_options.start_gravity);
        // p0 is where thick line ends (before actual tip)
        let p0 = self.end + scale(par, -head_length);
        // p1 and p2 are tips on sides
        let head_offset = scale(par, -head_length * arrow_dent_ratio);
        let p1 = p0 + head_offset + scale(perp, head_width);
        let p2 = p0 + head_offset + scale(perp, -head_width);
        // p0t and p0b are widened points where line actually ends
        let p0t = p0 + scale(perp, self.options.width * 0.5);
        let p0b = p0 + scale(perp, -self.options.width * 0.5);
        // pst and psb are widened points where line actually starts
        let pst = self.start + scale(perp_src, self.options.width * 0.5);
        let psb = self.start + scale(perp_src, -self.options.width * 0.5);
        let Some(path) = ({
            let mut pb = PathBuilder::new();
            pb.move_to(pst.x, pst.y);
            pb.cubic_to(
                pst.x + start_control.x,
                pst.y + start_control.y,
                p0t.x + end_control.x,
                p0t.y + end_control.y,
                p0t.x,
                p0t.y,
            );
            pb.line_to(p1.x, p1.y);
            pb.line_to(self.end.x, self.end.y);
            pb.line_to(p2.x, p2.y);
            pb.line_to(p0b.x, p0b.y);
            pb.cubic_to(
                p0b.x + end_control.x,
                p0b.y + end_control.y,
                psb.x + start_control.x,
                psb.y + start_control.y,
                psb.x,
                psb.y,
            );
            pb.close();
            pb.finish()
        }) else {
            bail!("could not make path");
        };
        // Draw shadow from path
        if let Some(ref arrow_outline) = self.options.outline {
            let stroke = Stroke {
                width: self.options.width,
                ..Default::default()
            };
        }

        canvas
            .pixmap
            .fill_path(&path, &paint, FillRule::EvenOdd, Transform::identity(), None);
        // let Some(path) = ({
        //     let mut pb = PathBuilder::new();
        //     pb.move_to(p1.x, p1.y);
        //     pb.line_to(self.end.x, self.end.y);
        //     pb.line_to(p2.x, p2.y);
        //     pb.line_to(p0b.x, p0b.y);
        //     pb.line_to(p0t.x, p0t.y);
        //     pb.close();
        //     pb.finish()
        // }) else {
        //     bail!("could not make path2");
        // };
        // canvas.pixmap.fill_path(
        //     &path,
        //     &paint,
        //     FillRule::EvenOdd,
        //     Transform::identity(),
        //     None,
        // );


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
    use crate::draw::GBox;
    use crate::style::standard_style;
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

        let arrow = Arrow::new(
            point(100.0, 100.0),
            point(375.0, 200.0),
            point(1.0, 0.0),
            point(1.0, 0.0),
            ArrowType::Fluid(FluidOptions {
                start_gravity: 50.0,
                end_gravity: 50.0,
            }),
            ArrowOptions {
                width: 20.0,
                head_length: 40.0,
                head_width: 40.0,
                color: color("#ff0")?,
                outline: Some(ArrowOutline {
                    width: 5.0,
                    color: color("#000")?,
                }),
                ..Default::default()
            },
        );
        arrow.draw(&mut canvas)?;

        canvas.save("test_render_arrow.png")?;
        Ok(())
    }
}
