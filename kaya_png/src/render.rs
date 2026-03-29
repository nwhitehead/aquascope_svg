use anyhow::Result;
use ab_glyph::{point, Font, Glyph, Point, ScaleFont};
use ab_glyph::{FontRef, FontVec, PxScale};
use image::{DynamicImage, ImageBuffer, Rgba};

const TEXT: &str = "This is ab_glyph rendered into a png!";

pub struct Canvas<'a> {
    img: ImageBuffer<Rgba<u8>, Vec<u8>>,
    font: Option<&'a FontVec>,
}

impl<'a> Canvas<'a> {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            img: DynamicImage::new_rgba8(256, 256).to_rgba8(),
            font: None,
        }
    }
    pub fn set_font(&mut self, font: &'a FontVec) {
        self.font = Some(font);
    }
    pub fn draw_glyph(&mut self, txt: &str, x: f32, y: f32, size: f32) {

    }
    pub fn save(&self, filename: &str) -> Result<()> {
        Ok(self.img.save(filename)?)
    }
}

pub fn test(filename: &str) -> Result<()> {

    let mut canvas = Canvas::new(256, 256);
    let mut image = DynamicImage::new_rgba8(256, 256).to_rgba8();

    let font = FontVec::try_from_vec(Vec::from(include_bytes!("../fonts/Lato-Regular.ttf")))?;
    canvas.set_font(&font);

    assert_eq!(font.glyph_id('s'), ab_glyph::GlyphId(118));
    // Get a glyph for 'q' with a scale & position.
    let q_glyph: Glyph = font
        .glyph_id('q')
        .with_scale_and_position(24.0, point(100.0, 0.0));

    // Draw it.
    let img_left = 100;
    let img_top = 100;
    /// Dark red colour
    const COLOUR: (u8, u8, u8) = (150, 0, 0);

    if let Some(q) = font.outline_glyph(q_glyph) {
        q.draw(|x, y, c| {
            /* draw pixel `(x, y)` with coverage: `c` */
            let px = image.get_pixel_mut(img_left + x, img_top + y);
            // Turn the coverage into an alpha value (blended with any previous)
            *px = Rgba([
                COLOUR.0,
                COLOUR.1,
                COLOUR.2,
                px.0[3].saturating_add((c * 255.0) as u8),
            ]);
        });
    }
    image.save(filename)?;
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
