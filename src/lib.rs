mod api;
mod error;
mod humanize;
mod arch;
#[macro_use]
pub mod grammar;
mod linker;
mod semantics;
pub mod complex;
pub mod statevector;
mod interpreter;
mod qe;

pub use crate::interpreter::{ Computation, Histogram, runtime::QasmType };
pub use crate::error::QasmSimError;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::arch::native::{ Run, RunTimes };

#[cfg(not(target_arch = "wasm32"))]
pub use crate::arch::native::run;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::arch::native::default_linker;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::arch::native::compile_with_linker;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::arch::native::execute;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::arch::native::execute_with_shots;

#[cfg(target_arch = "wasm32")]
pub use crate::arch::wasm::run;
