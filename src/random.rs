#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn random() -> f64 {
    rand::random()
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn random() -> f64 {
    js_sys::Math::random()
}
