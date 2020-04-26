mod api;
mod arch;
mod error;
mod complex;
mod interpreter;
mod qe;
mod semantics;

#[cfg(not(target_arch = "wasm32"))]
pub mod grammar;

#[cfg(not(target_arch = "wasm32"))]
pub mod linker;

#[cfg(not(target_arch = "wasm32"))]
pub mod statevector;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::{
    arch::native::{
        Run,
        RunTimes,
        compile_with_linker,
        default_linker,
        execute,
        execute_with_shots,
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
