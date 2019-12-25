#[macro_use(lalrpop_mod)]
extern crate lalrpop_util;
extern crate num;
extern crate float_cmp;
lalrpop_mod!(pub open_qasm2, "/grammar/open_qasm2.rs");

mod grammar;
mod complex;
mod statevector;
mod gatelib;
mod simulator;

use std::error::Error;

use statevector::StateVector;

pub fn run(input: &str) -> Result<StateVector, Box<dyn Error>> {
  let parser = open_qasm2::OpenQasmProgramParser::new();
  let program = parser.parse(&input).unwrap();
  simulator::runtime::execute(&program)
}