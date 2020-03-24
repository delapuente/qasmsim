#![cfg(not(target_arch = "wasm32"))]

use crate::api;
use crate::interpreter::Computation;

macro_rules! measure {
  ($measure_name:expr, $block:block) => {
    {
      use std::time::Instant;
      let measurement = Instant::now();
      let result = $block;
      println!("{}: {:.2}ms", $measure_name, measurement.elapsed().as_millis());
      result
    }
  };
}

pub fn run(input: &str) -> api::Result<Computation> {
  let linked = measure!("parsing", {
    api::compile_with_linker(input, api::default_linker())?
  });
  let out = measure!("computation", {
    api::execute(&linked)
  });
  out
}