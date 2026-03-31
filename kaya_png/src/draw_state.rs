use tiny_skia::*;

#[derive(Clone, Debug)]
pub struct DrawState {
    pub font: String,
    pub font_size: f32,
    pub font_max_width: f32,
    pub text_color: ColorU8,
    pub stroke_color: ColorU8,
    pub stroke: Stroke,
    pub border_radius: (f32, f32, f32, f32),
    pub padding: (f32, f32, f32, f32),
    pub margin: (f32, f32, f32, f32),
    pub border_clip: (bool, bool, bool, bool),
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
            padding: (0.0, 0.0, 0.0, 0.0),
            margin: (0.0, 0.0, 0.0, 0.0),
        }
    }
}
