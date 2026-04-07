#![allow(unused)]

use ab_glyph::{Point, Rect, point};
use anyhow::{Result, bail};
use tiny_skia::{ColorU8, FillRule, LineCap, LineJoin, Paint, PathBuilder, Stroke, Transform};

use crate::canvas::Canvas;
use crate::draw::{Drawable, norm, scale};
use crate::draw_state::DrawState;
use crate::style::color;

#[derive(Clone, Debug)]
pub struct FluidOptions {
    pub start_gravity: f32,
    pub end_gravity: f32,
    pub start_dir: Point,
    pub end_dir: Point,
}

#[derive(Clone, Debug)]
pub struct ArcOptions {
    pub start_dir: Point,
    pub end_dir: Point,
}

#[derive(Clone, Debug)]
pub struct ArrowOutline {
    pub width: f32,
    pub color: ColorU8,
}

#[derive(Clone, Debug)]
pub enum ArrowType {
    Straight,
    Fluid(FluidOptions),
    Arc(ArcOptions),
}

#[derive(Clone, Debug)]
pub struct ArrowOptions {
    pub width: f32,
    pub color: ColorU8,
    pub outline: Option<ArrowOutline>,
    pub head_width: f32,
    pub head_length: f32,
    pub dent_ratio: f32,
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
    arrow_type: ArrowType,
    options: ArrowOptions,
}

/// Decompose a direction vector into two vectors: along and parallel (unit length)
pub fn decomp(dir: Point) -> (Point, Point) {
    let norm = 1.0 / norm(dir);
    let parallel = point(dir.x * norm, dir.y * norm);
    let perp = point(parallel.y, -parallel.x);
    (parallel, perp)
}

impl Arrow {
    pub fn new(
        start: Point,
        end: Point,
        arrow_type: ArrowType,
        options: ArrowOptions,
    ) -> Self {
        Self {
            start,
            end,
            arrow_type,
            options,
        }
    }
}

fn draw_fluid_arrow(start: Point, end: Point, options: &ArrowOptions, fluid_options: &FluidOptions, canvas: &mut Canvas) -> Result<()> {
    let transform = Transform::from_scale(canvas.scale, canvas.scale);
    let (par, perp) = decomp(fluid_options.end_dir);
    let (par_src, perp_src) = decomp(fluid_options.start_dir);
    let head_length = options.head_length;
    let head_width = options.head_width;
    let arrow_dent_ratio = options.dent_ratio;
    let end_control = scale(par, -head_length - fluid_options.end_gravity);
    let start_control = scale(par_src, fluid_options.start_gravity);
    // p0 is where thick line ends (before actual tip)
    let p0 = end + scale(par, -head_length);
    // p1 and p2 are tips on sides
    let head_offset = scale(par, -head_length * arrow_dent_ratio);
    let p1 = p0 + head_offset + scale(perp, head_width);
    let p2 = p0 + head_offset + scale(perp, -head_width);
    // p0t and p0b are widened points where line actually ends
    // includes "- par" to close gap between line and arrowhead
    let p0t = p0 + scale(perp, options.width * 0.5) - par;
    let p0b = p0 + scale(perp, -options.width * 0.5) - par;
    // body_path is line from start to end
    let Some(body_path) = ({
        let mut pb = PathBuilder::new();
        pb.move_to(start.x, start.y);
        pb.cubic_to(
            start.x + start_control.x,
            start.y + start_control.y,
            p0.x + end_control.x,
            p0.y + end_control.y,
            p0.x,
            p0.y,
        );
        pb.finish()
    }) else {
        bail!("could not make path");
    };
    // head path is around arrowhead
    let Some(head_path) = ({
        let mut pb = PathBuilder::new();
        pb.move_to(p1.x, p1.y);
        pb.line_to(end.x, end.y);
        pb.line_to(p2.x, p2.y);
        pb.line_to(p0b.x, p0b.y);
        pb.line_to(p0t.x, p0t.y);
        pb.close();
        pb.finish()
    }) else {
        bail!("could not make path2");
    };

    let mut paint = Paint {
        anti_alias: true,
        ..Paint::default()
    };

    // Draw outline if set
    if let Some(ref arrow_outline) = options.outline {
        let color = arrow_outline.color;
        paint.set_color_rgba8(color.red(), color.green(), color.blue(), color.alpha());
        // body
        let stroke = Stroke {
            width: options.width + arrow_outline.width,
            line_cap: LineCap::Square,
            ..Default::default()
        };
        canvas
            .pixmap
            .stroke_path(&body_path, &paint, &stroke, transform, None);
        // head
        let stroke = Stroke {
            width: arrow_outline.width,
            line_join: LineJoin::Round,
            ..Default::default()
        };
        canvas
            .pixmap
            .stroke_path(&head_path, &paint, &stroke, transform, None);
    }
    // Draw main body of arrow
    let color = options.color;
    let stroke = Stroke {
        width: options.width,
        line_cap: LineCap::Square,
        ..Default::default()
    };
    paint.set_color_rgba8(color.red(), color.green(), color.blue(), color.alpha());
    canvas
        .pixmap
        .stroke_path(&body_path, &paint, &stroke, transform, None);

    // Draw arrow head
    canvas.pixmap.fill_path(
        &head_path,
        &paint,
        FillRule::EvenOdd,
        transform,
        None,
    );

    Ok(())
}

fn draw_straight_arrow(start: Point, end: Point, options: &ArrowOptions, canvas: &mut Canvas) -> Result<()> {
    let fluid_options = FluidOptions {
        start_gravity: 0.0,
        end_gravity: 0.0,
        start_dir: end - start,
        end_dir: end - start,
    };
    draw_fluid_arrow(start, end, options, &fluid_options, canvas)
}

fn draw_arc_arrow(start: Point, end: Point, options: &ArrowOptions, arc_options: &ArcOptions, canvas: &mut Canvas) -> Result<()> {
    let gravity = norm(end - start) * 0.5;
    let fluid_options = FluidOptions {
        start_gravity: gravity,
        end_gravity: gravity,
        start_dir: arc_options.start_dir,
        end_dir: arc_options.end_dir,
    };
    draw_fluid_arrow(start, end, options, &fluid_options, canvas)
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
        match self.arrow_type {
            ArrowType::Fluid(ref fluid_options) => draw_fluid_arrow(self.start, self.end, &self.options, fluid_options, canvas),
            ArrowType::Straight => draw_straight_arrow(self.start, self.end, &self.options, canvas),
            ArrowType::Arc(ref arc_options) => draw_arc_arrow(self.start, self.end, &self.options, arc_options, canvas),
        }
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
    use rand::{RngExt, SeedableRng, rngs::ChaCha8Rng};
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

        Arrow::new(
            point(600.0, 700.0),
            point(400.0, 400.0),
            ArrowType::Fluid(FluidOptions {
                start_gravity: 100.0,
                end_gravity: 100.0,
                start_dir: point(0.2, -0.3),
                end_dir: point(-0.2, 0.0),
            }),
            ArrowOptions {
                width: 20.0,
                head_length: 40.0,
                head_width: 40.0,
                dent_ratio: 0.2,
                color: color("#ff0")?,
                outline: Some(ArrowOutline {
                    width: 10.0,
                    color: color("#000")?,
                }),
                ..Default::default()
            },
        ).draw(&mut canvas)?;

        Arrow::new(
            point(200.0, 200.0),
            point(400.0, 300.0),
            ArrowType::Straight,
            ArrowOptions {
                width: 20.0,
                head_length: 40.0,
                head_width: 40.0,
                dent_ratio: 0.2,
                color: color("#ff0")?,
                outline: Some(ArrowOutline {
                    width: 10.0,
                    color: color("#000")?,
                }),
                ..Default::default()
            },
        ).draw(&mut canvas)?;

        Arrow::new(
            point(600.0, 200.0),
            point(400.0, 600.0),
            ArrowType::Arc( ArcOptions {
                start_dir: point(0.0, 1.0),
                end_dir: point(-1.0, 0.0),
            }),
            ArrowOptions {
                width: 20.0,
                head_length: 40.0,
                head_width: 40.0,
                dent_ratio: 0.2,
                color: color("#ff0")?,
                outline: Some(ArrowOutline {
                    width: 10.0,
                    color: color("#000")?,
                }),
                ..Default::default()
            },
        ).draw(&mut canvas)?;

        canvas.save("test_render_arrow.png")?;
        Ok(())
    }
}
