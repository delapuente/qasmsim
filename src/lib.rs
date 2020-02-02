#[macro_use(lalrpop_mod)]
extern crate lalrpop_util;
extern crate cfg_if;
#[cfg(feature = "wasm")]
extern crate wasm_bindgen;
#[cfg(feature = "wasm")]
extern crate js_sys;
#[cfg(feature = "wasm")]
extern crate console_error_panic_hook;
extern crate num;
#[cfg_attr(test, macro_use(approx_eq))]
extern crate float_cmp;
#[macro_use(cached, cached_key)]
extern crate cached;
extern crate rand;
#[macro_use]
extern crate lazy_static;
extern crate regex;
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

use cfg_if::cfg_if;

use grammar::lexer::Lexer;
use linker::Linker;
use interpreter::computation::Computation;

fn do_run(input: &str) -> Result<Computation, String> {
  let linker = Linker::with_embedded(HashMap::from_iter(vec![
    ("qelib1.inc".to_owned(), qe::QELIB1.to_owned())
  ]));
  let lexer = Lexer::new(&input);
  let parser = open_qasm2::OpenQasmProgramParser::new();
  let program = parser.parse(lexer).unwrap();
  let linked = linker.link(program).unwrap();
  interpreter::runtime::execute(&linked)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run(input: &str) -> Result<Computation, String> {
  do_run(input)
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;
#[cfg(target_arch = "wasm32")]
use interpreter::computation::wasm::{ JsComputation, as_js_computation };

cfg_if! {
  if #[cfg(feature = "wee_alloc")] {
    extern crate wee_alloc;
    #[global_allocator]
    static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
  }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn run(input: &str) -> JsComputation {
  let result = do_run(input).unwrap();
  as_js_computation(result)
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn init() {
  use std::panic;
  panic::set_hook(Box::new(console_error_panic_hook::hook));
}