use anyhow::{bail, Result};
use ab_glyph::{point, Font, Glyph, Point, Rect, ScaleFont};
use ab_glyph::{FontRef, FontVec, PxScale};
use image::{DynamicImage, ImageBuffer, Rgb, Rgba};

const TEXT: &str = "This is ab_glyph rendered into a png!";

pub struct Canvas<'a> {
    image: ImageBuffer<Rgba<u8>, Vec<u8>>,
    font: Option<&'a FontVec>,
}

pub struct Color(Rgb::<u8>);

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self(Rgb::<u8>([r, g, b]))
    }
}

impl<'a> Canvas<'a> {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            image: DynamicImage::new_rgba8(256, 256).to_rgba8(),
            font: None,
        }
    }
    pub fn set_font(&mut self, font: &'a FontVec) {
        self.font = Some(font);
    }
    pub fn draw_glyph(&mut self, c: char, x: f32, y: f32, size: f32, color: Color) -> Result<()> {
        let Some(f) = self.font else {
            bail!("no font");
        };
        let glyph: Glyph = f
            .glyph_id(c)
            .with_scale_and_position(size, point(x, y));
        let Some(outline) = f.outline_glyph(glyph) else {
            bail!("could not find glyph: '{c}'");
        };
        let x0 = x as u32;
        let y0 = y as u32;
        outline.draw(|x, y, c| {
            let px = self.image.get_pixel_mut(x0 + x, y0 + y);
            *px = Rgba([
                color.0.0[0],
                color.0.0[1],
                color.0.0[2],
                px.0[3].saturating_add((c * 255.0) as u8),
            ]);
        });

        Ok(())
    }
    fn layout_text(&self, text: &str, size: f32, position: Point, max_width: f32, target: &mut Vec<Glyph>) -> Result<Point> {
        let Some(font) = self.font else {
            bail!("no font");
        };
        let font = font.as_scaled(size);
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
    fn measure_glyphs(&self, glyphs: &[Glyph]) -> Rect {
        Rect { min: point(0.0, 0.0), max: point(1.0, 1.0) }
    }
    /// Measure text using font metric information
    // Actual drawn pixels may exceed the bounds returned here, but this is what should be used for computing layout
    pub fn measure_text(&self, text: &str, size: f32, max_width: f32) -> Result<Rect> {
        let mut glyphs = vec![];
        let caret = self.layout_text(text, size, point(0.0, 0.0), max_width, &mut glyphs)?;
        Ok(Rect { min: point(0.0, 0.0), max: caret})
    }
    pub fn save(&self, filename: &str) -> Result<()> {
        Ok(self.image.save(filename)?)
    }
}

pub fn test(filename: &str) -> Result<()> {

    let mut canvas = Canvas::new(256, 256);
    let mut image = DynamicImage::new_rgba8(256, 256).to_rgba8();

    let font = FontVec::try_from_vec(Vec::from(include_bytes!("../fonts/Lato-Regular.ttf")))?;
    canvas.set_font(&font);
    let color = Color::new(150, 0, 0);

    canvas.draw_glyph('&', 100.0, 100.0, 48.0, color)?;
    canvas.save(filename)?;
    let txt = "Hello, world!";
    println!("Size of \"{}\" is {:?}", txt, canvas.measure_text(txt, 48.0, 9999.0)?);

    Ok(())
}

// /// Simple paragraph layout for glyphs into `target`.
// /// Starts at position `(0, ascent)`.
// ///
// /// This is for testing and examples.
// pub fn layout_paragraph<F, SF>(
//     font: SF,
//     position: Point,
//     max_width: f32,
//     text: &str,
//     target: &mut Vec<Glyph>,
// ) where
//     F: Font,
//     SF: ScaleFont<F>,
// {
//     let v_advance = font.height() + font.line_gap();
//     let mut caret = position + point(0.0, font.ascent());
//     let mut last_glyph: Option<Glyph> = None;
//     for c in text.chars() {
//         if c.is_control() {
//             if c == '\n' {
//                 caret = point(position.x, caret.y + v_advance);
//                 last_glyph = None;
//             }
//             continue;
//         }
//         let mut glyph = font.scaled_glyph(c);
//         if let Some(previous) = last_glyph.take() {
//             caret.x += font.kern(previous.id, glyph.id);
//         }
//         glyph.position = caret;

//         last_glyph = Some(glyph.clone());
//         caret.x += font.h_advance(glyph.id);

//         if !c.is_whitespace() && caret.x > position.x + max_width {
//             caret = point(position.x, caret.y + v_advance);
//             glyph.position = caret;
//             last_glyph = None;
//         }

//         target.push(glyph);
//     }
// }
