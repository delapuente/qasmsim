use std::collections::HashMap;
use std::iter::FromIterator;

use crate::grammar::{ ast, lexer::Lexer };
use crate::linker::Linker;
use crate::interpreter::{ self, Computation };
use crate::open_qasm2;
use crate::qe;

pub fn run(input: &str) -> Result<Computation, String> {
  let linked = parse_and_link(input, default_linker())?;
  execute(&linked)
}

pub fn default_linker() -> Linker {
  Linker::with_embedded(HashMap::from_iter(vec![
    ("qelib1.inc".to_owned(), qe::QELIB1.to_owned())
  ]))
}

pub fn parse(input: &str) -> Result<ast::OpenQasmProgram, String> {
  let lexer = Lexer::new(&input);
  let parser = open_qasm2::OpenQasmProgramParser::new();
  parser.parse(lexer).or_else(|e| Err(format!("{}", e)))
}

pub fn parse_and_link(input: &str, linker: Linker) -> Result<ast::OpenQasmProgram, String> {
  let program = parse(&input)?;
  linker.link(program).or(Err(String::from("")))
}

pub fn execute(program: &ast::OpenQasmProgram) -> Result<Computation, String> {
  interpreter::runtime::execute(program)
}