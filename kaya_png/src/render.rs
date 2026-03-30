use anyhow::{bail, Result};
use ab_glyph::{point, Font, Glyph, Point, Rect, ScaleFont};
use ab_glyph::{FontRef, FontVec, PxScale};
use image::{DynamicImage, ImageBuffer, Rgb, Rgba};
use tiny_skia::*;
use std::collections::HashMap;

const TEXT: &str = "This is ab_glyph rendered into a png!";

pub struct Canvas {
    pixmap: Pixmap,
    fonts: HashMap<String, FontVec>,
    font: String,
    font_size: f32,
    font_max_width: f32,
    color: ColorU8,
}

fn pixmap_pixel_mut(pixmap: &mut Pixmap, x: u32, y: u32) -> Option<&mut PremultipliedColorU8> {
    let idx = pixmap.width().checked_mul(y)?.checked_add(x)?;
    Some(&mut pixmap.pixels_mut()[idx as usize])
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        let Some(pixmap) = Pixmap::new(width, height) else {
            bail!("pixmap failed");
        };
        Ok(Self {
            pixmap,
            fonts: HashMap::new(),
            font: "".into(),
            font_size: 24.0,
            font_max_width: 9999.0,
            color: ColorU8::from_rgba(0, 0, 0, 255),
        })
    }
    pub fn load_font(&mut self, name: &str, fontdata: &[u8]) -> Result<()> {
        self.fonts.insert(name.into(), FontVec::try_from_vec(Vec::from(fontdata))?);
        if self.font == "" {
            self.font = name.into();
        }
        Ok(())
    }
    pub fn set_font(&mut self, name: &str) {
        self.font = name.into();
    }
    pub fn set_font_size(&mut self, font_size: f32) {
        self.font_size = font_size;
    }
    pub fn set_color(&mut self, color: ColorU8) {
        self.color = color;
    }
    fn layout_text(&self, text: &str, position: Point, target: &mut Vec<Glyph>) -> Result<Point> {
        let Some(font) = self.fonts.get(&self.font) else {
            bail!("no font");
        };
        let font = font.as_scaled(PxScale::from(self.font_size));
        let v_advance = font.height() + font.line_gap();
        let mut caret = position + point(0.0, font.ascent());
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
            if !c.is_whitespace() && caret.x > self.font_max_width {
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
    pub fn measure_text(&self, text: &str) -> Result<Rect> {
        let mut glyphs = vec![];
        let caret = self.layout_text(text, point(0.0, 0.0), &mut glyphs)?;
        Ok(Rect { min: point(0.0, 0.0), max: caret})
    }
    pub fn draw_text(&mut self, text: &str, position: Point) -> Result<()> {
        let Some(font) = self.fonts.get(&self.font) else {
            bail!("no font");
        };
        let font = font.as_scaled(PxScale::from(self.font_size));
        let mut glyphs = vec![];
        let caret = self.layout_text(text, position, &mut glyphs)?;
        let outlined: Vec<_> = glyphs
            .into_iter()
            .filter_map(|g| font.outline_glyph(g))
            .collect();
        let color = &self.color;
        let mcolor = color.premultiply();
        // Now draw the outlines
        for glyph in outlined {
            let bounds = glyph.px_bounds();
            let x0 = bounds.min.x as u32;
            let y0 = bounds.min.y as u32;
            glyph.draw(|x, y, c| {
                if let Some(pmx) = pixmap_pixel_mut(&mut self.pixmap, x0 + x, y0 + y) {
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

    canvas.load_font("mono", include_bytes!("../fonts/DejaVu/DejaVuSansMono-Bold.ttf"))?;
    canvas.set_font("mono");
    canvas.set_color(ColorU8::from_rgba(150, 0, 0, 255));
    canvas.set_font_size(48.0);

    let txt = "What font is this? 42";
    canvas.draw_text(txt, point(100.0, 100.0))?;

    println!("Size of \"{}\" is {:?}", txt, canvas.measure_text(txt)?);

    let mut paint = Paint::default();
    paint.set_color_rgba8(0, 127, 0, 200);
    paint.anti_alias = true;

    let path = {
        let mut pb = PathBuilder::new();
        const RADIUS: f32 = 250.0;
        const CENTER: f32 = 250.0;
        pb.move_to(CENTER + RADIUS, CENTER);
        for i in 1..8 {
            let a = 2.6927937 * i as f32;
            pb.line_to(CENTER + RADIUS * a.cos(), CENTER + RADIUS * a.sin());
        }
        pb.finish().unwrap()
    };

    let mut stroke = Stroke::default();
    stroke.width = 6.0;
    stroke.line_cap = LineCap::Round;
    stroke.dash = StrokeDash::new(vec![20.0, 40.0], 0.0);

    canvas.pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    canvas.save(filename)?;
    Ok(())
}
