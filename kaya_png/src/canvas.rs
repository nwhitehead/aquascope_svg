use std::collections::HashMap;

use ab_glyph::{Font, Glyph, Point, Rect, ScaleFont, point};
use ab_glyph::{FontVec, PxScale};
use anyhow::{Result, bail};
use tiny_skia::{ColorU8, Pixmap, PremultipliedColorU8};

use crate::draw_state::DrawState;

pub struct Canvas {
    pub pixmap: Pixmap,
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
        let color = &state.text_color.premultiply();
        // Now draw the outlines
        for glyph in outlined {
            let bounds = glyph.px_bounds();
            let x0 = bounds.min.x as i32;
            let y0 = bounds.min.y as i32;
            glyph.draw(|x, y, c| {
                if let Some(pmx) =
                    pixmap_pixel_mut(&mut self.pixmap, x0 + (x as i32), y0 + (y as i32))
                {
                    // Final dst alpha is: alpha_src + (1-alpha_src) * alpha_dst
                    // Final r is: r_src + r_dst * (1-alpha_src)
                    // Blend alpha with previous, sum opacity
                    let mcolor = ColorU8::from_rgba(
                        color.red(),
                        color.green(),
                        color.blue(),
                        pmx.alpha().saturating_add((c * (color.alpha() as f32)) as u8),
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
