pub mod wasm;

use std::collections::HashMap;

use crate::statevector::StateVector;

#[derive(Debug)]
pub struct Computation {
  statevector: StateVector,
  memory: HashMap<String, u64>
}

impl Computation {
  pub fn new(memory: HashMap<String, u64>, statevector: StateVector) -> Self {
    Computation{ statevector, memory }
  }

  pub fn statevector(&self) -> &StateVector {
    &self.statevector
  }

  pub fn memory(&self) -> HashMap<String, u64> {
    self.memory.clone()
  }

  pub fn probabilities(&self) -> Vec<f64> {
    self.statevector.probabilities()
  }
}