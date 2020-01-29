#[cfg(feature = "wasm")]
use js_sys;
#[cfg(feature = "wasm")]
use wasm_bindgen;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Computation {
  memory: Vec<(String, f64)>,
  statevector: Vec<f64>
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Computation {
  pub fn get_memory(&self) -> js_sys::Array {
    let a = js_sys::Array::new();
    for (name, value) in &self.memory {
      let pair = js_sys::Array::new();
      pair.push(&JsValue::from_str(name));
      pair.push(&JsValue::from_f64(*value));
      a.push(&pair);
    }
    a
  }

  pub fn get_statevector(&self) -> Vec<f64> {
    self.statevector.iter().cloned().collect()
  }
}

#[cfg(target_arch = "wasm32")]
pub fn new_computation(memory: Vec<(String, f64)>, statevector: Vec<f64>)
-> Computation {
  Computation{ memory, statevector }
}