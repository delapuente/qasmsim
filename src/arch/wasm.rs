#![cfg(target_arch = "wasm32")]
#![allow(missing_docs)]

#[macro_use]
mod macros;
mod computation;
mod error;

use console_error_panic_hook;
use serde_wasm_bindgen;
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};
use web_sys;

use crate::api;
use crate::error::QasmSimError;
use crate::grammar;

macro_rules! adapt_parse_functions {
    ($($(#[$attr:meta])* $vis:vis fn $funcname:ident ($param:ident) => $parsefunc:path;)*) => {
        $(
            #[wasm_bindgen]
            #[allow(non_snake_case)]
            $(#[$attr])* $vis fn $funcname(
                $param: &str
            ) -> Result<JsValue, JsValue> {
                $parsefunc(source)
                    .map(|v| serde_wasm_bindgen::to_value(&v).unwrap())
                    .map_err(|err| err.into())
            }
        )*
    };
}

#[wasm_bindgen]
pub fn run(input: &str, shots: Option<usize>) -> Result<JsValue, JsValue> {
    let (linked, parsing_time) = measure!("parsing", { api::parse_and_link(input) });
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

adapt_parse_functions! {
    pub fn parseProgram(source) => grammar::parse_program;
    pub fn parseLibrary(source) => grammar::parse_library;
    pub fn parseExpression(source) => grammar::parse_expression;
    pub fn parseProgramBody(source) => grammar::parse_program_body;
    pub fn parseStatement(source) => grammar::parse_statement;
}

#[wasm_bindgen(start)]
pub fn init() {
    use std::panic;
    panic::set_hook(Box::new(console_error_panic_hook::hook))
}
