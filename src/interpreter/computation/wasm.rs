#![cfg(target_arch = "wasm32")]

use js_sys;
use wasm_bindgen;
use wasm_bindgen::prelude::*;

use statevector::StateVector;
use interpreter::computation::Computation;

#[wasm_bindgen]
pub struct JsComputation {
  memory: Vec<(String, f64)>,
  statevector: Vec<f64>,
  probabilities: Vec<f64>
}

#[wasm_bindgen]
impl JsComputation {
  pub fn get_memory(&self) -> js_sys::Array {
    use std::iter::FromIterator;
    js_sys::Array::from_iter(
      self.memory.iter()
      .map(|(k, v)| js_sys::Array::of2(
        &JsValue::from_str(k),
        &JsValue::from_f64(*v)
      ))
    )
  }

  pub fn get_statevector(&self) -> Vec<f64> {
    self.statevector.iter().cloned().collect()
  }

  pub fn probabilities(&self) -> Vec<f64> {
    self.probabilities.iter().cloned().collect()
  }
}

pub fn as_js_computation(computation: Computation) -> JsComputation {
  JsComputation{
    memory: computation.memory.iter().map(|(k, v)| (k.to_owned(), *v as f64)).collect(),
    statevector: as_float_array(&computation.statevector),
    probabilities: computation.probabilities()
  }
}

fn as_float_array(statevector: &StateVector) -> Vec<f64> {
  statevector.bases.iter().flat_map(|a| vec![a.re, a.im]).collect()
}