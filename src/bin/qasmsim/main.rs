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
    match qasmsim::run(&source, options.shots) {
        Ok(result) => print_result(&result, &options).expect("print result"),
        Err(error) => eprintln!("{}", error),
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
