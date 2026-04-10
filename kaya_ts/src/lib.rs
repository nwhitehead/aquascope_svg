use pest::error::LineColLocation;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use kaya_lib::states::Program;

#[derive(Serialize)]
pub enum ParseResult {
    Success(Program),
    Error(String, Vec<usize>),
}

#[wasm_bindgen]
pub fn parse(txt: String) -> Result<JsValue, JsError> {
    match kaya_lib::parser::parse(&txt) {
        Ok(prg) => {
            let res = ParseResult::Success(prg);
            Ok(serde_wasm_bindgen::to_value(&res)?)
        }
        Err(err) => {
            let pos = match err.line_col {
                LineColLocation::Pos((u0, u1)) => vec![u0, u1],
                LineColLocation::Span((u0, u1), (u2, u3)) => vec![u0, u1, u2, u3],
            };
            let res = ParseResult::Error(err.to_string(), pos);
            Ok(serde_wasm_bindgen::to_value(&res)?)
        }
    }
}

#[wasm_bindgen]
pub fn parse_partial(txt: String) -> Result<JsValue, JsError> {
    match kaya_lib::parser::parse_partial(&txt) {
        Ok(prg) => {
            let res = ParseResult::Success(prg);
            Ok(serde_wasm_bindgen::to_value(&res)?)
        }
        Err(err) => {
            let pos = match err.line_col {
                LineColLocation::Pos((u0, u1)) => vec![u0, u1],
                LineColLocation::Span((u0, u1), (u2, u3)) => vec![u0, u1, u2, u3],
            };
            let res = ParseResult::Error(err.to_string(), pos);
            Ok(serde_wasm_bindgen::to_value(&res)?)
        }
    }
}

#[wasm_bindgen]
pub fn draw_program_png(prg: JsValue, scale: f32, theme: String) -> Result<Vec<u8>, JsError> {
    let program: Program = serde_wasm_bindgen::from_value(prg)?;
    kaya_lib::render::draw_program_png(&program, scale, &theme).map_err(|e| JsError::from(&*e))
}
