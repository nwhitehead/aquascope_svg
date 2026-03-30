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

#[derive(Clone, Debug)]
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
        let mut paint = Paint::default();
        paint.set_color_rgba8(color.red(), color.green(), color.blue(), color.alpha());
        paint.anti_alias = true;
        let Some(path) = ({
            let mut pb = PathBuilder::new();
            pb.move_to(self.r.min.x, self.r.min.y);
            pb.line_to(self.r.max.x, self.r.min.y);
            pb.line_to(self.r.max.x, self.r.max.y);
            pb.line_to(self.r.min.x, self.r.max.y);
            pb.line_to(self.r.min.x, self.r.min.y);
            pb.finish()
        }) else {
            bail!("could not make path");
        };
        canvas.pixmap.stroke_path(&path, &paint, &self.state.stroke, Transform::identity(), None);
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

pub fn test(filename: &str) -> Result<()> {
    let mut canvas = Canvas::new(800, 400)?;

    canvas.load_font(
        "mono",
        include_bytes!("../fonts/DejaVu/DejaVuSansMono-Bold.ttf"),
    )?;
    let state = DrawState {
        font: "mono".into(),
        text_color: ColorU8::from_rgba(150, 0, 0, 255),
        font_size: 48.0,
        ..Default::default()
    };

    let txt = GText {
        text: "42".into(),
        position: point(100.0, 100.0),
        state
    };
    let bb = txt.bounding_box(&canvas)?;
    txt.draw(&mut canvas)?;
    let bx = GBox::new_with_options(bb, 4.0, ColorU8::from_rgba(0, 120, 0, 255));
    bx.draw(&mut canvas)?;

    canvas.save(filename)?;
    Ok(())
}
