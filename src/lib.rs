mod api;
mod error;
mod humanize;
mod arch;
pub mod grammar;
mod linker;
mod semantics;
pub mod complex;
pub mod statevector;
mod interpreter;
mod qe;

pub use crate::error::{ QasmSimError, ErrorKind };
pub use crate::humanize::humanize_error;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::arch::native::run;

#[cfg(target_arch = "wasm32")]
pub use crate::arch::wasm::run;
