use std::error;
use std::convert;
use std::fmt;

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
    match self {
      QasmSimError::UnknownError(msg) => { write!(f, "{}", msg)?; }
      QasmSimError::SyntaxError { kind, expected, token, lineno, startpos, .. } => {
        write!(f, "{:?}:", kind)?;
        let expected_str = expected.join(", ");
        if expected.len() > 1 { write!(f, " expected one of {}", expected_str)?; }
        else if expected.len() > 0 { write!(f, " expected {}", expected_str)?; }
        if let Some(token) = token { write!(f, ", found {}", token)?; }
        write!(f, " at L{}C{}", lineno, startpos)?;
      }
    }
    Ok(())
  }
}

impl error::Error for QasmSimError<'_> { }

impl convert::From<String> for QasmSimError<'_> {
  fn from(err: String) -> Self {
    QasmSimError::UnknownError(err)
  }
}
