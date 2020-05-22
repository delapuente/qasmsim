use std::collections::HashMap;
use std::iter::FromIterator;

use crate::error::QasmSimError;
use crate::grammar::{ast, parse_program};
use crate::interpreter;
use crate::linker::Linker;
use crate::qe;

pub type Result<'src, T> = std::result::Result<T, QasmSimError<'src>>;

/// Return the default linker which includes the [`qelib1.inc`] library.
///
/// [`qelib1.inc`]: https://github.com/Qiskit/openqasm/blob/master/examples/generic/qelib1.inc
fn default_linker() -> Linker {
    Linker::with_embedded(HashMap::from_iter(vec![(
        "qelib1.inc".to_owned(),
        qe::QELIB1.to_owned(),
    )]))
}

/// Return the AST of `input` and link external sources with `linker`.
///
/// # Errors
///
/// The function can fail if failing to parse the source code. In that case
/// it will return an `Err` variant with a value of [`QasmSimError`].
///
/// [`QasmSimError`]: ./error/enum.QasmSimError.html
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use qasmsim::parse_and_link;
///
/// let ast = parse_and_link(r#"
///     OPENQASM 2.0;
///     include "qelib1.inc";
///     qreg q[2];
///     h q[0];
///     cx q[0], q[1];
/// "#)?;
/// # use qasmsim::QasmSimError;
/// # Ok::<(), qasmsim::QasmSimError>(())
/// ```
pub fn parse_and_link(input: &str) -> Result<'_, ast::OpenQasmProgram> {
    let linker = default_linker();
    let program = parse_program(&input)?;
    linker
        .link(program)
        .map_err(|err| QasmSimError::from((input, err)))
}

pub use interpreter::runtime::simulate;

pub use interpreter::runtime::simulate_with_shots;
