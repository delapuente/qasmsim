#![cfg(not(target_arch = "wasm32"))]

use std::convert;
use std::collections::HashMap;

use crate::{
  api,
  statevector::StateVector
};

use crate::interpreter::{ Computation, Histogram };

pub use api::compile_with_linker;
pub use api::execute;
pub use api::execute_with_shots;
pub use api::default_linker;


macro_rules! measure {
  ($block:expr) => {
    {
      use std::time::Instant;
      let measurement = Instant::now();
      let result = $block;
      let elapsed = measurement.elapsed().as_millis();
      (result, elapsed)
    }
  };
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunTimes {
  pub parsing_time: u128,
  pub simulation_time: u128
}

#[derive(Debug, Clone, PartialEq)]
pub struct Run {
  pub statevector: StateVector,
  pub probabilities: Vec<f64>,
  pub memory: HashMap<String, u64>,
  pub histogram: Option<Histogram>,
  pub times: RunTimes
}

impl convert::From<(Computation, u128, u128)> for Run {

  fn from(value: (Computation, u128, u128)) -> Self {
    let (computation, parsing_time, simulation_time) = value;
    Run {
      statevector: computation.statevector,
      probabilities: computation.probabilities,
      memory: computation.memory,
      histogram: computation.histogram,
      times: RunTimes {
        parsing_time,
        simulation_time
      }
    }
  }

}

pub fn run(input: &str, shots: Option<usize>) -> api::Result<Run> {
  let (linked, parsing_time) = measure!({
    compile_with_linker(input, api::default_linker())
  });
  let (out, simulation_time) = measure!({
    match shots {
      None => execute(&linked?),
      Some(shots) => execute_with_shots(&linked?, shots)
    }
  });
  Ok(Run::from((out?, parsing_time, simulation_time)))
}