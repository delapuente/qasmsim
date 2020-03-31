use std::collections::HashMap;
use std::io::{ self, Write };
use std::iter::FromIterator;

use prettytable::{ Table, row, cell, format };

use qasmsim::{ Run, RunTimes, Histogram };
use qasmsim::statevector::StateVector;

use crate::options::Options;

pub fn print<W>(buffer: &mut W, result: &Run, options: &Options) where W: Write {

  writeln!(buffer, "").expect("writes");
  if let Some(_) = options.shots {
    if options.verbose > 0 {
      writeln!(buffer, "Memory histogram:").expect("writes");
    }
    let histogram = result.histogram.as_ref().expect("there is some histogram");
    print_histogram(buffer, histogram, options).expect("writes");
  }
  else {
    if options.verbose > 0 {
      writeln!(buffer, "Memory:").expect("writes");
    }
    print_memory(buffer, &result.memory, options).expect("writes");
  }
  if options.verbose > 0 {
    writeln!(buffer, "").expect("writes");
  }

  if (options.statevector || options.probabilities) && options.shots.is_none() {
    if options.verbose > 0 {
      writeln!(buffer, "Simulation state:").expect("writes");
    }
    print_state(buffer, &result.statevector, &result.probabilities, options).expect("writes");
  }
  if options.verbose > 0 {
    writeln!(buffer, "").expect("writes");
  }

  if options.times {
    if options.verbose > 0 {
      writeln!(buffer, "Times:").expect("writes");
    }
    print_times(buffer, &result.times).expect("writes");
  }
  if options.verbose > 0 {
    writeln!(buffer, "").expect("writes");
  }
}

fn print_memory<W>(buffer: &mut W, memory: &HashMap<String, u64>, options: &Options)
-> io::Result<()> where W: Write {
  let histogram = HashMap::from_iter(
    memory.iter().map(|(key, value)| (key.clone(), vec![(*value, 1)]))
  );
  print_memory_summary(buffer, &histogram, options, true)
}

fn print_histogram<W>(buffer: &mut W, histogram: &Histogram, options: &Options)
-> io::Result<()> where W: Write {
  print_memory_summary(buffer, histogram, options, false)
}

fn print_memory_summary<W>(buffer: &mut W, histogram: &Histogram, options: &Options, omit_count: bool)
-> io::Result<()> where W: Write {
  let mut table = Table::new();
  table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

  let binary = options.binary;
  let hexadecimal = options.hexadecimal;
  let integer = if binary || hexadecimal { options.integer } else { true };

  let mut titles = row![c -> "Name"];
  if integer { titles.add_cell(cell!(c -> "Int value")); }
  if hexadecimal { titles.add_cell(cell!(c -> "Hex value")); }
  if binary { titles.add_cell(cell!(c -> "Bin value")); }
  if !omit_count { titles.add_cell(cell!(c -> "Count")); }
  table.set_titles(titles);

  for (key, hist) in histogram {
    for (idx, (value, count)) in hist.iter().enumerate() {
      let mut row = row![r -> if idx == 0 { key } else { "" }];
      if integer { row.add_cell(cell!(r -> value)); }
      if hexadecimal { row.add_cell(cell!(r -> format!("0x{:x}", value))); }
      if binary { row.add_cell(cell!(r -> format!("0b{:b}", value))); }
      if !omit_count { row.add_cell(cell!(r -> count)); }
      table.add_row(row);
    }
  }

  write!(buffer, "{}", table)
}


fn print_state<W>(buffer: &mut W, statevector: &StateVector, probabilities: &Vec<f64>, options: &Options)
-> io::Result<()> where W: Write {
  assert!(
    options.statevector || options.probabilities,
    "at least one of probabibilities or statevector should be provided"
  );

  let mut table = Table::new();
  table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

  let mut titles = row![c -> "Base"];
  if options.statevector {
    titles.add_cell(cell!(c -> "Real"));
    titles.add_cell(cell!(c -> "Imaginary"));
  }
  if options.probabilities {
    titles.add_cell(cell!(c -> "Probability"));
  }
  table.set_titles(titles);

  for idx in 0..statevector.bases.len() {
    let mut row = row![idx];
    if options.statevector {
      row.add_cell(cell!(format!("{:.6}", statevector.bases[idx].re)));
      row.add_cell(cell!(format!("{:.6}", statevector.bases[idx].im)));
    }
    if options.probabilities {
      row.add_cell(cell!(format!("{:.6}", probabilities[idx])));
    }
    table.add_row(row);
  }

  write!(buffer, "{}", table)
}

fn print_times<W>(buffer: &mut W, times: &RunTimes)
-> io::Result<()> where W: Write {
  let mut table = Table::new();
  table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

  table.set_titles(row!["Name", "Duration (ms)"]);
  table.add_row(row!["parsing", times.parsing_time]);
  table.add_row(row!["simulation", times.simulation_time]);

  write!(buffer, "{}", table)
}