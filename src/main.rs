mod grammar;
mod complex;
mod statevector;
mod gatelib;
mod simulator;

use grammar::open_qasm2::open_qasm2;
use std::io::{self, Read};
use std::time::Instant;

fn main() -> io::Result<()> {
  let start = Instant::now();
  let mut input = String::new();
  io::stdin().read_to_string(&mut input)?;
  let parser = open_qasm2::OpenQasmProgramParser::new();
  let tree = parser.parse(&input).unwrap();
  let state_vector = simulator::runtime::execute(&tree);
  print!("Calculated in {:?}s:\n{:?}", start.elapsed().as_secs_f32(), state_vector);
  Ok(())
}
