//! This module exists to force using JavaScript `Math.random()` in replacement
//! of Rust `rand::random()` when targeting wasm32-unknown-unknown
//! architecture.
//!
//! For some reason, not using it, even [after enabling WASM features],
//! causes the module `crypto` not to be found in the browser. It seems the
//! location it tries to load `crypto` from is `undefined`:
//!
//! > can't access property "crypto", getObject(...) is undefined
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
