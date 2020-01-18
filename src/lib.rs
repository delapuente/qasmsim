#[macro_use(lalrpop_mod)]
extern crate lalrpop_util;
extern crate cfg_if;
#[cfg(feature = "wasm")]
extern crate wasm_bindgen;
extern crate num;
extern crate float_cmp;
#[macro_use(cached, cached_key)]
extern crate cached;
lalrpop_mod!(pub open_qasm2, "/grammar/open_qasm2.rs");

mod grammar;
mod semantics;
pub mod complex;
pub mod statevector;
mod interpreter;

use cfg_if::cfg_if;

use statevector::StateVector;

fn do_run(input: &str) -> Result<StateVector, String> {
  let parser = open_qasm2::OpenQasmProgramParser::new();
  let program = parser.parse(&input).unwrap();
  interpreter::runtime::execute(&program)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run(input: &str) -> Result<StateVector, String> {
  do_run(input)
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

cfg_if! {
  if #[cfg(feature = "wee_alloc")] {
    extern crate wee_alloc;
    #[global_allocator]
    static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
  }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn run(input: &str) -> Vec<f64> {
  use statevector::wasm::as_float_array;
  let result = do_run(input);
  as_float_array(&result.unwrap())
}