use std::collections::HashMap;
use std::iter::FromIterator;

use crate::error::QasmSimError;
use crate::grammar::open_qasm2;
use crate::grammar::{ast, lexer::Lexer};
use crate::interpreter;
use crate::linker::Linker;
use crate::qe;

pub type Result<'src, T> = std::result::Result<T, QasmSimError<'src>>;

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

pub fn compile_with_linker(input: &str, linker: Linker) -> Result<'_, ast::OpenQasmProgram> {
    let program = compile(&input)?;
    linker
        .link(program)
        .map_err(|err| QasmSimError::from((input, err)))
}

pub use interpreter::runtime::execute;

pub use interpreter::runtime::execute_with_shots;
