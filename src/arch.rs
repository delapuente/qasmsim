
pub mod wasm {
  #![cfg(target_arch = "wasm32")]

  use std::convert::From;
  use std::iter::IntoIterator;
  use std::collections::HashMap;

  use js_sys::{ self, Float64Array, Object, Map };
  use web_sys;
  use wasm_bindgen::prelude::{ wasm_bindgen, JsValue };
  use console_error_panic_hook;

  use crate::QasmSimError;
  use crate::api;
  use crate::interpreter::Computation;
  use crate::statevector::StateVector;

  macro_rules! measure {
    ($measure_name:expr, $block:block) => {
      {
        use web_sys;
        let window = web_sys::window().expect("get `window`");
        let performance = window.performance().expect("get `window.performance`");
        performance.clear_measures();
        performance.clear_marks();

        let start_mark = format!("{}_start", $measure_name);
        let end_mark = format!("{}_end", $measure_name);

        performance.mark(&start_mark).expect("set start mark");
        let result = $block;
        performance.mark(&end_mark).expect("set end mark");

        performance.measure_with_start_mark_and_end_mark(
          &$measure_name, &start_mark, &end_mark).expect("set the measure");
        web_sys::console::log(&performance.get_entries_by_type(&"measure"));
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
      ).expect("set `statevector`");
      js_sys::Reflect::set(&out,
        &"probabilities".into(),
        &as_typed_array(computation.probabilities).into()
      ).expect("set `probabilities`");
      js_sys::Reflect::set(&out,
        &"memory".into(),
        &as_map(computation.memory).into()
      ).expect("set `memory`");
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
      ).expect("set `bases`");
      js_sys::Reflect::set(&out,
        &"bitWidth".into(),
        &(statevector.bit_width as i32).into()
      ).expect("set `bitWidth`");
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

  impl From<QasmSimError<'_>> for JsValue {
    fn from(value: QasmSimError) -> Self {
      let message = format!("{}", &value);
      let obj = Object::new();
      js_sys::Reflect::set(&obj,
        &"message".into(),
        &JsValue::from_str(&message)
      ).expect("set `message`");
      js_sys::Reflect::set(&obj,
        &"toString".into(),
        &js_sys::Function::new_no_args("return this.message").into()
      ).expect("set `toString`");

      match value {
        QasmSimError::UnknownError (_) => {
          js_sys::Reflect::set(&obj,
            &"type".into(),
            &JsValue::from_str("Unknown")
          ).expect("set `type`");
        },
        QasmSimError::SyntaxError {
          kind,
          lineoffset,
          lineno,
          startpos,
          endpos,
          token,
          ..
        } => {
          js_sys::Reflect::set(&obj,
            &"type".into(),
            &JsValue::from_str(&format!("{:?}", kind))
          ).expect("set `type`");
          js_sys::Reflect::set(&obj,
            &"lineOffset".into(),
            &JsValue::from_f64(lineoffset as f64)
          ).expect("set `lineOffset`");
          js_sys::Reflect::set(&obj,
            &"lineNumber".into(),
            &JsValue::from_f64(lineno as f64)
          ).expect("set `lineNumber`");
          js_sys::Reflect::set(&obj,
            &"startPosition".into(),
            &JsValue::from_f64(startpos as f64)
          ).expect("set `startPosition`");
          if let Some(endpos) = endpos {
            js_sys::Reflect::set(&obj,
              &"endPosition".into(),
              &JsValue::from_f64(endpos as f64)
            ).expect("set `endPosition`");
          }
          if let Some(token) = token {
            js_sys::Reflect::set(&obj,
              &"token".into(),
              &JsValue::from_str(&format!("{}", token))
            ).expect("set `token`");
          }
        }
      };
      obj.into()
    }
  }

  #[wasm_bindgen]
  pub fn run(input: &str) -> Result<JsValue, JsValue> {
    let linked = measure!("parsing", {
      api::compile_with_linker(input, api::default_linker())
    })?;
    let computation: Computation = measure!("computation", {
      api::execute(&linked)
    })?;
    let out = measure!("serialization", {
      computation.into()
    });
    Ok(out)
  }

  #[wasm_bindgen(start)]
  pub fn init() {
    use std::panic;
    panic::set_hook(Box::new(console_error_panic_hook::hook))
  }
}

pub mod native {
  #![cfg(not(target_arch = "wasm32"))]

  use crate::api;
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

  pub fn run(input: &str) -> api::Result<Computation> {
    let linked = measure!("parsing", {
      api::compile_with_linker(input, api::default_linker())?
    });
    let out = measure!("computation", {
      api::execute(&linked)
    });
    out
  }
}