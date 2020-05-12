#![cfg(target_arch = "wasm32")]

#[macro_use]
mod macros;
mod computation;
mod error;

use console_error_panic_hook;
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};
use web_sys;

use crate::api;
use crate::error::QasmSimError;

/// Parse and simulate the input OPENQASM program with optional shots.
///
/// # Example
///
/// ```js
/// var result = qasmsim.run(`
/// OPENQASM 2.0;
/// include "qelib1.inc";
/// qreg q[2];
/// h q[0];
/// cx q[0], q[1];
/// `);
/// ```
#[wasm_bindgen]
pub fn run(input: &str, shots: Option<usize>) -> Result<JsValue, JsValue> {
    let (linked, parsing_time) = measure!("parsing", {
        api::compile_with_linker(input, api::default_linker())
    });
    let (computation, simulation_time) = measure!("simulation", {
        match shots {
            None => api::simulate(&linked?),
            Some(shots) => api::simulate_with_shots(&linked?, shots),
        }
    });
    let (out, serialization_time) = measure!("serialization", {
        computation
            .map_err(|err| QasmSimError::from((input, err)))?
            .into()
    });
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
