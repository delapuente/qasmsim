
pub mod wasm {
  #![cfg(target_arch = "wasm32")]
  use std::convert::From;
  use std::collections::HashMap;

  use serde::Serialize;
  use serde_wasm_bindgen;
  use wasm_bindgen::prelude::{ wasm_bindgen, JsValue };
  use console_error_panic_hook;

  use crate::qasmsim::do_run;
  use crate::interpreter;
  use crate::statevector::StateVector;

  #[derive(Serialize)]
  pub struct Computation {
    pub statevector: StateVector,
    pub memory: HashMap<String, u64>,
    pub probabilities: Vec<f64>
  }

  impl From<interpreter::Computation> for Computation {
    fn from(value: interpreter::Computation) -> Self {
      Computation {
        statevector: value.statevector,
        memory: value.memory,
        probabilities: value.probabilities
      }
    }
  }

  #[wasm_bindgen]
  pub fn run(input: &str) -> JsValue {
    let computation: Computation = do_run(input).unwrap().into();
    serde_wasm_bindgen::to_value(&computation).unwrap()
  }

  #[wasm_bindgen(start)]
  pub fn init() {
    use std::panic;
    panic::set_hook(Box::new(console_error_panic_hook::hook))
  }
}

pub mod native {
  #![cfg(not(target_arch = "wasm32"))]

  pub use crate::qasmsim::do_run as run;
}