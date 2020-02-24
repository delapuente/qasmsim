use std::collections::HashMap;

use crate::statevector::StateVector;

#[derive(Debug)]
pub struct Computation {
  pub statevector: StateVector,
  pub memory: HashMap<String, u64>,
  pub probabilities: Vec<f64>
}

impl Computation {
  pub fn new(memory: HashMap<String, u64>, statevector: StateVector) -> Self {
    Computation {
      probabilities: statevector.probabilities(),
      statevector,
      memory
    }
  }
}
