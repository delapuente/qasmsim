use std::collections::HashMap;
use std::iter::FromIterator;

use crate::grammar::{ ast, Lexer };
use crate::linker::Linker;
use crate::interpreter::{ self, Computation };
use crate::grammar::open_qasm2;
use crate::qe;
pub use crate::error::Result;

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

pub fn execute<'src, 'program>(program: &'program ast::OpenQasmProgram) -> Result<'src, Computation> {
  interpreter::runtime::execute(program)
}