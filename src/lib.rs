use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub open_qasm2, "/grammar/open_qasm2.rs");

mod qasmsim;
mod arch;
mod grammar;
mod linker;
mod semantics;
pub mod complex;
pub mod statevector;
mod interpreter;
mod qe;


#[cfg(not(target_arch = "wasm32"))]
pub use crate::arch::native::run;

#[cfg(target_arch = "wasm32")]
pub use crate::arch::wasm::run;
