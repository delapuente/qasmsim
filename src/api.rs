use std::collections::HashMap;
use std::iter::FromIterator;

use crate::error::QasmSimError;
use crate::grammar::{ast, parse_program};
use crate::semantics;
use crate::interpreter;
use crate::interpreter::runtime::RuntimeError;
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

type GateSignature = (String, Vec<String>, Vec<String>);

/// Return the signature and documentation of the gate `gate_name` if it is
/// defined in the source code `input`.
///
/// # Errors
///
/// The function can fail if `gate_name` is an opaque gate, or if it is not
/// found in the source. In that case it will return an `Err` wrapping
/// a special case for the [`QasmSimError::UndefinedGate`] variant with
/// `source` set to `""`, `lineno` set to `0` and `symbol_name` set to
/// `gate_name`.
///
/// [`QasmSimError::UndefinedGate`]: ./error/enum.QasmSimError.html#variant.UndefinedGate
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use qasmsim::get_gate_info;
///
/// let (docstring, (name, real_params, quantum_params)) = get_gate_info(r#"
///     OPENQASM 2.0;
///     // 3-parameter 2-pulse single qubit gate
///     gate u3(theta,phi,lambda) q { U(theta,phi,lambda) q; }
/// "#, "u3")?;
///
/// assert_eq!(
///     docstring,
///     String::from(" 3-parameter 2-pulse single qubit gate\n")
/// );
///
/// assert_eq!(
///     name,
///     String::from("u3")
/// );
///
/// assert_eq!(
///     real_params,
///     vec![
///         String::from("theta"),
///         String::from("phi"),
///         String::from("lambda"),
///     ]
/// );
///
/// assert_eq!(
///     quantum_params,
///     vec![
///         String::from("q"),
///     ]
/// );
///
/// # use qasmsim::QasmSimError;
/// # Ok::<(), qasmsim::QasmSimError>(())
pub fn get_gate_info<'src>(
    input: &'src str,
    gate_name: &str
) -> Result<'src, (String, GateSignature)> {
    let linked = parse_and_link(input)?;
    // TODO: Implement conversion from SemanticError to QasmSimError directly
    // without converting to RuntimeError first.
    let semantics = semantics::extract_semantics(&linked)
        .map_err(
            |err| QasmSimError::from((input, RuntimeError::from(err)))
        )?;

    let docstring = semantics.symbol_docstrings
        .get(gate_name)
        .ok_or(QasmSimError::UndefinedGate {
            source: "",
            lineno: 0,
            symbol_name: String::from(gate_name),
        })?;

    let macro_def = semantics.macro_definitions
        .get(gate_name)
        .ok_or(QasmSimError::UndefinedGate {
            source: "",
            lineno: 0,
            symbol_name: String::from(gate_name),
        })?;

    Ok((
        docstring.to_string(),
        (macro_def.0.clone(), macro_def.1.clone(), macro_def.2.clone())
    ))
}

pub use interpreter::runtime::simulate;

pub use interpreter::runtime::simulate_with_shots;
