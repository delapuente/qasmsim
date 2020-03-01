
pub mod wasm {
  #![cfg(target_arch = "wasm32")]

  use serde_wasm_bindgen;
  use wasm_bindgen::prelude::{ wasm_bindgen, JsValue };
  use console_error_panic_hook;

  use crate::qasmsim::do_run;

  #[wasm_bindgen]
  pub fn run(input: &str) -> JsValue {
    serde_wasm_bindgen::to_value(&do_run(input).unwrap()).unwrap()
  }

  #[wasm_bindgen(start)]
  pub fn init() {
    use std::panic;
    panic::set_hook(Box::new(console_error_panic_hook::hook))
  }
}

pub mod native {
  #![cfg(not(target_arch = "wasm32"))]

  pub use crate::qasmsim::do_run as run;
}