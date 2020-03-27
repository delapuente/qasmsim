#![cfg(target_arch = "wasm32")]

#[macro_use]
mod macros;
mod computation;
mod error;

use web_sys;
use wasm_bindgen::prelude::{ wasm_bindgen, JsValue };
use console_error_panic_hook;

use crate::api;

#[wasm_bindgen]
pub fn run(input: &str) -> Result<JsValue, JsValue> {
  let (linked, parsing_time) = measure!("parsing", {
    api::compile_with_linker(input, api::default_linker())
  });
  let (computation, simulation_time) = measure!("simulation", {
    api::execute(&linked?)
  });
  let (out, serialization_time) = measure!("serialization", {
    computation?.into()
  });
  set!(&out,
    "parsing" => parsing_time
  );
  let times = js_sys::Object::new();
  set!(&times,
    "parsing" => parsing_time,
    "simulation" => simulation_time,
    "serialization" => serialization_time
  );
  set!(&out, "times" => times);
  Ok(out)
}

#[wasm_bindgen(start)]
pub fn init() {
  use std::panic;
  panic::set_hook(Box::new(console_error_panic_hook::hook))
}
