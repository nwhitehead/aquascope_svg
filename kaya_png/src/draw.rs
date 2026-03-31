use ab_glyph::{Font, Glyph, Point, Rect, ScaleFont, point};
use ab_glyph::{FontVec, PxScale};
use anyhow::{Result, bail};
use std::collections::HashMap;
use tiny_skia::{ColorU8, Paint, PathBuilder, Stroke, Transform};

use crate::canvas::Canvas;
use crate::draw_state::DrawState;

pub trait Drawable {
    fn translate(&mut self, t: Point);
    fn bounding_box(&self, canvas: &Canvas) -> Result<Rect>;
    fn draw(&self, canvas: &mut Canvas) -> Result<()>;
}

pub struct GText {
    text: String,
    position: Point,
    state: DrawState,
}

impl GText {
    pub fn new(text: &str, position: Point, state: DrawState) -> Self {
        Self {
            text: text.to_string(),
            position,
            state,
        }
    }
}

impl Drawable for GText {
    fn translate(&mut self, t: Point) {
        self.position += t;
    }
    fn bounding_box(&self, canvas: &Canvas) -> Result<Rect> {
        let mut r = canvas.measure_text(&self.text, &self.state)?;
        r.min += self.position;
        r.max += self.position;
        Ok(r)
    }
    fn draw(&self, canvas: &mut Canvas) -> Result<()> {
        Ok(canvas.draw_text(&self.text, self.position, &self.state)?)
    }
}

#[derive(Clone, Debug)]
pub struct GLine {
    p0: Point,
    p1: Point,
    state: DrawState,
}

impl Drawable for GLine {
    fn translate(&mut self, t: Point) {
        self.p0 += t;
        self.p1 += t;
    }
    fn bounding_box(&self, canvas: &Canvas) -> Result<Rect> {
        let p0 = self.p0;
        let p1 = self.p1;
        Ok(Rect { min: point(p0.x.min(p1.x), p0.y.min(p1.y)),
            max: point(p0.x.max(p1.x), p0.y.max(p1.y)) })
    }
    fn draw(&self, canvas: &mut Canvas) -> Result<()> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct GBox {
    r: Rect,
    state: DrawState,
}

impl GBox {
    fn new_xywh(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            r: Rect {
                min: point(x, y),
                max: point(x + w, y + h),
            },
            state: Default::default(),
        }
    }
    fn new_with_options(r: Rect, width: f32, color: ColorU8) -> Self {
        Self {
            r,
            state: DrawState {
                stroke_color: color,
                stroke: Stroke {
                    width,
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}

impl Drawable for GBox {
    fn translate(&mut self, t: Point) {
        self.r.min += t;
        self.r.max += t;
    }
    fn bounding_box(&self, canvas: &Canvas) -> Result<Rect> {
        Ok(self.r)
    }
    fn draw(&self, canvas: &mut Canvas) -> Result<()> {
        let color = self.state.stroke_color;
        let br = self.state.border_radius;
        let bc = self.state.border_clip;
        let mut paint = Paint::default();
        paint.set_color_rgba8(color.red(), color.green(), color.blue(), color.alpha());
        paint.anti_alias = true;
        let kappa = 1.0 - 0.552228;
        let Some(path) = ({
            let mut pb = PathBuilder::new();
            pb.move_to(self.r.min.x + br.0, self.r.min.y);
            // top side
            pb.line_to(self.r.max.x - br.1, self.r.min.y);
            // upper right corner
            if bc.1 {
                pb.line_to(self.r.max.x, self.r.min.y + br.1);
            } else {
                pb.cubic_to(
                    self.r.max.x - br.1 * kappa,
                    self.r.min.y,
                    self.r.max.x,
                    self.r.min.y + br.1 * kappa,
                    self.r.max.x,
                    self.r.min.y + br.1,
                );
            }
            // right side
            pb.line_to(self.r.max.x, self.r.max.y - br.2);
            // lower right corner
            if bc.2 {
                pb.line_to(self.r.max.x - br.2, self.r.max.y);
            } else {
                pb.cubic_to(
                    self.r.max.x,
                    self.r.max.y - br.2 * kappa,
                    self.r.max.x - br.2 * kappa,
                    self.r.max.y,
                    self.r.max.x - br.2,
                    self.r.max.y,
                );
            }
            // bottom side
            pb.line_to(self.r.min.x + br.3, self.r.max.y);
            // lower left corner
            if bc.3 {
                pb.line_to(self.r.min.x, self.r.max.y - br.3);
            } else {
                pb.cubic_to(
                    self.r.min.x + br.3 * kappa,
                    self.r.max.y,
                    self.r.min.x,
                    self.r.max.y - br.3 * kappa,
                    self.r.min.x,
                    self.r.max.y - br.3,
                );
            }
            // left side
            pb.line_to(self.r.min.x, self.r.min.y + br.0);
            // upper left corner
            if bc.0 {
                pb.line_to(self.r.min.x + br.0, self.r.min.y);
            } else {
                pb.cubic_to(
                    self.r.min.x,
                    self.r.min.y + br.0 * kappa,
                    self.r.min.x + br.0 * kappa,
                    self.r.min.y,
                    self.r.min.x + br.0,
                    self.r.min.y,
                );
            }
            pb.close();
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
        Ok(())
    }
}

pub struct GPadding {
    item: Box<dyn Drawable>,
    // padding is left, top, right, bottom
    padding: (f32, f32, f32, f32),
}

impl GPadding {
    pub fn new(item: Box<dyn Drawable>, padding: (f32, f32, f32, f32)) -> Self {
        Self { item, padding }
    }
}

impl Drawable for GPadding {
    fn translate(&mut self, t: Point) {
        self.item.translate(t);
    }
    fn bounding_box(&self, canvas: &Canvas) -> Result<Rect> {
        let mut bb = self.item.bounding_box(&canvas)?;
        bb.min.x -= self.padding.0;
        bb.min.y -= self.padding.1;
        bb.max.x += self.padding.2;
        bb.max.y += self.padding.3;
        Ok(bb)
    }
    fn draw(&self, canvas: &mut Canvas) -> Result<()> {
        Ok(self.item.draw(canvas)?)
    }
}

pub struct GArray {
    items: Vec<Box<dyn Drawable>>,
}

impl GArray {
    pub fn push(&mut self, item: Box<dyn Drawable>) {
        self.items.push(item);
    }
    pub fn new() -> Self {
        Self { items: vec![] }
    }
}

impl Drawable for GArray {
    fn translate(&mut self, t: Point) {
        for item in &mut self.items {
            item.translate(t);
        }
    }

    fn bounding_box(&self, canvas: &Canvas) -> Result<Rect> {
        let mut rbb: Rect = Rect {
            min: point(1000.0, 1000.0),
            max: point(-1000.0, -1000.0),
        };
        for item in &self.items {
            let bb = item.bounding_box(&canvas)?;
            rbb.min.x = rbb.min.x.min(bb.min.x);
            rbb.min.y = rbb.min.y.min(bb.min.y);
            rbb.max.x = rbb.max.x.max(bb.max.x);
            rbb.max.y = rbb.max.y.max(bb.max.y);
        }
        Ok(rbb)
    }

    fn draw(&self, canvas: &mut Canvas) -> Result<()> {
        for item in &self.items {
            item.draw(canvas)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
enum FormulaType {
    AlignLow,
    AlignHigh,
    Centered,
    Sequenced,
}

fn apply_formula(formula: &FormulaType, x: f32, xx: f32, ix: f32, ixx: f32) -> f32 {
    match formula {
        FormulaType::AlignLow => x - ix,
        FormulaType::AlignHigh => xx - ixx,
        FormulaType::Centered => (x + 0.5 * (xx - x)) - (ix + 0.5 * (ixx - ix)),
        FormulaType::Sequenced => xx - ix,
    }
}

fn stack_general(
    items: Vec<Box<dyn Drawable>>,
    tx_formula: FormulaType,
    ty_formula: FormulaType,
    canvas: &Canvas,
) -> Result<GArray> {
    let mut bb: Option<Rect> = None;
    let mut c = GArray::new();
    for mut item in items {
        let item_bb = item.bounding_box(canvas)?.clone();
        if let Some(ref b) = bb {
            let tx = apply_formula(&tx_formula, b.min.x, b.max.x, item_bb.min.x, item_bb.max.x);
            let ty = apply_formula(&ty_formula, b.min.y, b.max.y, item_bb.min.y, item_bb.max.y);
            item.translate(point(tx, ty));
        } else {
            bb = Some(item.bounding_box(canvas)?);
        }
        c.push(item);
    }
    Ok(c)
}
pub fn stack(items: Vec<Box<dyn Drawable>>, canvas: &Canvas) -> Result<GArray> {
    Ok(stack_general(
        items,
        FormulaType::Centered,
        FormulaType::Centered,
        canvas,
    )?)
}
pub fn hstack(items: Vec<Box<dyn Drawable>>, canvas: &Canvas) -> Result<GArray> {
    Ok(stack_general(
        items,
        FormulaType::Sequenced,
        FormulaType::Centered,
        canvas,
    )?)
}
pub fn hstack_top(items: Vec<Box<dyn Drawable>>, canvas: &Canvas) -> Result<GArray> {
    Ok(stack_general(
        items,
        FormulaType::Sequenced,
        FormulaType::AlignLow,
        canvas,
    )?)
}
pub fn hstack_bottom(items: Vec<Box<dyn Drawable>>, canvas: &Canvas) -> Result<GArray> {
    Ok(stack_general(
        items,
        FormulaType::Sequenced,
        FormulaType::AlignHigh,
        canvas,
    )?)
}
pub fn vstack(items: Vec<Box<dyn Drawable>>, canvas: &Canvas) -> Result<GArray> {
    Ok(stack_general(
        items,
        FormulaType::Centered,
        FormulaType::Sequenced,
        canvas,
    )?)
}
pub fn vstack_left(items: Vec<Box<dyn Drawable>>, canvas: &Canvas) -> Result<GArray> {
    Ok(stack_general(
        items,
        FormulaType::AlignLow,
        FormulaType::Sequenced,
        canvas,
    )?)
}
pub fn vstack_right(items: Vec<Box<dyn Drawable>>, canvas: &Canvas) -> Result<GArray> {
    Ok(stack_general(
        items,
        FormulaType::AlignHigh,
        FormulaType::Sequenced,
        canvas,
    )?)
}

pub fn outline(r: Rect, d: f32) -> Rect {
    Rect {
        min: point(r.min.x - d, r.min.y - d),
        max: point(r.max.x + d, r.max.y + d),
    }
}

pub fn box_around(item: &dyn Drawable, d: f32, canvas: &Canvas, state: DrawState) -> Result<GBox> {
    let bb = item.bounding_box(canvas)?;
    let r = outline(bb, d);
    Ok(GBox { r, state })
}

/// A border has padding around an item, then drawn border, then margin around the result
// Get all stuff from drawstate not args here
pub fn border(
    item: Box<dyn Drawable>,
    canvas: &Canvas,
    state: DrawState,
) -> Result<Box<dyn Drawable>> {
    let mut res = GArray::new();
    let padded_item = GPadding::new(item, state.padding);
    let bb = padded_item.bounding_box(canvas)?;
    let border_padded_item = GBox {
        r: bb,
        state: state.clone(),
    };
    res.push(Box::new(padded_item));
    res.push(Box::new(border_padded_item));
    let padded_garray = GPadding::new(Box::new(res), state.margin);
    Ok(Box::new(padded_garray))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_drawing() -> Result<()> {
        let mut canvas = Canvas::new(800, 800)?;

        canvas.load_font(
            "mono",
            include_bytes!("../fonts/DejaVu/DejaVuSansMono-Bold.ttf"),
        )?;
        canvas.load_font("serif", include_bytes!("../fonts/Lato/Lato-Regular.ttf"))?;
        let state = DrawState {
            font: "mono".into(),
            text_color: ColorU8::from_rgba(150, 0, 0, 255),
            font_size: 48.0,
            ..Default::default()
        };

        let txt = GText {
            text: "✕✖✗✘×•●○◯42".into(),
            position: point(100.0, 100.0),
            state,
        };
        let bb = txt.bounding_box(&canvas)?;
        let bx = GBox::new_with_options(bb, 4.0, ColorU8::from_rgba(0, 120, 0, 255));
        let mut ga = GArray::new();
        ga.push(Box::new(txt));
        ga.push(Box::new(bx));
        let bx2 = GBox::new_with_options(
            Rect {
                min: point(0.0, 0.0),
                max: point(100.0, 120.0),
            },
            4.0,
            ColorU8::from_rgba(0, 120, 120, 255),
        );
        let stk = vstack_right(vec![Box::new(ga), Box::new(bx2)], &canvas)?;
        for item in &stk.items {
            println!("bb item = {:?}", item.bounding_box(&canvas)?);
        }
        let bb_stk = stk.bounding_box(&canvas)?;
        println!("bb_stk = {:?}", &bb_stk);
        let mut bx_state = DrawState {
            ..Default::default()
        };
        bx_state.stroke_color = ColorU8::from_rgba(0, 0, 255, 255);
        bx_state.stroke.width = 12.0;
        bx_state.border_radius = (40.0, 50.0, 40.0, 30.0);
        bx_state.border_clip = (false, false, true, false);
        let bx_bb_stk = box_around(&stk, 10.0, &canvas, bx_state)?;
        stk.draw(&mut canvas)?;
        bx_bb_stk.draw(&mut canvas)?;

        let mut s = DrawState {
            ..Default::default()
        };
        s.font = "mono".to_string();
        s.stroke_color = ColorU8::from_rgba(128, 0, 128, 255);
        s.stroke.width = 2.0;
        s.border_radius = (5.0, 5.0, 5.0, 5.0);
        s.border_clip = (false, false, false, false);
        s.padding = (60.0, 30.0, 60.0, 30.0);
        s.margin = (40.0, 10.0, 40.0, 10.0);

        let g1 = border(
            Box::new(GText::new("CSS", point(200.0, 400.0), s.clone())),
            &canvas,
            s.clone(),
        )?;
        let g2 = border(
            Box::new(GText::new("w/ layout", point(200.0, 400.0), s.clone())),
            &canvas,
            s.clone(),
        )?;
        let vs = vstack_left(vec![g1, g2], &canvas)?;
        vs.draw(&mut canvas)?;

        canvas.save("test_drawing.png")?;
        Ok(())
    }
}
