use std::collections::HashMap;
use std::iter::FromIterator;

use crate::grammar::{ ast, Lexer };
use crate::linker::Linker;
use crate::interpreter::{ self, Computation };
use crate::open_qasm2;
use crate::qe;
use crate::error::QasmSimError;

pub type Result<T> = std::result::Result<T, QasmSimError>;

pub fn default_linker() -> Linker {
  Linker::with_embedded(HashMap::from_iter(vec![
    ("qelib1.inc".to_owned(), qe::QELIB1.to_owned())
  ]))
}

pub fn compile(input: &str) -> Result<ast::OpenQasmProgram> {
  let lexer = Lexer::new(&input);
  let parser = open_qasm2::OpenQasmProgramParser::new();
  parser.parse(lexer).map_err(|err| (err, input).into())
}

pub fn compile_with_linker(input: &str, linker: Linker) -> Result<ast::OpenQasmProgram> {
  let program = compile(&input)?;
  linker.link(program).or_else(|e| Err(format!("{}", e).into()))
}

pub fn execute(program: &ast::OpenQasmProgram) -> Result<Computation> {
  interpreter::runtime::execute(program).or_else(|e| Err(format!("{}", e).into()))
}