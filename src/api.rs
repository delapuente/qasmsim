use std::collections::HashMap;
use std::iter::FromIterator;

use crate::grammar::{ ast, Lexer };
use crate::linker::Linker;
use crate::interpreter::{ self, runtime::RuntimeError };
use crate::grammar::{ open_qasm2, Location };
use crate::grammar::lexer;
use crate::qe;
use crate::semantics::SemanticError;
pub use crate::error::{ Result, ErrorKind, QasmSimError };

pub type ParseError =
  lalrpop_util::ParseError<Location, lexer::Tok, lexer::LexicalError<Location>>;

type SrcAndErr<'src, E> = (&'src str, E);

impl<'src> From<SrcAndErr<'src, ParseError>> for QasmSimError<'src> {
  fn from(src_and_err: SrcAndErr<'src, ParseError>) -> Self {
    let (input, error) = src_and_err;
    match error {
      ParseError::InvalidToken { location } => {
        let (source, lineno, startpos, endpos) = extract_line(location.0, None, input);
        QasmSimError::SyntaxError {
          kind: ErrorKind::InvalidToken,
          source,
          lineno,
          startpos,
          endpos,
          token: None,
          expected: Vec::new(),
        }
      }
      ParseError::UnrecognizedEOF { location, expected } => {
        let (source, lineno, startpos, endpos) = extract_line(location.0, None, input);
        QasmSimError::SyntaxError {
          kind: ErrorKind::UnexpectedEOF,
          source,
          lineno,
          startpos,
          endpos,
          token: None,
          expected
        }
      }
      ParseError::UnrecognizedToken { token, expected } => {
        let location = token.0;
        let endlocation = token.2;
        let (source, lineno, startpos, endpos) = extract_line(location.0, Some(endlocation.0), input);
        QasmSimError::SyntaxError {
          kind: ErrorKind::UnexpectedToken,
          source,
          lineno,
          startpos,
          endpos,
          token: Some(token.1),
          expected
        }
      }
      ParseError::ExtraToken { token } => {
        let location = token.0;
        let endlocation = token.2;
        let (source, lineno, startpos, endpos) = extract_line(location.0, Some(endlocation.0), input);
        QasmSimError::SyntaxError {
          kind: ErrorKind::UnexpectedToken,
          source,
          lineno,
          startpos,
          endpos,
          token: Some(token.1),
          expected: Vec::new()
        }
      }
      ParseError::User { error: lexer_error } => {
        let location = lexer_error.location;
        let (source, lineno, startpos, endpos) = extract_line(location.0, None, input);
        QasmSimError::SyntaxError {
          kind: ErrorKind::InvalidToken, // XXX: Actually, this should be "InvalidInput"
          source,
          lineno,
          startpos,
          endpos,
          token: None,
          expected: Vec::new()
        }
      }
    }
  }
}

impl<'src> From<SrcAndErr<'src, RuntimeError>> for QasmSimError<'src> {
  fn from(source_and_error: SrcAndErr<'src, RuntimeError>) -> Self {
    let (input, error) = source_and_error;
    match error {
      RuntimeError::Other => QasmSimError::UnknownError(format!("{:?}", error)),
      RuntimeError::UndefinedGate {
        location,
        symbol_name
      } => {
        let (source, lineno, _, _) = extract_line(location.0, None, input);
        QasmSimError::UndefinedGate {
          source: source.into(),
          lineno,
          symbol_name
        }
      }
      RuntimeError::WrongNumberOfParameters {
        are_registers,
        location,
        symbol_name,
        given,
        expected
      } => {
        let (source, lineno, _, _) = extract_line(location.0, None, input);
        QasmSimError::WrongNumberOfParameters {
          are_registers,
          source: source.into(),
          symbol_name,
          lineno,
          expected,
          given
        }
      }
      RuntimeError::SymbolNotFound {
        location,
        symbol_name
      } => {
        let (source, lineno, _, _) = extract_line(location.0, None, input);
        QasmSimError::SymbolNotFound {
          source: source.into(),
          symbol_name,
          lineno
        }
      }
      RuntimeError::IndexOutOfBounds {
        location,
        symbol_name,
        index,
        size
      } => {
        let (source, lineno, _, _) = extract_line(location.0, None, input);
        QasmSimError::IndexOutOfBounds {
          source: source.into(),
          symbol_name,
          lineno,
          size,
          index
        }
      }
      RuntimeError::SemanticError(semantic_error) => {
        match semantic_error {
          SemanticError::RedefinitionError { symbol_name, location, previous_location } => {
            let (source, lineno, _, _) = extract_line(location.0, None, input);
            let (_, previous_lineno, _, _) = extract_line(previous_location.0, None, input);
            QasmSimError::SemanticError {
              source: source.into(),
              symbol_name,
              lineno,
              previous_lineno
            }
          }
        }
      }
    }
  }
}

fn extract_line(offset: usize, endoffset: Option<usize>, doc: &str) -> (&str, usize, usize, Option<usize>) {
  assert!(offset <= doc.len(),
    "linestart={} must in the range 0..=doc.len()={}", offset, doc.len());

  let mut linecount = 1;
  let mut start = 0;
  let mut end = start + 1;
  for (idx, character) in doc.chars().enumerate() {
    if idx < offset && character == '\n' {
      start = idx + 1;
      linecount += 1;
    }
    if idx >= offset {
      end = idx + 1;
      if character == '\n' { break; }
    }
  }

  if end < start {
    end = doc.len();
  }

  let startpos = offset - start;
  let endpos = match endoffset {
    None => None,
    Some(endoffset) => Some(endoffset - start)
  };

  (&doc[start..end], linecount, startpos, endpos)
}

pub fn default_linker() -> Linker {
  Linker::with_embedded(HashMap::from_iter(vec![
    ("qelib1.inc".to_owned(), qe::QELIB1.to_owned())
  ]))
}

pub fn compile(input: &str) -> Result<ast::OpenQasmProgram> {
  let lexer = Lexer::new(&input);
  let parser = open_qasm2::OpenQasmProgramParser::new();
  parser.parse(lexer).map_err(|err| QasmSimError::from((input, err)))
}

pub fn compile_with_linker<'src>(input: &'src str, linker: Linker) -> Result<'src, ast::OpenQasmProgram> {
  let program = compile(&input)?;
  linker.link(program)
}

pub use interpreter::runtime::execute;

pub use interpreter::runtime::execute_with_shots;

#[cfg(test)]
mod test_into_doc_coords {
  use indoc::indoc;

  use super::extract_line;

  macro_rules! test_get_line_src {
    ($source:expr, $( $name:ident: $offset:expr, $offsetend:expr => $expected:expr ),*) => {
      $(
        #[test]
        fn $name() {
          assert_eq!(extract_line($offset, $offsetend, &$source), $expected);
        }
      )*
    };
  }

  test_get_line_src!(indoc!("
      line 1
      line 2
      line 3"
    ),
    test_beginning_of_source: 0, None => ("line 1\n", 1, 0, None),
    test_middle_of_source: 7, None => ("line 2\n", 2, 0, None),
    test_last_character: 20, None => ("line 3", 3, 6, None)
  );
}