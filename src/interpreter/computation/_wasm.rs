#![cfg(target_arch = "wasm32")]

use js_sys;
use wasm_bindgen;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Computation {
  memory: Vec<(String, f64)>,
  statevector: Vec<f64>
}

#[wasm_bindgen]
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

pub fn new_computation(memory: Vec<(String, f64)>, statevector: Vec<f64>)
-> Computation {
  Computation{ memory, statevector }
}