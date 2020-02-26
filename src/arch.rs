
pub mod wasm {
  #![cfg(target_arch = "wasm32")]

  use wasm_bindgen::prelude::wasm_bindgen;
  use console_error_panic_hook;

  use crate::qasmsim::do_run;

  #[wasm_bindgen]
  pub fn run(input: &str) -> Vec<f64> {
    let computation = do_run(input).unwrap();
    computation.statevector.bases.iter().map(|c| vec![c.re, c.im]).flatten().collect()
  }

  #[wasm_bindgen(start)]
  pub fn init() {
    use std::panic;
    panic::set_hook(Box::new(console_error_panic_hook::hook))
  }
}

pub mod native {
  #![cfg(not(target_arch = "wasm32"))]

  pub use crate::qasmsim::do_run as run;
}