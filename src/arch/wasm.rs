#![cfg(target_arch = "wasm32")]

mod computation;
mod error;

use web_sys;
use wasm_bindgen::prelude::{ wasm_bindgen, JsValue };
use console_error_panic_hook;

use crate::api;
use crate::interpreter::Computation;

macro_rules! measure {
  ($measure_name:expr, $block:block) => {
    {
      use web_sys;
      let window = web_sys::window().expect("get `window`");
      let performance = window.performance().expect("get `window.performance`");
      performance.clear_measures();
      performance.clear_marks();

      let start_mark = format!("{}_start", $measure_name);
      let end_mark = format!("{}_end", $measure_name);

      performance.mark(&start_mark).expect("set start mark");
      let result = $block;
      performance.mark(&end_mark).expect("set end mark");

      performance.measure_with_start_mark_and_end_mark(
        &$measure_name, &start_mark, &end_mark).expect("set the measure");
      web_sys::console::log(&performance.get_entries_by_type(&"measure"));
      result
    }
  };
}

#[wasm_bindgen]
pub fn run(input: &str) -> Result<JsValue, JsValue> {
  let linked = measure!("parsing", {
    api::compile_with_linker(input, api::default_linker())
  })?;
  let computation: Computation = measure!("computation", {
    api::execute(&linked)
  })?;
  let out = measure!("serialization", {
    computation.into()
  });
  Ok(out)
}

#[wasm_bindgen(start)]
pub fn init() {
  use std::panic;
  panic::set_hook(Box::new(console_error_panic_hook::hook))
}
