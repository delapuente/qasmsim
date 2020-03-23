#![cfg(test)]

extern crate qasmsim;
extern crate indoc;

use indoc::indoc;
use qasmsim::{ QasmSimError, ErrorKind };
use qasmsim::grammar::Tok;

#[test]
fn test_missing_semicolon_at_eof() {
  let source = indoc!("
    OPENQASM 2.0;
    qreg q[10]");
  let err = qasmsim::run(&source).unwrap_err();
  assert_eq!(err, QasmSimError::SyntaxError {
    kind: ErrorKind::UnexpectedEOF,
    source: &source,
    lineoffset: 14,
    lineno: 2,
    startpos: 10,
    endpos: None,
    token: None,
    expected: vec!["\";\"".into()]
  });
}

#[test]
fn test_missing_semicolon_almost_at_eof() {
  let source = indoc!("
    OPENQASM 2.0;
    qreg q[10]
  ");
  let err = qasmsim::run(&source).unwrap_err();
  assert_eq!(err, QasmSimError::SyntaxError {
    kind: ErrorKind::UnexpectedEOF,
    source: &source,
    lineoffset: 14,
    lineno: 2,
    startpos: 10,
    endpos: None,
    token: None,
    expected: vec!["\";\"".into()]
  });
}

#[test]
fn test_missing_semicolon_between_two_instructions() {
  let source = indoc!("
    OPENQASM 2.0;
    qreg q[10]
    qreg r[10];
  ");
  let err = qasmsim::run(&source).unwrap_err();
  assert_eq!(err, QasmSimError::SyntaxError {
    kind: ErrorKind::UnexpectedToken,
    source: &source,
    lineoffset: 25,
    lineno: 3,
    startpos: 0,
    endpos: Some(4),
    token: Some(Tok::QReg),
    expected: vec!["\";\"".into()]
  });
}

#[test]
fn test_missing_bracket() {
  let source = indoc!("
    OPENQASM 2.0;
    qreg q[10;
  ");
  let err = qasmsim::run(&source).unwrap_err();
  assert_eq!(err, QasmSimError::SyntaxError {
    kind: ErrorKind::UnexpectedToken,
    source: &source,
    lineoffset: 14,
    lineno: 2,
    startpos: 9,
    endpos: Some(10),
    token: Some(Tok::Semi),
    expected: vec!["\"]\"".into()]
  });
}

#[test]
fn test_missing_openqasm_header() {
  let source = indoc!("
    qreg q[10];
  ");
  let err = qasmsim::run(&source).unwrap_err();
  assert_eq!(err, QasmSimError::SyntaxError {
    kind: ErrorKind::UnexpectedToken,
    source: &source,
    lineoffset: 0,
    lineno: 1,
    startpos: 0,
    endpos: Some(4),
    token: Some(Tok::QReg),
    expected: vec!["\"OPENQASM\"".into()]
  });
}

#[test]
fn test_misspelling_openqasm_header() {
  let source = indoc!("
    OEPNQASM 2.0;
    qreg q[10];
  ");
  let err = qasmsim::run(&source).unwrap_err();
  assert_eq!(err, QasmSimError::SyntaxError {
    kind: ErrorKind::InvalidToken,
    source: &source,
    lineoffset: 0,
    lineno: 1,
    startpos: 0,
    endpos: None,
    token: None,
    expected: Vec::new()
  });
}

#[test]
fn test_missing_openqasm_version() {
  let source = indoc!("
    OPENQASM;
    qreg q[10];
  ");
  let err = qasmsim::run(&source).unwrap_err();
  assert_eq!(err, QasmSimError::SyntaxError {
    kind: ErrorKind::UnexpectedToken,
    source: &source,
    lineoffset: 0,
    lineno: 1,
    startpos: 8,
    endpos: Some(9),
    token: Some(Tok::Semi),
    expected: vec!["version".into()]
  });
}

#[test]
#[should_panic]
fn test_missing_arrow() {
  let source = indoc!("
    OPENQASM 2.0;
    qreg q[1];
    creg c[1];
    measure q c;
  ");
  // XXX: I have no idea why lalrpop is expecting something different than an
  // arrow here.
  let err = qasmsim::run(&source).unwrap_err();
  assert_eq!(err, QasmSimError::SyntaxError {
    kind: ErrorKind::UnexpectedToken,
    source: &source,
    lineoffset: 36,
    lineno: 4,
    startpos: 10,
    endpos: Some(11),
    token: Some(Tok::Id { repr: "c".into() }),
    expected: vec!["\"->\"".into()]
  });
}