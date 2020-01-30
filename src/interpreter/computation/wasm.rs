#![cfg(target_arch = "wasm32")]

use js_sys;
use wasm_bindgen;
use wasm_bindgen::prelude::*;

use statevector::StateVector;
use interpreter::computation::Computation;

#[wasm_bindgen]
pub struct JsComputation {
  memory: Vec<(String, f64)>,
  statevector: Vec<f64>
}

#[wasm_bindgen]
impl JsComputation {
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

pub fn as_js_computation(computation: Computation) -> JsComputation {
  JsComputation{
    memory: computation.memory.iter().map(|(k, v)| (k.to_owned(), *v as f64)).collect(),
    statevector: as_float_array(&computation.statevector)
  }
}

fn as_float_array(statevector: &StateVector) -> Vec<f64> {
  statevector.bases.iter().flat_map(|a| vec!(a.re, a.im)).collect()
}