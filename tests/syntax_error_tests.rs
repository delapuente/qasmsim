#![cfg(test)]

extern crate qasmsim;
extern crate indoc;

use indoc::indoc;
use qasmsim::QasmSimError;

#[test]
fn test_missing_semicolon() {
  let source = indoc!("
    OPENQASM 2.0;
    qreg q[10]
    qreg r[10]
  ");
  let err = qasmsim::run(&source).unwrap_err();
  assert_eq!(err, QasmSimError::SyntaxError {
    msg: "missing semicolon: `;`".into(),
    lineno: 2,
    startpos: 10,
    endpos: None,
    linesrc: Some("qreg q[10]".into()),
    help: Some("consider adding a semicolon: `;`".into())
  });
}