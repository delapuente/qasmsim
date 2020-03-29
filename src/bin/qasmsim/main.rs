mod output;
mod options;

use std::io::{ self, Read };
use std::fs;
use std::path::PathBuf;

use structopt::StructOpt;

use qasmsim::Run;

fn main() -> io::Result<()> {
  let options = options::Options::from_args();
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

fn print_result(result: &Run, options: &options::Options) -> io::Result<()> {
  match &options.output {
    None => {
      println!("{}", output::tabular::prints(result, options));
    }
    Some(_) => {
      //output::csv::prints(result, options);
    }
  }
  Ok(())
}
