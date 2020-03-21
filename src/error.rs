mod humanize;

use std::error;
use std::convert;
use std::fmt;

pub use humanize::humanize_error;
use crate::grammar::ParseError;

#[derive(Debug, Clone, PartialEq)]
pub enum QasmSimError<'src> {
  UnknownError (String),
  SyntaxError { error: ParseError, source: &'src str }
}

impl fmt::Display for QasmSimError<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      QasmSimError::UnknownError(msg) => { write!(f, "{}", msg) }
      QasmSimError::SyntaxError { error, .. } => { write!(f, "{}", error) }
    }
  }
}

impl error::Error for QasmSimError<'_> {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match self {
      QasmSimError::UnknownError(_) => None,
      QasmSimError::SyntaxError{ error, .. } => Some(error)
    }
  }
}

impl convert::From<String> for QasmSimError<'_> {
  fn from(err: String) -> Self {
    QasmSimError::UnknownError(err)
  }
}

type ErrAndSrc<'src> = (ParseError, &'src str);

impl<'src> convert::From<ErrAndSrc<'src>> for QasmSimError<'src> {
  fn from(err_and_src: ErrAndSrc<'src>) -> Self {
    let (error, source) = err_and_src;
    QasmSimError::SyntaxError { error, source }
  }
}
