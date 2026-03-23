
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
pub fn render(prg: JsValue, show_heap: bool) -> Result<String, JsError> {
    let program: Program = serde_wasm_bindgen::from_value(prg)?;
    kaya_lib::render::render(&program, show_heap).map_err(|e| JsError::from(&*e))
}
