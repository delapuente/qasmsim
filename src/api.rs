use std::collections::HashMap;
use std::iter::FromIterator;

use crate::error::QasmSimError;
use crate::grammar::open_qasm2;
use crate::grammar::{ast, lexer::Lexer};
use crate::interpreter;
use crate::linker::Linker;
use crate::qe;

pub type Result<'src, T> = std::result::Result<T, QasmSimError<'src>>;

/// Return the default linker which includes the [`qelib1.inc`] library.
///
/// [`qelib1.inc`]: https://github.com/Qiskit/openqasm/blob/master/examples/generic/qelib1.inc
pub fn default_linker() -> Linker {
    Linker::with_embedded(HashMap::from_iter(vec![(
        "qelib1.inc".to_owned(),
        qe::QELIB1.to_owned(),
    )]))
}

pub fn compile(input: &str) -> Result<ast::OpenQasmProgram> {
    let lexer = Lexer::new(&input);
    let parser = open_qasm2::OpenQasmProgramParser::new();
    parser
        .parse(lexer)
        .map_err(|err| QasmSimError::from((input, err)))
}

/// Return the AST of `input` and link external sources with `linker`.
///
/// # Examples
///
/// ```
/// use qasmsim::{default_linker, compile_with_linker};
///
/// let ast = compile_with_linker(r#"
///     OPENQASM 2.0;
///     include "qelib1.inc";
///     qdef q[2];
///     h q[0];
///     cx q[0], q[1];
/// "#, default_linker())?;
/// ```
pub fn compile_with_linker(input: &str, linker: Linker) -> Result<'_, ast::OpenQasmProgram> {
    let program = compile(&input)?;
    linker
        .link(program)
        .map_err(|err| QasmSimError::from((input, err)))
}

pub use interpreter::runtime::execute;

pub use interpreter::runtime::execute_with_shots;
