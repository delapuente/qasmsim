use std::collections::HashMap;
use std::io::{self, Write};
use std::iter::FromIterator;

use prettytable::{cell, format, row, Table};

use qasmsim::statevector::StateVector;
use qasmsim::{Execution, ExecutionTimes, Histogram};

use crate::options::Options;

/// Writes the `msg` in the `buffer` if `options.verbose` is greater than 0.
macro_rules! vvprint {
    ($options:expr, $buffer:expr, $msg:expr) => {{
        if $options.verbose > 0 {
            write!($buffer, $msg)
        } else {
            Ok(())
        }
    }};
}

/// Writes the `msg` in the `buffer` if `options.verbose` is greater than 0 and
/// ends it with a newline.
macro_rules! vvprintln {
    ($options:expr, $buffer:expr, $msg:literal) => {{
        vvprint!($options, $buffer, concat!($msg, "\n"))
    }};
    ($options:expr, $buffer:expr) => {{
        vvprintln!($options, $buffer, "")
    }};
}

pub fn print<W>(buffer: &mut W, result: &Execution, options: &Options)
where
W: Write,
{
    do_print(buffer, result, options).expect("writes in stdout");
}

fn do_print<W>(buffer: &mut W, result: &Execution, options: &Options)
-> io::Result<()>
where
    W: Write,
{
    if options.shots.is_some() {
        let histogram = result.histogram().as_ref().expect("there is some histogram");
        if !histogram.is_empty() {
            vvprintln!(options, buffer, "Memory histogram:")?;
            print_histogram(buffer, histogram, options)?;
            vvprintln!(options, buffer)?;
        }
    } else {
        let memory = result.memory();
        if !memory.is_empty() {
            vvprintln!(options, buffer, "Memory:")?;
            print_memory(buffer, memory, options)?;
            vvprintln!(options, buffer)?;
        }
    }

    if (options.statevector || options.probabilities) && options.shots.is_none() {
        vvprintln!(options, buffer, "Simulation state:")?;
        print_state(buffer, result.statevector(), result.probabilities(), options)?;
        vvprintln!(options, buffer)?;
    }

    if options.times {
        vvprintln!(options, buffer, "Times:")?;
        print_times(buffer, result.times())?;
        vvprintln!(options, buffer)?;
    }
    Ok(())
}

fn print_memory<W>(
    buffer: &mut W,
    memory: &HashMap<String, u64>,
    options: &Options,
) -> io::Result<()>
where
    W: Write,
{
    let histogram = HashMap::from_iter(
        memory
            .iter()
            .map(|(key, value)| (key.clone(), vec![(*value, 1)])),
    );
    print_memory_summary(buffer, &histogram, options, true)
}

fn print_histogram<W>(buffer: &mut W, histogram: &Histogram, options: &Options) -> io::Result<()>
where
    W: Write,
{
    print_memory_summary(buffer, histogram, options, false)
}

fn print_memory_summary<W>(
    buffer: &mut W,
    histogram: &Histogram,
    options: &Options,
    omit_count: bool,
) -> io::Result<()>
where
    W: Write,
{
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    let binary = options.binary;
    let hexadecimal = options.hexadecimal;
    let integer = if binary || hexadecimal {
        options.integer
    } else {
        true
    };

    let mut titles = row![c -> "Name"];
    if integer {
        titles.add_cell(cell!(c -> "Int value"));
    }
    if hexadecimal {
        titles.add_cell(cell!(c -> "Hex value"));
    }
    if binary {
        titles.add_cell(cell!(c -> "Bin value"));
    }
    if !omit_count {
        titles.add_cell(cell!(c -> "Count"));
    }
    table.set_titles(titles);

    for (key, hist) in histogram {
        for (idx, (value, count)) in hist.iter().enumerate() {
            let mut row = row![r -> if idx == 0 { key } else { "" }];
            if integer {
                row.add_cell(cell!(r -> value));
            }
            if hexadecimal {
                row.add_cell(cell!(r -> format!("0x{:x}", value)));
            }
            if binary {
                row.add_cell(cell!(r -> format!("0b{:b}", value)));
            }
            if !omit_count {
                row.add_cell(cell!(r -> count));
            }
            table.add_row(row);
        }
    }

    write!(buffer, "{}", table)
}

fn print_state<W>(
    buffer: &mut W,
    statevector: &StateVector,
    probabilities: &[f64],
    options: &Options,
) -> io::Result<()>
where
    W: Write,
{
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

    let amplitudes_and_probabilities = statevector.as_complex().iter().zip(probabilities).enumerate();
    for (idx, (amplitude, probability)) in amplitudes_and_probabilities {
        let mut row = row![idx];
        if options.statevector {
            row.add_cell(cell!(format!("{:.6}", amplitude.re)));
            row.add_cell(cell!(format!("{:.6}", amplitude.im)));
        }
        if options.probabilities {
            row.add_cell(cell!(format!("{:.6}", probability)));
        }
        table.add_row(row);
    }

    write!(buffer, "{}", table)
}

fn print_times<W>(buffer: &mut W, times: &ExecutionTimes) -> io::Result<()>
where
    W: Write,
{
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    table.set_titles(row!["Name", "Duration (ms)"]);
    table.add_row(row!["parsing", times.parsing_time()]);
    table.add_row(row!["simulation", times.simulation_time()]);

    write!(buffer, "{}", table)
}
