use std::error;
use std::convert;
use std::fmt;

use crate::humanize::humanize_error;
use crate::grammar::Tok;

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
  InvalidToken,
  UnexpectedEOF,
  UnexpectedToken
}

#[derive(Debug, Clone, PartialEq)]
pub enum QasmSimError<'src> {
  UnknownError (String),
  SyntaxError {
    kind: ErrorKind,
    source: &'src str,
    lineoffset: usize,
    lineno: usize,
    startpos: usize,
    endpos: Option<usize>,
    token: Option<Tok>,
    expected: Vec<String>,
  }
}

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
