use std::convert;
use std::error;
use std::fmt;

use crate::grammar::lexer;
use crate::grammar::{Location, Tok};
use crate::humanize::humanize_error;
use crate::interpreter::runtime::QasmType;
use crate::interpreter::runtime::RuntimeError;
use crate::linker::LinkerError;
use crate::semantics::SemanticError;

pub type ParseError = lalrpop_util::ParseError<Location, lexer::Tok, lexer::LexicalError<Location>>;

pub type SrcAndErr<'src, E> = (&'src str, E);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum QasmSimError<'src> {
    UnknownError(String),
    InvalidToken {
        source: &'src str,
        lineno: usize,
        startpos: usize,
        endpos: Option<usize>,
        token: Option<Tok>,
        expected: Vec<String>,
    },
    UnexpectedEOF {
        source: &'src str,
        lineno: usize,
        startpos: usize,
        endpos: Option<usize>,
        token: Option<Tok>,
        expected: Vec<String>,
    },
    UnexpectedToken {
        source: &'src str,
        lineno: usize,
        startpos: usize,
        endpos: Option<usize>,
        token: Option<Tok>,
        expected: Vec<String>,
    },
    RedefinitionError {
        source: &'src str,
        symbol_name: String,
        lineno: usize,
        previous_lineno: usize,
    },
    LibraryNotFound {
        source: &'src str,
        libpath: String,
        lineno: usize,
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
        expected: QasmType,
    },
    WrongNumberOfParameters {
        source: &'src str,
        lineno: usize,
        symbol_name: String,
        are_registers: bool,
        given: usize,
        expected: usize,
    },
    UndefinedGate {
        source: &'src str,
        lineno: usize,
        symbol_name: String,
    },
    TypeMismatch {
        source: &'src str,
        lineno: usize,
        symbol_name: String,
        expected: QasmType,
    },
    RegisterSizeMismatch {
        source: &'src str,
        lineno: usize,
        symbol_name: String,
        sizes: Vec<usize>,
    },
}

pub type Result<'src, T> = std::result::Result<T, QasmSimError<'src>>;

impl fmt::Display for QasmSimError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buffer = String::new();
        humanize_error(&mut buffer, self)?;
        write!(f, "{}", buffer)
    }
}

impl error::Error for QasmSimError<'_> {}

impl convert::From<String> for QasmSimError<'_> {
    fn from(err: String) -> Self {
        QasmSimError::UnknownError(err)
    }
}

impl<'src> From<SrcAndErr<'src, ParseError>> for QasmSimError<'src> {
    fn from(src_and_err: SrcAndErr<'src, ParseError>) -> Self {
        let (input, error) = src_and_err;
        match error {
            ParseError::InvalidToken { location } => {
                let (source, lineno, startpos, endpos) = extract_line(location.0, None, input);
                QasmSimError::InvalidToken {
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
                QasmSimError::UnexpectedEOF {
                    source,
                    lineno,
                    startpos,
                    endpos,
                    token: None,
                    expected,
                }
            }
            ParseError::UnrecognizedToken { token, expected } => {
                let location = token.0;
                let endlocation = token.2;
                let (source, lineno, startpos, endpos) =
                    extract_line(location.0, Some(endlocation.0), input);
                QasmSimError::UnexpectedToken {
                    source,
                    lineno,
                    startpos,
                    endpos,
                    token: Some(token.1),
                    expected,
                }
            }
            ParseError::ExtraToken { token } => {
                let location = token.0;
                let endlocation = token.2;
                let (source, lineno, startpos, endpos) =
                    extract_line(location.0, Some(endlocation.0), input);
                QasmSimError::UnexpectedToken {
                    source,
                    lineno,
                    startpos,
                    endpos,
                    token: Some(token.1),
                    expected: Vec::new(),
                }
            }
            ParseError::User { error: lexer_error } => {
                let location = lexer_error.location;
                let (source, lineno, startpos, endpos) = extract_line(location.0, None, input);
                QasmSimError::InvalidToken {
                    // XXX: Actually, this should be "InvalidInput"
                    source,
                    lineno,
                    startpos,
                    endpos,
                    token: None,
                    expected: Vec::new(),
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
            RuntimeError::RegisterSizeMismatch {
                location,
                symbol_name,
                sizes,
            } => {
                let (source, lineno, _, _) = extract_line(location.0, None, input);
                QasmSimError::RegisterSizeMismatch {
                    source,
                    lineno,
                    symbol_name,
                    sizes,
                }
            }
            RuntimeError::TypeMismatch {
                location,
                symbol_name,
                expected,
            } => {
                let (source, lineno, _, _) = extract_line(location.0, None, input);
                QasmSimError::TypeMismatch {
                    source,
                    lineno,
                    symbol_name,
                    expected,
                }
            }
            RuntimeError::UndefinedGate {
                location,
                symbol_name,
            } => {
                let (source, lineno, _, _) = extract_line(location.0, None, input);
                QasmSimError::UndefinedGate {
                    source,
                    lineno,
                    symbol_name,
                }
            }
            RuntimeError::WrongNumberOfParameters {
                are_registers,
                location,
                symbol_name,
                given,
                expected,
            } => {
                let (source, lineno, _, _) = extract_line(location.0, None, input);
                QasmSimError::WrongNumberOfParameters {
                    are_registers,
                    source,
                    symbol_name,
                    lineno,
                    expected,
                    given,
                }
            }
            RuntimeError::SymbolNotFound {
                location,
                symbol_name,
                expected,
            } => {
                let (source, lineno, _, _) = extract_line(location.0, None, input);
                QasmSimError::SymbolNotFound {
                    source,
                    symbol_name,
                    lineno,
                    expected,
                }
            }
            RuntimeError::IndexOutOfBounds {
                location,
                symbol_name,
                index,
                size,
            } => {
                let (source, lineno, _, _) = extract_line(location.0, None, input);
                QasmSimError::IndexOutOfBounds {
                    source,
                    symbol_name,
                    lineno,
                    size,
                    index,
                }
            }
            RuntimeError::SemanticError(semantic_error) => match semantic_error {
                SemanticError::RedefinitionError {
                    symbol_name,
                    location,
                    previous_location,
                } => {
                    let (source, lineno, _, _) = extract_line(location.0, None, input);
                    let (_, previous_lineno, _, _) = extract_line(previous_location.0, None, input);
                    QasmSimError::RedefinitionError {
                        source,
                        symbol_name,
                        lineno,
                        previous_lineno,
                    }
                }
            },
        }
    }
}

impl<'src> From<SrcAndErr<'src, LinkerError>> for QasmSimError<'src> {
    fn from(source_and_error: SrcAndErr<'src, LinkerError>) -> Self {
        let (input, error) = source_and_error;
        match error {
            LinkerError::LibraryNotFound { location, libpath } => {
                let (source, lineno, _, _) = extract_line(location.0, None, input);
                QasmSimError::LibraryNotFound {
                    source,
                    libpath,
                    lineno,
                }
            }
        }
    }
}

fn extract_line(
    offset: usize,
    endoffset: Option<usize>,
    doc: &str,
) -> (&str, usize, usize, Option<usize>) {
    assert!(
        offset <= doc.len(),
        "linestart={} must in the range 0..=doc.len()={}",
        offset,
        doc.len()
    );

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
            if character == '\n' {
                break;
            }
        }
    }

    if end < start {
        end = doc.len();
    }

    let startpos = offset - start;
    let endpos = match endoffset {
        None => None,
        Some(endoffset) => Some(endoffset - start),
    };

    (&doc[start..end], linecount, startpos, endpos)
}

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
