use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, Clone, PartialEq, Eq, StructOpt, Hash)]
#[structopt(
    name = "qasmsim",
    about = "A QASM interpreter and quantum simulator in Rust."
)]
pub struct Options {
    /// QASM program file, read from stdin if not present.
    #[structopt(parse(from_os_str))]
    pub source: Option<PathBuf>,

    /// Output files prefix, print in the stdout if not present. The output
    /// format of each file is CSV. At most, three files are created with the
    /// names out.memory.csv, out.state.csv and out.times.csv
    #[structopt(long)]
    pub out: Option<PathBuf>,

    /// Verbosity of the output.
    #[structopt(short, parse(from_occurrences))]
    pub verbose: u64,

    /// Prints the binary representation of the values.
    #[structopt(long, short = "b")]
    pub binary: bool,

    /// Prints the hexadecimal representation of the values.
    #[structopt(long, short = "x")]
    pub hexadecimal: bool,

    /// Prints the interger representation of the values. Default option.
    #[structopt(long, short = "i")]
    pub integer: bool,

    /// Prints the state vector of the simulation. Ignored if shots is set.
    #[structopt(long)]
    pub statevector: bool,

    /// Prints the probabilities vector of the simulation. Ignored if shots is set.
    #[structopt(long)]
    pub probabilities: bool,

    /// Prints times measured for parsing and simulating.
    #[structopt(short, long)]
    pub times: bool,

    /// Specify the number of simulations.
    #[structopt(long)]
    pub shots: Option<usize>,

    /// Show gate-related information.
    #[structopt(long)]
    pub info: Option<String>,
}
