extern crate qasmsim;

use std::process;
use std::io::{ self, Read, Error, ErrorKind };
use std::time::Instant;

fn main() -> io::Result<()> {
  let start = Instant::now();
  let mut input = String::new();

  io::stdin().read_to_string(&mut input)?;
  let result = qasmsim::run(&input);

  if let Err(error) = result {
    let mut buffer = String::new();
    qasmsim::humanize_error(&mut buffer, &error)
      .map_err(|_| Error::new(ErrorKind::Other, ""))?;
    println!("{}", buffer);
    process::exit(1);
  }
  else {
    let computation = result.unwrap();
  }

  // println!("{:?}", computation);
  // println!(
  //   "Calculated {} item statevector in {:?}s",
  //   computation.statevector.len(),
  //   start.elapsed().as_secs_f32()
  // );
  Ok(())
}
