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

// #[wasm_bindgen]
// pub fn render(prg: JsValue, show_heap: bool) -> Result<String, JsError> {
//     let program: Program = serde_wasm_bindgen::from_value(prg)?;
//     kaya_lib::render::render(&program, show_heap).map_err(|e| JsError::from(&*e))
// }

// #[wasm_bindgen]
// pub fn render_parts(prg: JsValue, show_heap: bool) -> Result<Vec<String>, JsError> {
//     let program: Program = serde_wasm_bindgen::from_value(prg)?;
//     let (prg_txt, arrow_txt) =
//         kaya_lib::render::render_parts(&program, show_heap).map_err(|e| JsError::from(&*e))?;
//     Ok(vec![prg_txt, arrow_txt])
// }

// #[wasm_bindgen]
// pub fn render_program(prg: JsValue, hide_heap: bool) -> Result<JsValue, JsError> {
//     let program: Program = serde_wasm_bindgen::from_value(prg)?;
//     let result =
//         kaya_lib::render::render_program(&program, hide_heap).map_err(|e| JsError::from(&*e))?;
//     Ok(serde_wasm_bindgen::to_value(&result)?)
// }

// #[wasm_bindgen]
// pub fn arrow_options(info: JsValue, idx: usize) -> Result<JsValue, JsError> {
//     let info: ArrowInfo = serde_wasm_bindgen::from_value(info)?;
//     Ok(serde_wasm_bindgen::to_value(
//         &kaya_lib::render::arrow_options(&info, idx),
//     )?)
// }
