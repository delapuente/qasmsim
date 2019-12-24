extern crate qasmsim;

use std::io::{self, Read};
use std::time::Instant;

fn main() -> io::Result<()> {
  let start = Instant::now();
  let mut input = String::new();
  io::stdin().read_to_string(&mut input)?;
  let state_vector = qasmsim::run(&input);
  //println!("{:?}", state_vector);
  println!("Calculated {} item statevector in {:?}s", state_vector.len(), start.elapsed().as_secs_f32());
  Ok(())
}
