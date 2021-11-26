mod options;
mod output;

use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use structopt::StructOpt;

use qasmsim::Execution;

fn main() -> io::Result<()> {
    let options = options::Options::from_args();
    let source = source(&options.source)?;
    if let Some(gate_name) = options.info {
        match qasmsim::get_gate_info(&source, &gate_name) {
            Ok((docstring, (name, real_params, quantum_params))) => {
                print_info(&docstring, &name, &real_params, &quantum_params)
                    .expect("print gate info")
            }
            Err(error) => {
                eprintln!("{}", error);
                std::process::exit(1);
            }
        }
    } else {
        match qasmsim::run(&source, options.shots) {
            Ok(result) => print_result(&result, &options).expect("print result"),
            Err(error) => {
                eprintln!("{}", error);
                std::process::exit(1);
            }
        }
    }
    Ok(())
}

fn source(source: &Option<PathBuf>) -> io::Result<String> {
    if let Some(path) = source {
        fs::read_to_string(path)
    } else {
        let mut source = String::new();
        io::stdin().read_to_string(&mut source)?;
        Ok(source)
    }
}

fn print_info(
    docstring: &str,
    name: &str,
    real_params: &[String],
    quantum_params: &[String],
) -> io::Result<()> {
    println!(
        "gate {}{} {}",
        name,
        match real_params.len() {
            0 => String::from(""),
            _ => format!("({})", real_params.join(", ")),
        },
        quantum_params.join(" ")
    );
    println!("{}", docstring);
    Ok(())
}

fn print_result(result: &Execution, options: &options::Options) -> io::Result<()> {
    match &options.out {
        None => {
            let stdout = io::stdout();
            let mut handle = io::BufWriter::new(stdout.lock());
            output::tabular::print(&mut handle, result, options);
        }
        Some(path) => {
            let mut path = PathBuf::from(path);
            output::csv::print(&mut path, result, options);
        }
    }
    Ok(())
}
