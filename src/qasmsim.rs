use std::collections::HashMap;
use std::iter::FromIterator;
use std::error;
use std::convert;
use std::fmt;

use crate::grammar::{ ast, lexer::Lexer };
use crate::linker::Linker;
use crate::interpreter::{ self, Computation };
use crate::open_qasm2;
use crate::qe;

pub type Result<T> = std::result::Result<T, QasmSimError>;

#[derive(Debug, Clone, PartialEq)]
pub enum QasmSimError {
  UnknownError(String),
  SyntaxError {
    msg: String,
    lineno: usize,
    linepos: usize,
    linesrc: String,
    help: String
  }
}

impl fmt::Display for QasmSimError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      QasmSimError::UnknownError(msg) => { write!(f, "{}", msg) }
      QasmSimError::SyntaxError {
        msg,
        lineno,
        linepos: _,
        linesrc,
        help
      } => {
        write!(f, "\
error: {}
    |
{}  | {}
    | {}
", msg, lineno, linesrc, help) }
    }
  }
}

impl error::Error for QasmSimError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    None
  }
}

impl convert::From<String> for QasmSimError {
  fn from(err: String) -> Self {
    QasmSimError::UnknownError(err)
  }
}

pub fn default_linker() -> Linker {
  Linker::with_embedded(HashMap::from_iter(vec![
    ("qelib1.inc".to_owned(), qe::QELIB1.to_owned())
  ]))
}

pub fn compile(input: &str) -> Result<ast::OpenQasmProgram> {
  let lexer = Lexer::new(&input);
  let parser = open_qasm2::OpenQasmProgramParser::new();
  parser.parse(lexer).or_else(|e| Err(format!("{}", e).into()))
}

pub fn compile_with_linker(input: &str, linker: Linker) -> Result<ast::OpenQasmProgram> {
  let program = compile(&input)?;
  linker.link(program).or_else(|e| Err(format!("{}", e).into()))
}

pub fn execute(program: &ast::OpenQasmProgram) -> Result<Computation> {
  interpreter::runtime::execute(program).or_else(|e| Err(format!("{}", e).into()))
}