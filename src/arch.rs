pub mod wasm {
  #![cfg(target_arch = "wasm32")]

  use std::collections::HashMap;
  use std::convert::From;

  use serde::Serialize;
  use wasm_bindgen::prelude::{ wasm_bindgen, JsValue };
  use serde_wasm_bindgen;

  use crate::api::do_run;
  use crate::statevector::StateVector;
  use crate::interpreter::computation::Computation;
  use crate::complex::Complex;

  #[global_allocator]
  static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

  #[derive(Serialize)]
  pub struct JsStateVector {
    pub bases: Vec<Complex>,
    pub bit_width: usize
  }

  impl From<StateVector> for JsStateVector {
    fn from(statevector: StateVector) -> Self {
      JsStateVector {
        bases: statevector.bases,
        bit_width: statevector.bit_width
      }
    }
  }

  #[derive(Serialize)]
  pub struct JsComputation {
    pub memory: HashMap<String, u64>,
    pub statevector: JsStateVector,
    pub probabilities: Vec<f64>
  }

  impl From<Computation> for JsComputation {
    fn from(computation: Computation) -> Self {
      JsComputation {
        memory: computation.memory,
        statevector: JsStateVector::from(computation.statevector),
        probabilities: computation.probabilities
      }
    }
  }

  #[wasm_bindgen]
  pub fn run(input: &str) -> JsValue {
    let result = do_run(input).unwrap();
    serde_wasm_bindgen::to_value(&JsComputation::from(result)).unwrap()
  }

  #[wasm_bindgen(start)]
  pub fn init() {
    use std::panic;
    panic::set_hook(Box::new(console_error_panic_hook::hook));
  }
}

pub mod native {
  #![cfg(not(target_arch = "wasm32"))]

  pub use crate::api::do_run as run;
}