mod humanize;

use std::error;
use std::convert;
use std::fmt;

pub use humanize::humanize_error;
use crate::grammar::ParseError;

#[derive(Debug, Clone, PartialEq)]
pub enum QasmSimError {
  UnknownError (String),
  SyntaxError {
    msg: String,
    lineno: usize,
    startpos: usize,
    endpos: Option<usize>,
    linesrc: Option<String>,
    help: Option<String>
  }
}

impl fmt::Display for QasmSimError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      QasmSimError::UnknownError (msg) => { write!(f, "{}", msg) }
      QasmSimError::SyntaxError { msg, .. } => { write!(f, "{}", msg) }
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

type ErrAndSrc<'a> = (ParseError, &'a str);

impl convert::From<ErrAndSrc<'_>> for QasmSimError {
  fn from(err_and_src: ErrAndSrc) -> Self {
    let (err, src) = err_and_src;
    match err {
      ParseError::InvalidToken { location } => {
        let (lineno, startpos, linesrc) = into_doc_coords(location, src);
        QasmSimError::SyntaxError {
          msg: "invalid token".into(),
          lineno,
          startpos,
          endpos: None,
          linesrc: Some(linesrc.into()),
          help: None
        }
      }
      ParseError::UnrecognizedEOF { location: _, expected } => {
        let expectation = expectation(&expected);
        let (lineno, startpos, linesrc) = into_doc_coords(src.len() - 1, src);
        QasmSimError::SyntaxError {
          msg: format!("{}, found EOF", &expectation),
          lineno,
          startpos: startpos + 1,
          endpos: None,
          linesrc: Some(linesrc.into()),
          help: Some(format!("{} here", hint(&expected)))
        }
      }
      ParseError::UnrecognizedToken { token, expected } => {
        let (start, token, end) = token;
        let expectation = expectation(&expected);
        let (lineno, startpos, linesrc) = into_doc_coords(start, src);
        let endpos = if end >= src.len() { linesrc.len() } else { into_doc_coords(end, src).1 };
        QasmSimError::SyntaxError {
          msg: format!("{}, found \"{}\"", &expectation, &token),
          lineno,
          startpos,
          endpos: Some(endpos),
          linesrc: Some(linesrc.into()),
          help: Some(format!("{} before this", hint(&expected)))
        }
      }
      ParseError::ExtraToken { token } => {
        let (start, token, end) = token;
        let (lineno, startpos, linesrc) = into_doc_coords(start, src);
        let (_, endpos, _) = into_doc_coords(end, src);
        QasmSimError::SyntaxError {
          msg: format!("unexpected \"{}\" found", &token),
          lineno,
          startpos,
          endpos: Some(endpos),
          linesrc: Some(linesrc.into()),
          help: None
        }
      }
      ParseError::User { error } => {
        // Transform into InvalidToken and launch the conversion again
        (ParseError::InvalidToken{ location: error.location }, src).into()
      }
    }
  }
}

fn into_doc_coords(pos: usize, doc: &str) -> (usize, usize, &str) {
  assert!(pos < doc.len(), "pos={} must in the range 0..doc.len()={}", pos, doc.len());

  let mut lineno = 1;
  let mut startpos = 0;
  let mut linestart = 0;
  let mut lineend = 0;

  for (idx, c) in doc.chars().enumerate() {
    if idx >= pos  {
      lineend = idx + 1;
      match c {'\n' => break, _ => continue }
    }

    if c == '\n' {
      lineno += 1;
      startpos = 0;
      linestart = idx + 1;
    }
    else if c != '\r' {
      startpos += 1;
    }
  }

  (lineno, startpos, &doc[linestart..lineend])
}

fn expectation(expected: &Vec<String>) -> String {
  let choices = list_of_choices(expected).expect("len() is greater than 0");
  format!("expected {}", choices)
}

fn hint(expected: &Vec<String>) -> String {
  let choices = list_of_choices(expected).expect("len() is greater than 0");
  format!("consider adding {}{}",
    if choices.len() == 1 { "one of " } else { "" }, choices)
}

fn list_of_choices(choices: &Vec<String>) -> Option<String> {
  let len = choices.len();
  match len {
    0 => None,
    1 => Some(choices.first().unwrap().clone()),
    _ => Some({
      let last = choices.last().unwrap();
      let except_last: Vec<String> =
        choices.iter().take(len - 1).map(|item| (*item).clone()).collect();
      format!("{}, or {}", except_last.join(", "), last)
    })
  }
}

#[cfg(test)]
mod test {
  use indoc::indoc;

  use super::into_doc_coords;

  macro_rules! test_into_doc_coords {
    ($source:expr, $( $name:ident: $offset:expr => $expected:expr ),*) => {
      $(
        #[test]
        fn $name() {
          assert_eq!(into_doc_coords($offset, &$source), $expected);
        }
      )*
    };
  }

  test_into_doc_coords!(indoc!("
      line 1
      line 2
      line 3
    "),
    test_beginning_of_source: 0 => (1, 0, "line 1\n"),
    test_middle_of_source: 11 => (2, 4, "line 2\n"),
    test_end_of_source: 20 => (3, 6, "line 3\n")
  );
}
