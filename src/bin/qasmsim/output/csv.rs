use std::collections::HashMap;
use std::io::{self, Write};
use std::iter::FromIterator;
use std::path::PathBuf;

use csv;

use qasmsim::statevector::StateVector;
use qasmsim::{Execution, ExecutionTimes, Histogram};

use crate::options::Options;

pub fn print(path: &mut PathBuf, result: &Execution, options: &Options) {
    // TODO: Add error handling for path operations.
    let prefix = path
        .file_name()
        .expect("a valid file name")
        .to_str()
        .expect("a valid name for the filename")
        .to_owned();

    path.set_file_name(format!("{}.memory.csv", prefix));
    let mut writer = csv::Writer::from_path(&path).expect("can open the file");
    let writer_ref = &mut writer;

    if options.shots.is_some() {
        let histogram = result
            .histogram()
            .as_ref()
            .expect("there is some histogram");
        print_histogram(writer_ref, histogram, options).expect("writes");
    } else {
        print_memory(writer_ref, result.memory(), options).expect("writes");
    }

    if (options.statevector || options.probabilities) && options.shots.is_none() {
        path.set_file_name(format!("{}.state.csv", &prefix));
        let mut writer = csv::Writer::from_path(&path).expect("can open the file");
        let writer_ref = &mut writer;
        print_state(
            writer_ref,
            result.statevector(),
            result.probabilities(),
            &options,
        )
        .expect("writes");
    }

    if options.times {
        path.set_file_name(format!("{}.times.csv", &prefix));
        let mut writer = csv::Writer::from_path(path).expect("can open the file");
        let writer_ref = &mut writer;
        print_times(writer_ref, result.times()).expect("writes");
    }
}

fn print_memory<W>(
    writer: &mut csv::Writer<W>,
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
    print_memory_summary(writer, &histogram, options, true)
}

fn print_histogram<W>(
    writer: &mut csv::Writer<W>,
    histogram: &Histogram,
    options: &Options,
) -> io::Result<()>
where
    W: Write,
{
    print_memory_summary(writer, histogram, options, false)
}

fn print_memory_summary<W>(
    writer: &mut csv::Writer<W>,
    histogram: &Histogram,
    options: &Options,
    omit_count: bool,
) -> io::Result<()>
where
    W: Write,
{
    let binary = options.binary;
    let hexadecimal = options.hexadecimal;
    let integer = if binary || hexadecimal {
        options.integer
    } else {
        true
    };

    let mut titles = vec!["Name"];
    if integer {
        titles.push("Int value");
    }
    if hexadecimal {
        titles.push("Hex value");
    }
    if binary {
        titles.push("Bin value");
    }
    if !omit_count {
        titles.push("Count");
    }
    writer.write_record(&titles)?;

    for (key, hist) in histogram {
        for (value, count) in hist {
            let mut record: Vec<String> = vec![key.clone()];
            if integer {
                record.push(format!("{}", value));
            }
            if hexadecimal {
                record.push(format!("0x{:x}", value));
            }
            if binary {
                record.push(format!("0b{:b}", value));
            }
            if !omit_count {
                record.push(format!("{}", count));
            }
            writer.write_record(&record)?;
        }
    }

    Ok(())
}

fn print_state<W>(
    writer: &mut csv::Writer<W>,
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

    let mut titles = vec!["Base"];
    if options.statevector {
        titles.push("Real");
        titles.push("Imaginary");
    }
    if options.probabilities {
        titles.push("Probability");
    }
    writer.write_record(&titles)?;

    let amplitudes_and_probabilities = statevector
        .as_complex_bases()
        .iter()
        .zip(probabilities)
        .enumerate();
    for (idx, (amplitude, probability)) in amplitudes_and_probabilities {
        let mut record = vec![format!("{}", idx)];
        if options.statevector {
            record.push(format!("{:.6}", amplitude.re));
            record.push(format!("{:.6}", amplitude.im));
        }
        if options.probabilities {
            record.push(format!("{:.6}", probability));
        }
        writer.write_record(&record)?;
    }

    Ok(())
}

fn print_times<W>(writer: &mut csv::Writer<W>, times: &ExecutionTimes) -> io::Result<()>
where
    W: Write,
{
    writer.write_record(&["Name", "Duration (ms)"])?;
    writer.serialize(("parsing", times.parsing_time()))?;
    writer.serialize(("simulation", times.parsing_time()))?;
    Ok(())
}
