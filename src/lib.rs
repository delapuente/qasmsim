use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub open_qasm2, "/grammar/open_qasm2.rs");

mod grammar;
mod linker;
mod semantics;
pub mod complex;
pub mod statevector;
mod interpreter;
mod qe;

use std::collections::HashMap;
use std::iter::FromIterator;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;
#[cfg(target_arch = "wasm32")]
use console_error_panic_hook;

use crate::grammar::lexer::Lexer;
use crate::linker::Linker;
use crate::interpreter::computation::Computation;

pub fn run(input: &str) -> Result<Computation, String> {
  let linker = Linker::with_embedded(HashMap::from_iter(vec![
    ("qelib1.inc".to_owned(), qe::QELIB1.to_owned())
  ]));
  let lexer = Lexer::new(&input);
  let parser = open_qasm2::OpenQasmProgramParser::new();
  let program = parser.parse(lexer).unwrap();
  let linked = linker.link(program).unwrap();
  interpreter::runtime::execute(&linked)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn js_run(input: &str) -> Vec<f64> {
  let computation = run(input).unwrap();
  computation.statevector.bases.iter().map(|c| vec![c.re, c.im]).flatten().collect()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn init() {
  use std::panic;
  panic::set_hook(Box::new(console_error_panic_hook::hook))
}