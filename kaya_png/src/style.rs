use anyhow::{Result, bail};
use std::collections::HashMap;
use tiny_skia::ColorU8;

/// Encode values into one datatype for remembering styling info
// This simplifies the interface to the styling data
// Note that if you do add_number("radius", 5.0) then do
// get_bool("radius"), it will return None and just ignore the 5.0.
#[derive(Clone, Debug)]
pub enum AnyValue {
    Number(f32),
    Bool(bool),
    String(String),
    Color(ColorU8),
}

#[derive(Clone, Debug, Default)]
pub struct Styling {
    data: HashMap<String, AnyValue>,
}

impl Styling {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    pub fn add_number(&mut self, key: &str, value: f32) {
        self.data.insert(key.to_string(), AnyValue::Number(value));
    }
    pub fn add_bool(&mut self, key: &str, value: bool) {
        self.data.insert(key.to_string(), AnyValue::Bool(value));
    }
    pub fn add_string(&mut self, key: &str, value: &str) {
        self.data
            .insert(key.to_string(), AnyValue::String(value.to_string()));
    }
    pub fn add_color(&mut self, key: &str, value: ColorU8) {
        self.data.insert(key.to_string(), AnyValue::Color(value));
    }
    pub fn get_number(&self, key: &str) -> Option<f32> {
        match self.data.get(key) {
            Some(AnyValue::Number(x)) => Some(*x),
            _ => None,
        }
    }
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.data.get(key) {
            Some(AnyValue::Bool(x)) => Some(*x),
            _ => None,
        }
    }
    pub fn get_string(&self, key: &str) -> Option<String> {
        match self.data.get(key) {
            Some(AnyValue::String(x)) => Some(x.clone()),
            _ => None,
        }
    }
    pub fn get_color(&self, key: &str) -> Option<ColorU8> {
        match self.data.get(key) {
            Some(AnyValue::Color(x)) => Some(*x),
            _ => None,
        }
    }
    pub fn get_number_or(&self, key: &str, value: f32) -> f32 {
        self.get_number(key).unwrap_or(value)
    }
    pub fn get_bool_or(&self, key: &str, value: bool) -> bool {
        self.get_bool(key).unwrap_or(value)
    }
    pub fn get_string_or(&self, key: &str, value: &str) -> String {
        self.get_string(key).unwrap_or(value.to_string())
    }
    pub fn get_color_or(&self, key: &str, value: ColorU8) -> ColorU8 {
        self.get_color(key).unwrap_or(value)
    }
    // Synthetic methods, computing values for simplicity of queries
    pub fn get_padding(&self, key: &str, value: f32) -> (f32, f32, f32, f32) {
        let v = self.get_number_or(key, value);
        (
            self.get_number_or(&format!("{}.left", key), v),
            self.get_number_or(&format!("{}.top", key), v),
            self.get_number_or(&format!("{}.right", key), v),
            self.get_number_or(&format!("{}.bottom", key), v),
        )
    }
    pub fn get_radius(&self, key: &str, value: f32) -> (f32, f32, f32, f32) {
        let v = self.get_number_or(key, value);
        (
            self.get_number_or(&format!("{}.nw", key), v),
            self.get_number_or(&format!("{}.ne", key), v),
            self.get_number_or(&format!("{}.se", key), v),
            self.get_number_or(&format!("{}.sw", key), v),
        )
    }
}

pub fn color(txt: &str) -> Result<ColorU8> {
    let r;
    let g;
    let b;
    let mut a = 255;
    if !txt.starts_with('#') {
        bail!("colors must start with #");
    }
    let txt = txt.trim_start_matches('#');
    match txt.len() {
        3 => {
            r = u8::from_str_radix(&txt[0..1], 16)? * (0x10 + 0x01);
            g = u8::from_str_radix(&txt[1..2], 16)? * (0x10 + 0x01);
            b = u8::from_str_radix(&txt[2..3], 16)? * (0x10 + 0x01);
        }
        4 => {
            r = u8::from_str_radix(&txt[0..1], 16)? * (0x10 + 0x01);
            g = u8::from_str_radix(&txt[1..2], 16)? * (0x10 + 0x01);
            b = u8::from_str_radix(&txt[2..3], 16)? * (0x10 + 0x01);
            a = u8::from_str_radix(&txt[3..4], 16)? * (0x10 + 0x01);
        }
        6 => {
            r = u8::from_str_radix(&txt[0..2], 16)?;
            g = u8::from_str_radix(&txt[2..4], 16)?;
            b = u8::from_str_radix(&txt[4..6], 16)?;
        }
        8 => {
            r = u8::from_str_radix(&txt[0..2], 16)?;
            g = u8::from_str_radix(&txt[2..4], 16)?;
            b = u8::from_str_radix(&txt[4..6], 16)?;
            a = u8::from_str_radix(&txt[6..8], 16)?;
        }
        _ => bail!("unknown color length"),
    }
    Ok(ColorU8::from_rgba(r, g, b, a))
}

pub fn standard_style() -> Result<Styling> {
    let mut style = Styling::new();
    style.add_string("value.number.font", "mono");
    style.add_number("value.number.font_size", 23.0);
    style.add_color("value.number.color", color("#bccfa9")?);
    style.add_number("value.number.padding", 5.0);
    style.add_number("value.number.padding.bottom", 8.0);
    style.add_string("value.char.font", "mono");
    style.add_number("value.char.font_size", 23.0);
    style.add_color("value.char.color", color("#bf947a")?);
    style.add_string("value.pointer.font", "mono");
    style.add_number("value.pointer.font_size", 23.0);
    style.add_color("value.pointer.color", color("#ccc")?);
    style.add_number("value.array.empty.w", 0.0);
    style.add_number("value.array.empty.h", 20.0);
    style.add_color("value.array.separator.color", color("#7197d580")?);
    style.add_number("value.array.separator.vmargin", 5.0);
    style.add_number("value.array.separator.padding.left", 10.0);
    style.add_number("value.array.separator.padding.top", 5.0);
    style.add_number("value.array.separator.padding.right", 10.0);
    style.add_number("value.array.separator.padding.bottom", 5.0);
    style.add_number("value.array.padding.left", 10.0);
    style.add_number("value.array.padding.top", 2.0);
    style.add_number("value.array.padding.right", 10.0);
    style.add_number("value.array.padding.bottom", 2.0);
    style.add_color("value.array.border.color", color("#7197d5")?);
    style.add_number("value.array.border.width", 1.5);
    style.add_number("value.array.border.radius", 5.0);
    style.add_number("value.tuple.empty.w", 0.0);
    style.add_number("value.tuple.empty.h", 20.0);
    style.add_color("value.tuple.separator.color", color("#b785c080")?);
    style.add_number("value.tuple.separator.vmargin", 5.0);
    style.add_number("value.tuple.separator.padding.left", 10.0);
    style.add_number("value.tuple.separator.padding.top", 5.0);
    style.add_number("value.tuple.separator.padding.right", 10.0);
    style.add_number("value.tuple.separator.padding.bottom", 5.0);
    style.add_number("value.tuple.padding.left", 10.0);
    style.add_number("value.tuple.padding.top", 2.0);
    style.add_number("value.tuple.padding.right", 10.0);
    style.add_number("value.tuple.padding.bottom", 2.0);
    style.add_color("value.tuple.border.color", color("#b785c0")?);
    style.add_number("value.tuple.border.width", 1.5);
    style.add_number("value.tuple.border.radius", 5.0);
    style.add_number("value.tuple.border.radius.nw", 0.0);
    style.add_number("value.tuple.border.radius.se", 0.0);
    style.add_string("def.label.font", "mono");
    style.add_number("def.label.font_size", 23.0);
    style.add_color("def.label.color", color("#b2d9fd")?);
    style.add_string("def.separator.font", "mono");
    style.add_number("def.separator.font_size", 23.0);
    style.add_color("def.separator.color", color("#ccc")?);
    style.add_string("def.separator.text", ":");
    style.add_number("def.separator.padding.left", 3.0);
    style.add_number("def.separator.padding.right", 5.0);
    style.add_number("def.left.padding.bottom", 3.0);
    style.add_number("def.value.padding", 8.0);
    style.add_number("def.value.margin", 0.0);
    style.add_number("def.value.margin.top", 3.0);
    style.add_number("def.value.margin.bottom", 3.0);
    style.add_color("def.value.border.color", color("#282828")?);
    style.add_number("def.value.border.width", 1.5);
    style.add_number("def.value.border.radius", 5.0);
    style.add_string("value.struct.name.font", "mono");
    style.add_number("value.struct.name.font_size", 23.0);
    style.add_color("value.struct.name.color", color("#7fc8b0")?);
    style.add_string("value.struct.label.font", "mono");
    style.add_number("value.struct.label.font_size", 23.0);
    style.add_color("value.struct.label.color", color("#7fc8b0")?);
    style.add_string("value.struct.separator.font", "mono");
    style.add_number("value.struct.separator.font_size", 23.0);
    style.add_color("value.struct.separator.color", color("#ccc")?);
    style.add_string("value.struct.separator.text", ":");
    style.add_number("value.struct.separator.padding.left", 3.0);
    style.add_number("value.struct.separator.padding.right", 3.0);
    style.add_number("value.struct.padding", 10.0);
    style.add_number("value.struct.margin.left", 10.0);
    style.add_color("value.struct.border.color", color("#789a56")?);
    style.add_number("value.struct.border.width", 1.5);
    style.add_number("value.struct.border.radius", 5.0);
    style.add_number("value.struct.left.padding.bottom", 3.0);
    style.add_number("value.struct.divider.vmargin", 0.0);
    style.add_number("value.struct.divider.padding", 0.0);
    style.add_number("value.struct.divider.padding.left", 7.0);
    style.add_number("value.struct.divider.padding.right", 12.0);
    style.add_color("value.struct.divider.color", color("#789a5680")?);
    style.add_string("value.invalid.font", "mono");
    style.add_number("value.invalid.font_size", 48.0);
    style.add_color("value.invalid.color", color("#e44")?);
    style.add_number("value.invalid.padding", 0.0);
    style.add_number("value.invalid.padding.bottom", 10.0);
    style.add_string("region.header.font", "serif");
    style.add_number("region.header.font_size", 23.0);
    style.add_color("region.header.color", color("#ccc")?);
    style.add_number("region.header.padding", 0.0);
    style.add_number("region.header.padding.top", 10.0);
    style.add_number("region.header.padding.bottom", 10.0);
    style.add_number("region.padding", 0.0);
    style.add_number("location.region.gap", 25.0);
    style.add_string("location.header.font", "serif_bold");
    style.add_number("location.header.font_size", 28.0);
    style.add_color("location.header.color", color("#ccc")?);
    style.add_number("location.header.padding", 0.0);
    style.add_string("step.header.font", "serif_bold");
    style.add_number("step.header.font_size", 28.0);
    style.add_color("step.header.color", color("#dbdeab")?);
    style.add_number("step.header.padding", 3.0);
    style.add_number("step.header.padding.right", 20.0);
    style.add_color("step.separator.color", color("#404040")?);
    style.add_number("step.separator.size", 26.0);
    style.add_number("step.separator.padding", 5.0);
    style.add_number("step.separator.padding.right", 25.0);
    style.add_number("step.location.gap", 40.0);
    style.add_number("step.padding", 20.0);
    style.add_number("step.padding.left", 40.0);
    style.add_number("step.padding.right", 40.0);
    style.add_number("step.margin", 5.0);
    style.add_color("step.border.color", color("#404040")?);
    style.add_number("step.border.width", 1.5);
    style.add_number("step.border.radius", 5.0);
    style.add_number("program.step.gap", 5.0);
    Ok(style)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_styling() {
        let mut s = Styling::new();
        s.add_number("value.padding", 5.0);
        s.add_number("value.radius", 8.0);
        assert_eq!(s.get_number("value.padding"), Some(5.0));
        assert_eq!(s.get_number("value.radius"), Some(8.0));
        assert_eq!(s.get_number("value.margin"), None);
        assert_eq!(s.get_bool("value.padding"), None);
        assert_eq!(s.get_string("value.padding"), None);
        s.add_bool("value.clip.nw", true);
        assert_eq!(s.get_bool("value.clip.nw"), Some(true));
        assert_eq!(s.get_bool("value.clip.sw"), None);
        s.add_string("value.font", "mono");
        assert_eq!(s.get_string("value.font"), Some("mono".to_string()));
        assert_eq!(s.get_string("value.style"), None);
        s.add_color("value.color", ColorU8::from_rgba(255, 0, 128, 255));
        assert_eq!(
            s.get_color("value.color"),
            Some(ColorU8::from_rgba(255, 0, 128, 255))
        );
        assert_eq!(s.get_string("value.color"), None);
    }

    #[test]
    pub fn test_color() -> Result<()> {
        assert_eq!(color("#000")?, ColorU8::from_rgba(0, 0, 0, 0xff));
        assert_eq!(color("#f00")?, ColorU8::from_rgba(0xff, 0, 0, 0xff));
        assert_eq!(color("#080")?, ColorU8::from_rgba(0, 0x88, 0, 0xff));
        assert_eq!(color("#00a")?, ColorU8::from_rgba(0, 0, 0xaa, 0xff));
        assert_eq!(color("#0008")?, ColorU8::from_rgba(0, 0, 0, 0x88));
        assert_eq!(color("#1234")?, ColorU8::from_rgba(0x11, 0x22, 0x33, 0x44));
        assert_eq!(
            color("#123456")?,
            ColorU8::from_rgba(0x12, 0x34, 0x56, 0xff)
        );
        assert_eq!(
            color("#12345678")?,
            ColorU8::from_rgba(0x12, 0x34, 0x56, 0x78)
        );
        assert!(color("000").is_err());
        assert!(color("#12345").is_err());
        assert!(color("#0g3456").is_err());
        assert!(color("#fffffffff").is_err());
        Ok(())
    }
}
