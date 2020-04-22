#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashMap;
use std::convert;

use crate::{api, statevector::StateVector};

use crate::interpreter::{Computation, Histogram};

pub use api::compile_with_linker;
pub use api::default_linker;
pub use api::execute;
pub use api::execute_with_shots;

macro_rules! measure {
    ($block:expr) => {{
        use std::time::Instant;
        let measurement = Instant::now();
        let result = $block;
        let elapsed = measurement.elapsed().as_millis();
        (result, elapsed)
    }};
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RunTimes {
    pub parsing_time: u128,
    pub simulation_time: u128,
}

impl RunTimes {
    pub fn new(parsing_time: u128, simulation_time: u128) -> Self {
        RunTimes {
            parsing_time,
            simulation_time,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    pub statevector: StateVector,
    pub probabilities: Vec<f64>,
    pub memory: HashMap<String, u64>,
    pub histogram: Option<Histogram>,
    pub times: RunTimes,
}

impl Run {
    pub fn new(
        statevector: StateVector,
        probabilities: Vec<f64>,
        memory: HashMap<String, u64>,
        histogram: Option<Histogram>,
        times: RunTimes,
    ) -> Self {
        Run {
            statevector,
            probabilities,
            memory,
            histogram,
            times,
        }
    }
}

impl convert::From<(Computation, u128, u128)> for Run {
    fn from(value: (Computation, u128, u128)) -> Self {
        let (computation, parsing_time, simulation_time) = value;
        Run {
            statevector: computation.statevector,
            probabilities: computation.probabilities,
            memory: computation.memory,
            histogram: computation.histogram,
            times: RunTimes {
                parsing_time,
                simulation_time,
            },
        }
    }
}

pub fn run(input: &str, shots: Option<usize>) -> api::Result<'_, Run> {
    let (linked, parsing_time) = measure!({ compile_with_linker(input, api::default_linker()) });
    let (out, simulation_time) = measure!({
        match shots {
            None => execute(&linked?),
            Some(shots) => execute_with_shots(&linked?, shots),
        }
    });
    let out = out.map_err(|err| api::QasmSimError::from((input, err)));
    Ok(Run::from((out?, parsing_time, simulation_time)))
}
