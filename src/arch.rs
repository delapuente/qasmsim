
pub mod wasm {
  #![cfg(target_arch = "wasm32")]

  use std::convert::From;
  use std::iter::IntoIterator;
  use std::collections::HashMap;

  use js_sys::{ self, Float64Array };
  use serde::Serialize;
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
    let computation: Computation = do_run(input).unwrap().into();
    let out = js_sys::Object::new();
    js_sys::Reflect::set(&out,
      &"statevector".into(),
      &sv_into_f64array(computation.statevector).into()
    );
    js_sys::Reflect::set(&out,
      &"probabilities".into(),
      &into_f64array(computation.probabilities).into()
    );
    out.into()
  }

  fn sv_into_f64array(statevector: StateVector) -> Float64Array {
    let bases = statevector.bases;
    let flatten_amplitudes = bases.iter().flat_map(|c| vec![c.re, c.im]);
    into_f64array(flatten_amplitudes)
  }

  fn into_f64array<'a, I>(iterator: I) -> Float64Array
  where I: IntoIterator<Item=f64> {
    let values: Vec<f64> = iterator.into_iter().collect();
    Float64Array::from(&values[..])
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