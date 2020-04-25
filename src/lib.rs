mod api;
mod arch;
mod error;
mod grammar;
mod complex;
mod interpreter;
mod linker;
mod qe;
mod semantics;
mod statevector;

pub use crate::error::QasmSimError;
pub use crate::interpreter::{Computation, Histogram};

#[cfg(not(target_arch = "wasm32"))]
pub use crate::arch::native::Run;

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
