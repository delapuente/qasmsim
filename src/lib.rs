#![warn(missing_docs)]
//! The `qasmsim` library includes a
//! [OPENQASM 2.0](https://github.com/Qiskit/openqasm/blob/master/spec-human/)
//! parser and interpreter, along with a statevector simulator. Compiled with
//! default features, the library presents a `qasmsim` CLI for running programs
//! like the following one:
//!
//! ```qasm
//! OPENQASM 2.0;
//! include "qelib1.inc";
//! qreg q[2];
//! creg c[2];
//! h q[0];
//! cx q[0], q[1];
//! measure q -> c;
//! ```

mod api;
mod arch;
mod complex;
mod interpreter;
mod qe;
mod semantics;

#[cfg(not(target_arch = "wasm32"))]
pub mod error;

#[cfg(not(target_arch = "wasm32"))]
pub mod grammar;

#[cfg(not(target_arch = "wasm32"))]
pub mod linker;

#[cfg(not(target_arch = "wasm32"))]
pub mod statevector;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::{
    arch::native::{
        Execution,
        ExecutionTimes,
        compile_with_linker,
        default_linker,
        simulate,
        simulate_with_shots,
        run
    },
    error::QasmSimError,
    interpreter::{
        Computation,
        Histogram
    },
    semantics::QasmType
};

#[cfg(target_arch = "wasm32")]
mod grammar;

#[cfg(target_arch = "wasm32")]
mod linker;

#[cfg(target_arch = "wasm32")]
mod statevector;

#[cfg(target_arch = "wasm32")]
pub use crate::arch::wasm::run;
