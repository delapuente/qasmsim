pub mod wasm {
  #![cfg(target_arch = "wasm32")]

  use std::collections::HashMap;
  use std::convert::From;

  use serde_derive::Serialize;
  use cfg_if::cfg_if;
  use wasm_bindgen::prelude::{ wasm_bindgen, JsValue };

  use crate::api::do_run;
  use crate::statevector::StateVector;
  use crate::interpreter::computation::Computation;

  cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
      extern crate wee_alloc;
      #[global_allocator]
      static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
  }

  #[derive(Serialize)]
  pub struct JsComputation {
    pub memory: HashMap<String, u64>,
    pub statevector: StateVector,
    pub probabilities: Vec<f64>
  }

  impl From<Computation> for JsComputation {
    fn from(computation: Computation) -> Self {
      JsComputation{
        memory: computation.memory,
        statevector: computation.statevector,
        probabilities: computation.probabilities
      }
    }
  }

  #[wasm_bindgen]
  pub fn run(input: &str) -> JsValue {
    let result = do_run(input).unwrap();
    JsValue::from_serde(&JsComputation::from(result)).unwrap()
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