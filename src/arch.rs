
pub mod wasm {
  #![cfg(target_arch = "wasm32")]

  use std::convert::From;
  use std::iter::IntoIterator;
  use std::collections::HashMap;

  use js_sys::{ self, Float64Array, Object, Map };
  use web_sys;
  use wasm_bindgen::prelude::{ wasm_bindgen, JsValue };
  use console_error_panic_hook;

  use crate::qasmsim;
  use crate::interpreter::Computation;
  use crate::statevector::StateVector;

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

  impl From<Computation> for JsValue {
    fn from(computation: Computation) -> Self {
      let out = Object::new();
      js_sys::Reflect::set(&out,
        &"statevector".into(),
        &computation.statevector.into()
      );
      js_sys::Reflect::set(&out,
        &"probabilities".into(),
        &as_typed_array(computation.probabilities).into()
      );
      js_sys::Reflect::set(&out,
        &"memory".into(),
        &as_map(computation.memory).into()
      );
      out.into()
    }
  }

  impl From<StateVector> for JsValue {
    fn from(statevector: StateVector) -> Self {
      let bases = statevector.bases;
      let flatten_amplitudes: Vec<f64> = bases.iter().flat_map(|c| vec![c.re, c.im]).collect();
      let out = Object::new();
      js_sys::Reflect::set(&out,
        &"bases".into(),
        &as_typed_array(flatten_amplitudes).into()
      );
      js_sys::Reflect::set(&out,
        &"bitWidth".into(),
        &(statevector.bit_width as i32).into()
      );
      out.into()
    }
  }

  fn as_typed_array<I>(iterator: I) -> Float64Array
  where I: IntoIterator<Item=f64> {
    let values: Vec<f64> = iterator.into_iter().collect();
    Float64Array::from(&values[..])
  }

  fn as_map(hashmap: HashMap<String, u64>) -> Map {
    let map = Map::new();
    for (key, value) in hashmap {
      map.set(&key.into(), &(value as f64).into());
    }
    map
  }

  #[wasm_bindgen]
  pub fn run(input: &str) -> JsValue {
    let linked = measure!("parsing", {
      qasmsim::compile_with_linker(input, qasmsim::default_linker()).unwrap()
    });
    let computation: Computation = measure!("computation", {
      qasmsim::execute(&linked).unwrap()
    });
    let out = measure!("serialization", {
      computation.into()
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

  use crate::qasmsim;
  use crate::interpreter::Computation;

  macro_rules! measure {
    ($measure_name:expr, $block:block) => {
      {
        use std::time::Instant;
        let measurement = Instant::now();
        let result = $block;
        println!("{}: {:.2}ms", $measure_name, measurement.elapsed().as_millis());
        result
      }
    };
  }

  pub fn run(input: &str) -> Result<Computation, String> {
    let linked = measure!("parsing", {
      qasmsim::compile_with_linker(input, qasmsim::default_linker())?
    });
    let out = measure!("computation", {
      qasmsim::execute(&linked)
    });
    out
  }
}