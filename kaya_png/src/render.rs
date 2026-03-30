use anyhow::{bail, Result};
use ab_glyph::{point, Font, Glyph, Point, Rect, ScaleFont};
use ab_glyph::{FontRef, FontVec, PxScale};
use image::{DynamicImage, ImageBuffer, Rgb, Rgba};
use tiny_skia::*;
use std::collections::HashMap;

const TEXT: &str = "This is ab_glyph rendered into a png!";

pub struct Canvas {
    pixmap: Pixmap,
    image: ImageBuffer<Rgba<u8>, Vec<u8>>,
    fonts: HashMap<String, FontVec>,
    font: String,
}

pub struct Color(Rgb::<u8>);

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self(Rgb::<u8>([r, g, b]))
    }
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        let Some(pixmap) = Pixmap::new(width, height) else {
            bail!("pixmap failed");
        };
        Ok(Self {
            pixmap,
            image: DynamicImage::new_rgba8(width, height).to_rgba8(),
            fonts: HashMap::new(),
            font: "".into(),
        })
    }
    pub fn load_font(&mut self, name: &str, fontdata: &[u8]) -> Result<()> {
        self.fonts.insert(name.to_string(), FontVec::try_from_vec(Vec::from(fontdata))?);
        Ok(())
    }
    pub fn set_font(&mut self, name: &str) {
        self.font = name.into();
    }
    fn layout_text(&self, text: &str, size: f32, position: Point, max_width: f32, target: &mut Vec<Glyph>) -> Result<Point> {
        let Some(font) = self.fonts.get(&self.font) else {
            bail!("no font");
        };
        let font = font.as_scaled(PxScale::from(size));
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
            if !c.is_whitespace() && caret.x > max_width {
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
    pub fn measure_text(&self, text: &str, size: f32, max_width: f32) -> Result<Rect> {
        let mut glyphs = vec![];
        let caret = self.layout_text(text, size, point(0.0, 0.0), max_width, &mut glyphs)?;
        Ok(Rect { min: point(0.0, 0.0), max: caret})
    }
    pub fn draw_text(&mut self, text: &str, size: f32, position: Point, max_width: f32, color: Color) -> Result<()> {
        let Some(font) = self.fonts.get(&self.font) else {
            bail!("no font");
        };
        let font = font.as_scaled(PxScale::from(size));
        let mut glyphs = vec![];
        let caret = self.layout_text(text, size, position, max_width, &mut glyphs)?;
        let outlined: Vec<_> = glyphs
            .into_iter()
            .filter_map(|g| font.outline_glyph(g))
            .collect();
        // Now draw the outlines
        for glyph in outlined {
            let bounds = glyph.px_bounds();
            let x0 = bounds.min.x as u32;
            let y0 = bounds.min.y as u32;
            glyph.draw(|x, y, c| {
                let px = self.image.get_pixel_mut(x0 + x, y0 + y);
                // Blend alpha with previous, sum opacity
                *px = Rgba([
                    color.0.0[0],
                    color.0.0[1],
                    color.0.0[2],
                    px.0[3].saturating_add((c * 255.0) as u8),
                ]);
            });
        }

        Ok(())
    }
    pub fn save(&self, filename: &str) -> Result<()> {
        Ok(self.image.save(filename)?)
    }
}

pub fn test(filename: &str) -> Result<()> {

    let mut canvas = Canvas::new(800, 400)?;

    canvas.load_font("mono", include_bytes!("../fonts/DejaVu/DejaVuSansMono-Bold.ttf"))?;
    canvas.set_font("mono");
    let color = Color::new(150, 0, 0);

    let txt = "What font is this? 42";
    canvas.draw_text(txt, 48.0, point(100.0, 100.0), 9999.0, color)?;
    canvas.save(filename)?;

    println!("Size of \"{}\" is {:?}", txt, canvas.measure_text(txt, 48.0, 9999.0)?);

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

    let mut pixmap = Pixmap::new(500, 500).unwrap();
    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    pixmap.save_png("image.png").unwrap();
    Ok(())
}
