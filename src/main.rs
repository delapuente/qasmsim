extern crate qasmsim;

use std::process;
use std::io::{ self, Read };
use std::time::Instant;

fn main() -> io::Result<()> {
  let start = Instant::now();
  let mut input = String::new();

  io::stdin().read_to_string(&mut input)?;
  let result = qasmsim::run(&input);

  if let Err(err) = result {
    println!("An error happened: {}", err);
    process::exit(1);
  }

  let computation = result.unwrap();
  let state_vector = &computation.statevector;

  println!("{:?}", state_vector);
  println!("Calculated {} item statevector in {:?}s", state_vector.len(), start.elapsed().as_secs_f32());
  Ok(())
}
