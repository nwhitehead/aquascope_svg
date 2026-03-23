
use wasm_bindgen::prelude::*;

use kaya_lib::states::Program;

// use kaya_lib::render::render;

        // let program = parse(&contents)?;
        // let output = render(&program, args.show_heap)?;

//.map_err(|e| JsError::from(&*e))

#[wasm_bindgen]
pub fn parse(txt: String) -> Result<JsValue, JsError> {
    let prg = kaya_lib::parser::parse(&txt)?;
    Ok(serde_wasm_bindgen::to_value(&prg)?)
}

#[wasm_bindgen]
pub fn get_css() -> String {
    String::from_utf8(kaya_lib::render::CSS_STYLE.to_vec()).expect("Style must be UTF-8")
}

#[wasm_bindgen]
pub fn render(prg: JsValue, show_heap: bool) -> Result<String, JsError> {
    let program: Program = serde_wasm_bindgen::from_value(prg)?;
    kaya_lib::render::render(&program, show_heap).map_err(|e| JsError::from(&*e))
}

#[wasm_bindgen]
pub fn render_parts(prg: JsValue, show_heap: bool) -> Result<Vec<String>, JsError> {
    let program: Program = serde_wasm_bindgen::from_value(prg)?;
    let (prg_txt, arrow_txt) = kaya_lib::render::render_parts(&program, show_heap).map_err(|e| JsError::from(&*e))?;
    Ok(vec![prg_txt, arrow_txt])
    //kaya_lib::render::render_parts(&program, show_heap).map_err(|e| JsError::from(&*e))
}
