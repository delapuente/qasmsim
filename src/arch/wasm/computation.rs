#![cfg(target_arch = "wasm32")]

use std::collections::HashMap;
use std::convert::From;
use std::iter::IntoIterator;

use js_sys::{self, Array, Float64Array, Object};
use wasm_bindgen::prelude::JsValue;

use crate::interpreter::Computation;
use crate::statevector::StateVector;

struct JsMemory<'a>(&'a HashMap<String, u64>);
struct JsHistogram<'a>(&'a HashMap<String, Vec<(u64, usize)>>);

impl From<JsMemory<'_>> for JsValue {
    fn from(value: JsMemory) -> Self {
        let hashmap = value.0;
        let obj = Object::new();
        for (register_name, value) in hashmap {
            set!(&obj, register_name => *value as f64);
        }
        obj.into()
    }
}

impl From<JsHistogram<'_>> for JsValue {
    fn from(value: JsHistogram) -> Self {
        let hashmap = value.0;
        let obj = Object::new();
        for (register_name, histogram) in hashmap {
            let array = Array::new();
            for (value, count) in histogram {
                let pair = Array::of2(&(*value as f64).into(), &(*count as f64).into());
                array.push(&pair.into());
            }
            set!(&obj, register_name => array);
        }
        obj.into()
    }
}

impl From<Computation> for JsValue {
    fn from(computation: Computation) -> Self {
        let out = Object::new();
        set!(&out,
            "statevector" => computation.statevector(),
            "probabilities" => as_typed_array(computation.probabilities().to_vec()),
            "memory" => JsMemory(computation.memory())
        );
        if let Some(histogram) = computation.histogram() {
            set!(&out,
                "histogram" => JsHistogram(histogram)
            );
        }
        out.into()
    }
}

impl From<&StateVector> for JsValue {
    fn from(statevector: &StateVector) -> Self {
        let bases = statevector.as_complex_bases();
        let flatten_amplitudes: Vec<f64> = bases.iter().flat_map(|c| vec![c.re, c.im]).collect();
        let out = Object::new();
        set!(&out,
            "bases" => as_typed_array(flatten_amplitudes),
            "qubitWidth" => statevector.qubit_width() as f64
        );
        out.into()
    }
}

fn as_typed_array<I>(iterator: I) -> Float64Array
where
    I: IntoIterator<Item = f64>,
{
    let values: Vec<f64> = iterator.into_iter().collect();
    Float64Array::from(&values[..])
}
