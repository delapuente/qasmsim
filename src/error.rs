//! Contain the error types ragarding the different tasks that QasmSim can
//! perform.
mod humanize;

use std::convert;
use std::error;
use std::fmt;

use crate::grammar::lexer::{self, Location, Tok};
pub use crate::interpreter::runtime::RuntimeError;
pub use crate::linker::LinkerError;
use crate::semantics::QasmType;
pub use crate::semantics::SemanticError;
use self::humanize::humanize_error;

/// Represent a parsing error.
pub type ParseError = lalrpop_util::ParseError<Location, lexer::Tok, lexer::LexicalError<Location>>;

/// An alias for a pair relating some source code with an error.
pub type SrcAndErr<'src, E> = (&'src str, E);

/// Types of errors in QasmSim. QasmSim errors contain information about
/// the error and the location in the source code where the error happens.
///
/// Conversion between [`ParseError`], [`RuntimeError`] and [`LinkerError`] is
/// possible thanks to the trait `From` is defined for the pair
/// `(&'source str, T)` (see alias [`SrcAndErr`]) for all the errors listed
/// above.
///
/// # Examples
///
/// The error type of [`simulate`] is [`RuntimeError`].
/// `RuntimeError` is a _sourceless_ error in the sense it does not relate with
/// the concrete source code beyond the location in the AST at which the error
/// happens.
///
/// You can use [`map_err`] for for capturing the error and converting it
/// into a `QasmSimError` from its pairing with the source.
///
/// ```
/// use qasmsim::{QasmSimError, compile_with_linker, default_linker, simulate};
///
/// let source = r#"
/// OPENQASM 2.0;
/// qreg q[2];
/// CX q[1], q[2]; // Notice we are indexing out of bounds here.
/// "#;
/// let program = compile_with_linker(source, default_linker())?;
/// let runtime_error = simulate(&program).expect_err("Index out of bounds");
/// let qasmsim_error = QasmSimError::from((source, runtime_error));
/// # Ok::<(), QasmSimError>(())
/// ```
///
/// [`ParseError`]: ./type.ParseError.html
/// [`RuntimeError`]: ./enum.RuntimeError.html
/// [`LinkerError`]: ../linker/enum.LinkerError.html
/// [`SrcAndErr`]: ./type.SrcAndErr.html
/// [`simulate]: ../fn.simulate.html
/// [`map_err`]: ../../std/result/enum.Result.html#method.map_err
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum QasmSimError<'src> {
    /// A generic unknown error.
    UnknownError(String),
    /// Found an invalid token at some position.
    InvalidToken {
        /// Line source.
        source: &'src str,
        /// Line number.
        lineno: usize,
        /// Position inside the line (0-based) where the invalid token starts.
        startpos: usize,
        /// Position inside the line (0-based) where the invalid token ends.
        endpos: Option<usize>,
        /// Token found.
        token: Option<Tok>,
        /// A list of expected tokens.
        expected: Vec<String>,
    },
    /// Found an unexpected end of file.
    UnexpectedEOF {
        /// Line source.
        source: &'src str,
        /// Line number.
        lineno: usize,
        /// Position inside the line (0-based) where the invalid token starts.
        startpos: usize,
        /// Position inside the line (0-based) where the invalid token ends.
        endpos: Option<usize>,
        /// Token found.
        token: Option<Tok>,
        /// A list of expected tokens.
        expected: Vec<String>,
    },
    /// Found an unexpected token.
    UnexpectedToken {
        /// Line source.
        source: &'src str,
        /// Line number.
        lineno: usize,
        /// Position inside the line (0-based) where the invalid token starts.
        startpos: usize,
        /// Position inside the line (0-based) where the invalid token ends.
        endpos: Option<usize>,
        /// Token found.
        token: Option<Tok>,
        /// A list of expected tokens.
        expected: Vec<String>,
    },
    /// Found a redefinition of a register.
    RedefinitionError {
        /// Line source.
        source: &'src str,
        /// Name of the register declared for second time.
        symbol_name: String,
        /// Line number.
        lineno: usize,
        /// Line number where the register was originally declared.
        previous_lineno: usize,
    },
    /// The unability of linking a library.
    LibraryNotFound {
        /// Line source.
        source: &'src str,
        /// Path to the library to be included.
        libpath: String,
        /// Line number.
        lineno: usize,
    },
    /// Use of register index that does not fit the register size.
    IndexOutOfBounds {
        /// Line source.
        source: &'src str,
        /// Line number.
        lineno: usize,
        /// Name of the register being indexed.
        symbol_name: String,
        /// Index tried to access.
        index: usize,
        /// Size of the register.
        size: usize,
    },
    /// Use of an unknown/undeclared symbol.
    SymbolNotFound {
        /// Line source.
        source: &'src str,
        /// Line number.
        lineno: usize,
        /// Name of the unknown symbol.
        symbol_name: String,
        /// The expected type.
        expected: QasmType,
    },
    /// The attempt of applying an operation passing the wrong number of
    /// parameters.
    WrongNumberOfParameters {
        /// Line source.
        source: &'src str,
        /// Line number.
        lineno: usize,
        /// Name of the operation.
        symbol_name: String,
        /// Indicate if the parameters are registers or real values.
        are_registers: bool,
        /// The number of passed parameters.
        given: usize,
        /// The number of expected parameters.
        expected: usize,
    },
    /// Use of a gate not previously defined.
    UndefinedGate {
        /// Line source.
        source: &'src str,
        /// Line number.
        lineno: usize,
        /// Name of the unknown gate.
        symbol_name: String,
    },
    /// Found an unexpected type of value.
    TypeMismatch {
        /// Line source.
        source: &'src str,
        /// Line number.
        lineno: usize,
        /// Name of the symbol with the incorrect type.
        symbol_name: String,
        /// Expected type.
        expected: QasmType,
    },
    /// Attempt of applying an operation to different sizes registers.
    RegisterSizeMismatch {
        /// Line source.
        source: &'src str,
        /// Line number.
        lineno: usize,
        /// Name of the operation.
        symbol_name: String,
        /// Sizes of the different registers involved.
        sizes: Vec<usize>,
    },
}

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
