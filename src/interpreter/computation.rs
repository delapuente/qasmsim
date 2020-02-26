use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use serde::Serialize;

use crate::statevector::StateVector;

#[derive(Debug)]
#[cfg_attr(target_arch = "wasm32", derive(Serialize))]
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
