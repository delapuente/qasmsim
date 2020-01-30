pub mod wasm;

use std::collections::HashMap;

use statevector::StateVector;

#[derive(Debug)]
pub struct Computation {
  pub statevector: StateVector,
  pub memory: HashMap<String, u64>
}

impl Computation {
  pub fn new(memory: HashMap<String, u64>, statevector: StateVector) -> Self {
    Computation{ statevector, memory }
  }
}