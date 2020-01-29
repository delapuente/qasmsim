#[cfg(target_arch = "wasm32")]
mod _wasm;

#[cfg(target_arch = "wasm32")]
pub use interpreter::computation::_wasm::{ Computation, new_computation };