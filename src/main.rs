extern crate qasmsim;

use std::collections::HashMap;
use std::io::{ self, Read };
use std::fs;
use std::path::PathBuf;
use std::fmt::{ self, Write };
use std::iter::FromIterator;

use structopt::StructOpt;
use prettytable::{ Table, row, cell, format };

use qasmsim::{ Run, RunTimes, Histogram };
use qasmsim::statevector::StateVector;

#[derive(Debug, StructOpt)]
#[structopt(name = "qasmsim", about = "A QASM interpreter and quantum simulator in Rust.")]
struct Options {

  /// QASM program file, read from stdin if not present.
  #[structopt(parse(from_os_str))]
  source: Option<PathBuf>,

  /// Output file, stdout if not present.
  #[structopt(long)]
  output: Option<PathBuf>,

  /// Verbosity of the output.
  #[structopt(short, parse(from_occurrences))]
  verbose: u64,

  /// Prints the binary representation of the values.
  #[structopt(long, short="b")]
  binary: bool,

  /// Prints the hexadecimal representation of the values.
  #[structopt(long, short="x")]
  hexadecimal: bool,

  /// Prints the interger representation of the values. Default option.
  #[structopt(long, short="i")]
  integer: bool,

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
    if options.verbose > 0 {
      writeln!(&mut buffer, "Memory histogram:")?;
    }
    let histogram = result.histogram.as_ref().expect("there is some histogram");
    print_histogram(&mut buffer, histogram, options).expect("can print");
  }
  else {
    if options.verbose > 0 {
      writeln!(&mut buffer, "Memory:")?;
    }
    print_memory(&mut buffer, &result.memory, options).expect("can print");
  }
  if options.verbose > 0 {
    writeln!(&mut buffer, "")?;
  }

  if options.statevector || options.probabilities {
    let statevector = if options.statevector { Some(&result.statevector) } else { None };
    let probabilities = if options.probabilities { Some(&result.probabilities) } else { None };
    if options.verbose > 0 {
      writeln!(&mut buffer, "Simulation state:")?;
    }
    print_state(&mut buffer, statevector, probabilities).expect("can print");
  }
  if options.verbose > 0 {
    writeln!(&mut buffer, "")?;
  }

  if options.times {
    if options.verbose > 0 {
      writeln!(&mut buffer, "Times:")?;
    }
    print_times(&mut buffer, &result.times).expect("can print");
  }
  if options.verbose > 0 {
    writeln!(&mut buffer, "")?;
  }

  Ok(match &options.output {
    Some(path) => fs::write(path, buffer).expect("can write"),
    None => print!("{}", buffer)
  })
}

fn print_memory(buffer: &mut String, memory: &HashMap<String, u64>, options: &Options) -> fmt::Result {
  let histogram = HashMap::from_iter(
    memory.iter().map(|(key, value)| (key.clone(), vec![(*value, 1)]))
  );
  print_universal_table(buffer, &histogram, options, true)
}

fn print_histogram(buffer: &mut String, histogram: &Histogram, options: &Options) -> fmt::Result {
  print_universal_table(buffer, histogram, options, false)
}

fn print_universal_table(buffer: &mut String, histogram: &Histogram, options: &Options, omit_count: bool) -> fmt::Result {
  let mut table = Table::new();
  table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

  let binary = options.binary;
  let hexadecimal = options.hexadecimal;
  let integer = if binary || hexadecimal { options.integer } else { true };

  match (integer, hexadecimal, binary) {
    (false, false, true) => {
      let mut titles = row![c -> "Name", c -> "Bin value"];
      if !omit_count { titles.add_cell(cell!(c -> "Count")); }
      table.set_titles(titles);

      for (key, hist) in histogram {
        for (idx, (value, count)) in hist.iter().enumerate() {
          table.add_row(row![
            r -> if idx == 0 { key } else { "" },
            r -> format!("0b{:b}", value)
          ]);
          let last_row = table.len() - 1;
          if !omit_count { table[last_row].add_cell(cell!(r -> count)); }
        }
      }
    },
    (false, true, false) => {
      table.set_titles(row![c -> "Name", c -> "Hex value", c -> "Count"]);
      for (key, hist) in histogram {
        for (idx, (value, count)) in hist.iter().enumerate() {
          table.add_row(row![
            r -> if idx == 0 { key } else { "" },
            r -> format!("0x{:x}", value),
            r -> count
          ]);
        }
      }
    },
    (false, true, true) => {
      table.set_titles(row![c -> "Name", c -> "Hex value", c -> "Bin value", c -> "Count"]);
      for (key, hist) in histogram {
        for (idx, (value, count)) in hist.iter().enumerate() {
          table.add_row(row![
            r -> if idx == 0 { key } else { "" },
            r -> format!("0x{:x}", value),
            r -> format!("0b{:b}", value),
            r -> count
          ]);
        }
      }
    },
    (true, false, false) => {
      table.set_titles(row![c -> "Name", c -> "Int value", c -> "Count"]);
      for (key, hist) in histogram {
        for (idx, (value, count)) in hist.iter().enumerate() {
          table.add_row(row![
            r -> if idx == 0 { key } else { "" },
            r -> value,
            r -> count
          ]);
        }
      }
    },
    (true, false, true) => {
      table.set_titles(row![c -> "Name", c -> "Int value", c -> "Bin value", c -> "Count"]);
      for (key, hist) in histogram {
        for (idx, (value, count)) in hist.iter().enumerate() {
          table.add_row(row![
            r -> if idx == 0 { key } else { "" },
            r -> value,
            r -> format!("0b{:b}", value),
            r -> count
          ]);
        }
      }
    },
    (true, true, false) => {
      table.set_titles(row![c -> "Name", c -> "Int value", c -> "Hex value", c -> "Count"]);
      for (key, hist) in histogram {
        for (idx, (value, count)) in hist.iter().enumerate() {
          table.add_row(row![
            r -> if idx == 0 { key } else { "" },
            r -> value,
            r -> format!("0x{:x}", value),
            r -> count
          ]);
        }
      }
    },
    (true, true, true) => {
      table.set_titles(row![c -> "Name", c -> "Int value", c -> "Hex value", c -> "Bin value", c -> "Count"]);
      for (key, hist) in histogram {
        for (idx, (value, count)) in hist.iter().enumerate() {
          table.add_row(row![
            r -> if idx == 0 { key } else { "" },
            r -> value,
            r -> format!("0x{:x}", value),
            r -> format!("0b{:b}", value),
            r -> count
          ]);
        }
      }
    },
    _ => unreachable!("all headers possibilities should be covered.")
  }

  write!(buffer, "{}", table)
}


fn print_state(buffer: &mut String, statevector: Option<&StateVector>, probabilities: Option<&Vec<f64>>) -> fmt:: Result {
  let mut table = Table::new();
  table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

  match (statevector, probabilities) {
    (None, None) => panic!("at least one of probabibilities or statevector should be provided"),
    (None, Some(probabilities)) => {
      table.set_titles(row!["Base", "Probability"]);
      for (idx, chance) in probabilities.iter().enumerate() {
        table.add_row(row![idx, format!("{:.6}", chance)]);
      }
    }
    (Some(statevector), None) => {
      table.set_titles(row!["Base", "Real", "Imaginary"]);
      for (idx, chance) in statevector.bases.iter().enumerate() {
        table.add_row(row![
          idx,
          format!("{:.6}", chance.re),
          format!("{:.6}", chance.im)
        ]);
      }
    }
    (Some(statevector), Some(probabilities)) => {
      table.set_titles(row!["Base", "Real", "Imaginary", "Probability"]);
      for (idx, chance) in statevector.bases.iter().enumerate() {
        table.add_row(row![
          idx,
          format!("{:.6}", chance.re),
          format!("{:.6}", chance.im),
          format!("{:.6}", probabilities[idx])
        ]);
      }
    }
  }

  write!(buffer, "{}", table)
}

fn print_times(buffer: &mut String, times: &RunTimes) -> fmt::Result {
  let mut table = Table::new();
  table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

  table.set_titles(row!["Name", "Duration (ms)"]);
  table.add_row(row!["parsing", times.parsing_time]);
  table.add_row(row!["simulation", times.simulation_time]);

  write!(buffer, "{}", table)
}
