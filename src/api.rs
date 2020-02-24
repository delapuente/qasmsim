use std::collections::HashMap;
use std::iter::FromIterator;

use crate::grammar::lexer::Lexer;
use crate::linker::Linker;
use crate::interpreter::{ self, computation::Computation };
use crate::qe;
use crate::open_qasm2;

pub fn do_run(input: &str) -> Result<Computation, String> {
  let linker = Linker::with_embedded(HashMap::from_iter(vec![
    ("qelib1.inc".to_owned(), qe::QELIB1.to_owned())
  ]));
  let lexer = Lexer::new(&input);
  let parser = open_qasm2::OpenQasmProgramParser::new();
  let program = parser.parse(lexer).unwrap();
  let linked = linker.link(program).unwrap();
  interpreter::runtime::execute(&linked)
}
