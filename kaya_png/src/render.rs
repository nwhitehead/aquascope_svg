use ab_glyph::{Font, Glyph, Point, Rect, ScaleFont, point};
use ab_glyph::{FontVec, PxScale};
use anyhow::{Result, bail};
use std::collections::HashMap;
use tiny_skia::*;

#[derive(Clone, Debug)]
struct DrawState {
    font: String,
    font_size: f32,
    font_max_width: f32,
    text_color: ColorU8,
    stroke_color: ColorU8,
    stroke: Stroke,
    border_radius: (f32, f32, f32, f32),
    border_clip: (bool, bool, bool, bool),
}

pub struct Canvas {
    pixmap: Pixmap,
    fonts: HashMap<String, FontVec>,
}

fn pixmap_pixel_mut(pixmap: &mut Pixmap, x: i32, y: i32) -> Option<&mut PremultipliedColorU8> {
    if x < 0 || x >= pixmap.width() as i32 {
        return None;
    }
    if y < 0 || y >= pixmap.height() as i32 {
        return None;
    }
    let idx = (pixmap.width() as i32).checked_mul(y)?.checked_add(x)?;
    Some(&mut pixmap.pixels_mut()[idx as usize])
}

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
pub struct GBox {
    r: Rect,
    state: DrawState,
}

impl GBox {
    fn new_xywh(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            r: Rect { min: point(x, y), max: point(x + w, y + h) },
            state: Default::default(),
        }
    }
    fn new_with_options(r: Rect, width: f32, color: ColorU8) -> Self {
        Self {
            r,
            state: DrawState { 
                stroke_color: color,
                stroke: Stroke { width, ..Default::default() },
                ..Default::default() },
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
                    self.r.max.x - br.1 * kappa, self.r.min.y,
                    self.r.max.x, self.r.min.y + br.1 * kappa,
                    self.r.max.x, self.r.min.y + br.1
                );
            }
            // right side
            pb.line_to(self.r.max.x, self.r.max.y - br.2);
            // lower right corner
            if bc.2 {
                pb.line_to(self.r.max.x - br.2, self.r.max.y);
            } else {
                pb.cubic_to(
                    self.r.max.x, self.r.max.y - br.2 * kappa,
                    self.r.max.x - br.2 * kappa, self.r.max.y,
                    self.r.max.x - br.2, self.r.max.y
                );
            }
            // bottom side
            pb.line_to(self.r.min.x + br.3, self.r.max.y);
            // lower left corner
            if bc.3 {
                pb.line_to(self.r.min.x, self.r.max.y - br.3);
            } else {
                pb.cubic_to(
                    self.r.min.x + br.3 * kappa, self.r.max.y,
                    self.r.min.x, self.r.max.y - br.3 * kappa,
                    self.r.min.x, self.r.max.y - br.3
                );
            }
            // left side
            pb.line_to(self.r.min.x, self.r.min.y + br.0);
            // upper left corner
            if bc.0 {
                pb.line_to(self.r.min.x + br.0, self.r.min.y);
            } else {
                pb.cubic_to(
                    self.r.min.x, self.r.min.y + br.0 * kappa,
                    self.r.min.x + br.0 * kappa, self.r.min.y,
                    self.r.min.x + br.0, self.r.min.y
                );
            }
            pb.close();
            pb.finish()
        }) else {
            bail!("could not make path");
        };
        canvas.pixmap.stroke_path(&path, &paint, &self.state.stroke, Transform::identity(), None);
        Ok(())
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
        let mut rbb: Rect = Rect { min: point(1000.0, 1000.0), max: point(-1000.0, -1000.0) };
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

impl Default for DrawState {
    fn default() -> Self {
        Self {
            font: "".into(),
            font_size: 24.0,
            font_max_width: 9999.0,
            stroke_color: ColorU8::from_rgba(0, 0, 0, 255),
            text_color: ColorU8::from_rgba(0, 0, 0, 255),
            stroke: Default::default(),
            border_radius: (0.0, 0.0, 0.0, 0.0),
            border_clip: (false, false, false, false),
        }
    }
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        let Some(pixmap) = Pixmap::new(width, height) else {
            bail!("pixmap failed");
        };
        Ok(Self {
            pixmap,
            fonts: HashMap::new(),
        })
    }
    pub fn load_font(&mut self, name: &str, fontdata: &[u8]) -> Result<()> {
        self.fonts
            .insert(name.into(), FontVec::try_from_vec(Vec::from(fontdata))?);
        Ok(())
    }
    /// position is at left, baseline
    fn layout_text(
        &self,
        text: &str,
        position: Point,
        state: &DrawState,
        target: &mut Vec<Glyph>,
    ) -> Result<Point> {
        let Some(font) = self.fonts.get(&state.font) else {
            bail!("no font");
        };
        let font = font.as_scaled(PxScale::from(state.font_size));
        let v_advance = font.height() + font.line_gap();
        let mut caret = position;
        // to make position be at upper left, add in this: + point(0.0, font.ascent())
        // to initial caret position
        let mut last_glyph: Option<Glyph> = None;
        for c in text.chars() {
            if c.is_control() {
                if c == '\n' {
                    caret = point(position.x, caret.y + v_advance);
                    last_glyph = None;
                }
                continue;
            }
            let mut glyph = font.scaled_glyph(c);
            if let Some(previous) = last_glyph.take() {
                caret.x += font.kern(previous.id, glyph.id);
            }
            glyph.position = caret;
            last_glyph = Some(glyph.clone());
            caret.x += font.h_advance(glyph.id);
            if !c.is_whitespace() && caret.x > state.font_max_width {
                caret = point(position.x, caret.y + v_advance);
                glyph.position = caret;
                last_glyph = None;
            }
            target.push(glyph);
        }
        Ok(caret)
    }
    /// Measure text using font metric information
    // Actual drawn pixels may exceed the bounds returned here, but this is what should be used for computing layout
    pub fn measure_text(&self, text: &str, state: &DrawState) -> Result<Rect> {
        let Some(font) = self.fonts.get(&state.font) else {
            bail!("no font");
        };
        let font = font.as_scaled(PxScale::from(state.font_size));
        let mut glyphs = vec![];
        let caret = self.layout_text(text, point(0.0, 0.0), &state, &mut glyphs)?;
        Ok(Rect {
            min: point(0.0, -font.ascent()),
            max: caret,
        })
    }
    pub fn draw_text(&mut self, text: &str, position: Point, state: &DrawState) -> Result<()> {
        let Some(font) = self.fonts.get(&state.font) else {
            bail!("no font");
        };
        let font = font.as_scaled(PxScale::from(state.font_size));
        let mut glyphs = vec![];
        let _ = self.layout_text(text, position, &state, &mut glyphs)?;
        let outlined: Vec<_> = glyphs
            .into_iter()
            .filter_map(|g| font.outline_glyph(g))
            .collect();
        let color = &state.text_color;
        // Now draw the outlines
        for glyph in outlined {
            let bounds = glyph.px_bounds();
            let x0 = bounds.min.x as i32;
            let y0 = bounds.min.y as i32;
            glyph.draw(|x, y, c| {
                if let Some(pmx) =
                    pixmap_pixel_mut(&mut self.pixmap, x0 + (x as i32), y0 + (y as i32))
                {
                    // Blend alpha with previous, sum opacity
                    let mcolor = ColorU8::from_rgba(
                        color.red(),
                        color.green(),
                        color.blue(),
                        pmx.alpha().saturating_add((c * 255.0) as u8),
                    );
                    *pmx = mcolor.premultiply();
                }
            });
        }

        Ok(())
    }
    pub fn save(&self, filename: &str) -> Result<()> {
        Ok(self.pixmap.save_png(filename)?)
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
    canvas: &Canvas
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
    Ok(stack_general(items, FormulaType::Centered, FormulaType::Centered, canvas)?)
}
pub fn hstack(items: Vec<Box<dyn Drawable>>, canvas: &Canvas) -> Result<GArray> {
    Ok(stack_general(items, FormulaType::Sequenced, FormulaType::Centered, canvas)?)
}
pub fn hstack_top(items: Vec<Box<dyn Drawable>>, canvas: &Canvas) -> Result<GArray> {
    Ok(stack_general(items, FormulaType::Sequenced, FormulaType::AlignLow, canvas)?)
}
pub fn hstack_bottom(items: Vec<Box<dyn Drawable>>, canvas: &Canvas) -> Result<GArray> {
    Ok(stack_general(items, FormulaType::Sequenced, FormulaType::AlignHigh, canvas)?)
}
pub fn vstack(items: Vec<Box<dyn Drawable>>, canvas: &Canvas) -> Result<GArray> {
    Ok(stack_general(items, FormulaType::Centered, FormulaType::Sequenced, canvas)?)
}
pub fn vstack_left(items: Vec<Box<dyn Drawable>>, canvas: &Canvas) -> Result<GArray> {
    Ok(stack_general(items, FormulaType::AlignLow, FormulaType::Sequenced, canvas)?)
}
pub fn vstack_right(items: Vec<Box<dyn Drawable>>, canvas: &Canvas) -> Result<GArray> {
    Ok(stack_general(items, FormulaType::AlignHigh, FormulaType::Sequenced, canvas)?)
}

pub fn outline(r: Rect, d: f32) -> Rect {
    Rect { min: point(r.min.x - d, r.min.y - d), max: point(r.max.x + d, r.max.y + d) }
}

pub fn box_around(item: &dyn Drawable, d: f32, canvas: &Canvas, state: &DrawState) -> Result<GBox> {
    let bb = item.bounding_box(canvas)?;
    let r = outline(bb, d);
    Ok(GBox { r, state: state.clone() })
}

pub fn test(filename: &str) -> Result<()> {
    let mut canvas = Canvas::new(800, 400)?;

    canvas.load_font(
        "mono",
        include_bytes!("../fonts/DejaVu/DejaVuSansMono-Bold.ttf"),
    )?;
    canvas.load_font(
        "serif",
        include_bytes!("../fonts/Lato/Lato-Regular.ttf"),
    )?;
    let state = DrawState {
        font: "mono".into(),
        text_color: ColorU8::from_rgba(150, 0, 0, 255),
        font_size: 48.0,
        ..Default::default()
    };

    let txt = GText {
        text: "✕✖✗✘×•●○◯42".into(),
        position: point(100.0, 100.0),
        state
    };
    let bb = txt.bounding_box(&canvas)?;
    let bx = GBox::new_with_options(bb, 4.0, ColorU8::from_rgba(0, 120, 0, 255));
    let mut ga = GArray::new();
    ga.push(Box::new(txt));
    ga.push(Box::new(bx));
    let bx2 = GBox::new_with_options(Rect { min: point(0.0, 0.0), max: point(100.0, 120.0) }, 4.0, ColorU8::from_rgba(0, 120, 120, 255));
    let stk = vstack_right(vec![Box::new(ga), Box::new(bx2)], &canvas)?;
    for item in &stk.items {
        println!("bb item = {:?}", item.bounding_box(&canvas)?);
    }
    let bb_stk = stk.bounding_box(&canvas)?;
    println!("bb_stk = {:?}", &bb_stk);
    let mut bx_state = DrawState { ..Default::default() };
    bx_state.stroke_color = ColorU8::from_rgba(0, 0, 255, 255);
    bx_state.stroke.width = 12.0;
    bx_state.border_radius = (40.0, 50.0, 40.0, 30.0);
    bx_state.border_clip = (false, false, true, false);
    let bx_bb_stk = box_around(&stk, 10.0, &canvas, &bx_state)?;
    stk.draw(&mut canvas)?;
    bx_bb_stk.draw(&mut canvas)?;

    canvas.save(filename)?;
    Ok(())
}
