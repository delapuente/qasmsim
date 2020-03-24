#![cfg(target_arch = "wasm32")]

use std::convert::From;
use std::iter::IntoIterator;
use std::collections::HashMap;

use wasm_bindgen::prelude::JsValue;
use js_sys::{ self, Float64Array, Object, Map };

use crate::interpreter::Computation;
use crate::statevector::StateVector;

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
