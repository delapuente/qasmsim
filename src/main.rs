extern crate qasmsim;

use std::collections::HashMap;
use std::io::{ self, Read };
use std::fs;
use std::path::PathBuf;
use std::fmt::{ self, Write };

use qasmsim::{ Run, RunTimes, Histogram };
use qasmsim::statevector::StateVector;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "qasmsim", about = "A QASM interpreter and quantum simulator in Rust.")]
struct Options {

  /// QASM program file, read from stdin if not present.
  #[structopt(parse(from_os_str))]
  source: Option<PathBuf>,

  /// Output file, stdout if not present.
  #[structopt(long)]
  output: Option<PathBuf>,

  /// Prints the state vector of the simulation.
  #[structopt(long)]
  statevector: bool,

  /// Prints the probabilities vector of the simulation.
  #[structopt(long)]
  probabilities: bool,

  /// Prints times measured for parsing and simulating.
  #[structopt(short, long)]
  times: bool,

  /// Specify the number of simulations. If ommited, only one simulation is
  /// run and there will be no histogram among the results. If specified,
  /// the state and probabilities vector correspond to the latest execution.
  #[structopt(long)]
  shots: Option<usize>
}

fn main() -> io::Result<()> {
  let options = Options::from_args();
  let source = get_source(&options.source)?;
  match qasmsim::run(&source, options.shots) {
    Ok(result) => print_result(&result, &options).expect("print result"),
    Err(error) => eprintln!("{}", error)
  }
  Ok(())
}

fn get_source(source: &Option<PathBuf>) -> io::Result<String> {
  if let Some(path) = source {
    fs::read_to_string(path)
  }
  else {
    let mut source = String::new();
    io::stdin().read_to_string(&mut source)?;
    Ok(source)
  }
}

fn print_result(result: &Run, options: &Options) -> fmt::Result {
  let mut buffer = String::new();

  if let Some(_) = options.shots {
    let histogram = result.histogram.as_ref().expect("there is some histogram");
    print_histogram(&mut buffer, histogram).expect("can print");
  }

  print_memory(&mut buffer, &result.memory).expect("can print");

  if options.statevector {
    print_statevector(&mut buffer, &result.statevector).expect("can print");
  }

  if options.probabilities {
    print_probabilities(&mut buffer, &result.probabilities).expect("can print");
  }

  if options.times {
    print_times(&mut buffer, &result.times).expect("can print");
  }

  Ok(match &options.output {
    Some(path) => fs::write(path, buffer).expect("can write"),
    None => print!("{}", buffer)
  })
}

fn print_histogram(buffer: &mut String, histogram: &Histogram) -> fmt::Result {
  for (key, values) in histogram {
    let entries_str: Vec<String> = values.iter().map(|(x, y)| format!("({}: {})", x, y)).collect();
    writeln!(buffer, "{} => [{}]", &key, entries_str.join(", "))?;
  }
  Ok(())
}

fn print_memory(buffer: &mut String, memory: &HashMap<String, u64>) -> fmt::Result {
  for (key, value) in memory {
    writeln!(buffer, "{} => {}", key, value)?;
  }
  Ok(())
}

fn print_probabilities(buffer: &mut String, probabilities: &Vec<f64>) -> fmt::Result {
  let width = format!("{}", probabilities.len() - 1).len();
  for (idx, chance) in probabilities.iter().enumerate() {
    writeln!(buffer, "{1: >0$}: {2:.6}%", width, idx, *chance * 100.0)?;
  }
  Ok(())
}

fn print_statevector(buffer: &mut String, statevector: &StateVector) -> fmt::Result {
  let width = format!("{}", statevector.len() - 1).len();
  for (idx, amplitude) in statevector.bases.iter().enumerate() {
    writeln!(buffer, "{1: >0$}: {2:.6}", width, idx, amplitude)?;
  }
  Ok(())
}

fn print_times(buffer: &mut String, times: &RunTimes) -> fmt::Result {
  writeln!(buffer, "parsing_time: {}", times.parsing_time)?;
  writeln!(buffer, "simulation_time: {}", times.simulation_time)?;
  Ok(())
}
