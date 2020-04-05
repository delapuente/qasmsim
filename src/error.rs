use std::error;
use std::convert;
use std::fmt;

use crate::humanize::humanize_error;
use crate::grammar::Tok;
use crate::interpreter::runtime::QasmType;

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
  InvalidToken,
  UnexpectedEOF,
  UnexpectedToken
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeKind {
  DifferentSizeRegisters
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum QasmSimError<'src> {
  UnknownError (String),
  SyntaxError {
    kind: ErrorKind,
    source: &'src str,
    lineno: usize,
    startpos: usize,
    endpos: Option<usize>,
    token: Option<Tok>,
    expected: Vec<String>,
  },
  SemanticError {
    source: &'src str,
    symbol_name: String,
    lineno: usize,
    previous_lineno: usize
  },
  LibraryNotFound {
    source: &'src str,
    libpath: String,
    lineno: usize
  },
  RuntimeError {
    kind: RuntimeKind,
    symbol_name: String,
  },
  IndexOutOfBounds {
    source: &'src str,
    lineno: usize,
    symbol_name: String,
    index: usize,
    size: usize,
  },
  SymbolNotFound {
    source: &'src str,
    lineno: usize,
    symbol_name: String,
    expected: QasmType
  },
  WrongNumberOfParameters {
    source: &'src str,
    lineno: usize,
    symbol_name: String,
    are_registers: bool,
    given: usize,
    expected: usize
  },
  UndefinedGate {
    source: &'src str,
    lineno: usize,
    symbol_name: String
  },
  TypeMismatch {
    source: &'src str,
    lineno: usize,
    symbol_name: String,
    expected: QasmType
  }
}

pub type Result<'src, T> = std::result::Result<T, QasmSimError<'src>>;

impl fmt::Display for QasmSimError<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut buffer = String::new();
    humanize_error(&mut buffer, self)?;
    write!(f, "{}", buffer)
  }
}

impl error::Error for QasmSimError<'_> { }

impl convert::From<String> for QasmSimError<'_> {
  fn from(err: String) -> Self {
    QasmSimError::UnknownError(err)
  }
}
