//! This module exists to force using JavaScript `Math.random()` in replacement
//! of Rust `rand::random()` when targeting wasm32-unknown-unknown
//! architecture.
//!
//! For some reason, not using it, although [after enabling WASM features]
//! causes the module `cryto` not to be found.
//!
//! [after enabling WASM features]: https://rust-random.github.io/book/crates.html?highlight=wasm#wasm-support

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn random() -> f64 {
    rand::random()
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn random() -> f64 {
    js_sys::Math::random()
}
