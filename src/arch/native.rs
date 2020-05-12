#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashMap;
use std::convert;

use crate::{api, statevector::StateVector};

use crate::interpreter::{Computation, Histogram};
use crate::error::QasmSimError;

pub use api::compile_with_linker;
pub use api::default_linker;
pub use api::simulate;
pub use api::simulate_with_shots;

macro_rules! measure {
    ($block:expr) => {{
        use std::time::Instant;
        let measurement = Instant::now();
        let result = $block;
        let elapsed = measurement.elapsed().as_millis();
        (result, elapsed)
    }};
}

/// Register the milliseconds spent in parsing the program and simulating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExecutionTimes {
    /// Time spent in parsing the program and converting it to an AST.
    pub parsing_time: u128,
    /// Time spent in simulating the program.
    pub simulation_time: u128,
}

impl ExecutionTimes {
    /// Create a new times statistics.
    pub fn new(parsing_time: u128, simulation_time: u128) -> Self {
        ExecutionTimes {
            parsing_time,
            simulation_time,
        }
    }
}

/// Represent a complete execution of a program, from parsing to simulating.
///
/// This structure is similar to [`Computation`] although this also includes
/// [time statistics] regarding parsing and execution times.
///
/// # Examples
///
/// See the [`run`] function for a complete example.
///
/// [`run`]: ./fn.run.html
/// [`Computation`]: ./struct.Computation.html
/// [time statistics]: ./struct.ExecutionTimes.html
#[derive(Debug, Clone, PartialEq)]
pub struct Execution {
    /// The statevector of the quantum system.
    pub statevector: StateVector,
    /// The probabilities associated with the state-vector.
    pub probabilities: Vec<f64>,
    /// An associative map with classical names and the classical outcomes.
    pub memory: HashMap<String, u64>,
    /// The histogram when simulating with several shots.
    pub histogram: Option<Histogram>,
    /// Time spent in parsing and performing the simulation.
    pub times: ExecutionTimes,
}

impl Execution {
    /// Create a new `Execution` instance.
    pub fn new(
        statevector: StateVector,
        probabilities: Vec<f64>,
        memory: HashMap<String, u64>,
        histogram: Option<Histogram>,
        times: ExecutionTimes,
    ) -> Self {
        Execution {
            statevector,
            probabilities,
            memory,
            histogram,
            times,
        }
    }
}

impl convert::From<(Computation, u128, u128)> for Execution {
    fn from(value: (Computation, u128, u128)) -> Self {
        let (computation, parsing_time, simulation_time) = value;
        Execution {
            statevector: computation.statevector,
            probabilities: computation.probabilities,
            memory: computation.memory,
            histogram: computation.histogram,
            times: ExecutionTimes {
                parsing_time,
                simulation_time,
            },
        }
    }
}

/// Parse and simulate the `input` OPENQASM program with optional `shots`.
///
/// # Errors
///
/// The function can fail if the source code presents an error or something
/// unexpected happens during the simulation. In this case, an `Err` variant
/// wrapping a value of [`QasmSimError`] is returned.
///
/// [`QasmSimError`]: ./error/enum.QasmSimError.html
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use qasmsim::run;
///
/// let execution = run(r#"
/// OPENQASM 2.0;
/// include "qelib1.inc";
/// qreg q[2];
/// "#, None)?;
/// # use qasmsim::QasmSimError;
/// # Ok::<(), QasmSimError>(())
/// ```
pub fn run(input: &str, shots: Option<usize>) -> api::Result<'_, Execution> {
    let (linked, parsing_time) = measure!({ compile_with_linker(input, api::default_linker()) });
    let (out, simulation_time) = measure!({
        match shots {
            None => simulate(&linked?),
            Some(shots) => simulate_with_shots(&linked?, shots),
        }
    });
    let out = out.map_err(|err| QasmSimError::from((input, err)));
    Ok(Execution::from((out?, parsing_time, simulation_time)))
}
