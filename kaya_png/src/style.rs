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
}
