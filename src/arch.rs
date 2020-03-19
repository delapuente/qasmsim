
pub mod wasm {
  #![cfg(target_arch = "wasm32")]

  use std::convert::From;
  use std::collections::HashMap;

  use web_sys;
  use serde::Serialize;
  use serde_wasm_bindgen;
  use wasm_bindgen::prelude::{ wasm_bindgen, JsValue };
  use console_error_panic_hook;

  use crate::qasmsim::do_run;
  use crate::interpreter;
  use crate::statevector;
  use crate::complex::Complex;

  #[derive(Serialize)]
  pub struct StateVector {
    pub bases: Vec<Complex>,
    pub bit_width: usize
  }

  impl From<statevector::StateVector> for StateVector {
    fn from(value: statevector::StateVector) -> Self {
      StateVector {
        bases: value.bases,
        bit_width: value.bit_width
      }
    }
  }

  #[derive(Serialize)]
  pub struct Computation {
    pub statevector: StateVector,
    pub memory: HashMap<String, u64>,
    pub probabilities: Vec<f64>
  }

  impl From<interpreter::Computation> for Computation {
    fn from(value: interpreter::Computation) -> Self {
      Computation {
        statevector: value.statevector.into(),
        memory: value.memory,
        probabilities: value.probabilities
      }
    }
  }

  macro_rules! measure {
    ($measure_name:expr, $block:block) => {
      {
        use web_sys;
        let window = web_sys::window().expect("should have a window");
        let performance = window.performance().expect("performance should be available");

        let start_mark = format!("{}_start", $measure_name);
        let end_mark = format!("{}_end", $measure_name);

        performance.mark(&start_mark);
        let result = $block;
        performance.mark(&end_mark);

        performance.measure_with_start_mark_and_end_mark(
          &$measure_name, &start_mark, &end_mark);
        web_sys::console::log(&performance.get_entries_by_type(&"measure"));
        performance.clear_measures();
        performance.clear_marks();
        result
      }
    };
  }

  #[wasm_bindgen]
  pub fn run(input: &str) -> JsValue {
    let computation: interpreter::Computation = do_run(input).unwrap();
    let out = measure!("serialization", {
      serde_wasm_bindgen::to_value(&Computation::from(computation)).unwrap()
    });
    out
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