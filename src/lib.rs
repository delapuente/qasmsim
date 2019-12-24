mod grammar;
mod complex;
mod statevector;
mod gatelib;
mod simulator;

use std::error::Error;

use statevector::StateVector;
use grammar::open_qasm2::open_qasm2;

pub fn run(input: &str) -> Result<StateVector, Box<dyn Error>> {
  let parser = open_qasm2::OpenQasmProgramParser::new();
  let program = parser.parse(&input).unwrap();
  simulator::runtime::execute(&program)
}