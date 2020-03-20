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
  ");
  let err = qasmsim::run(&source).unwrap_err();
  assert_eq!(err, QasmSimError::SyntaxError {
    msg: String::from("missing semicolon: `;`"),
    lineno: 2,
    linepos: 10,
    linesrc: String::from("qreg q[10]"),
    help: String::from("consider adding a semicolor: `;`")
  });
  // assert_eq!(err, indoc!("
  //   error: missing semicolon: `;`
  //     |
  //   2 |   qreg q[10]
  //     |             ^ help: consider adding a semicolon: `;`
  // "));
}