#![cfg(test)]

extern crate qasmsim;
extern crate indoc;

use indoc::indoc;
use qasmsim::QasmSimError;

#[test]
fn test_missing_semicolon_at_eof() {
  let source = indoc!("
    OPENQASM 2.0;
    qreg q[10]");
  let err = qasmsim::run(&source).unwrap_err();
  assert_eq!(err, QasmSimError::SyntaxError {
    msg: r#"expected ";", found EOF"#.into(),
    lineno: 2,
    startpos: 10,
    endpos: None,
    linesrc: Some("qreg q[10]".into()),
    help: Some(r#"consider adding ";" here"#.into())
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
    msg: r#"expected ";", found EOF"#.into(),
    lineno: 2,
    startpos: 11,
    endpos: None,
    linesrc: Some("qreg q[10]\n".into()),
    help: Some(r#"consider adding ";" here"#.into())
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
    msg: r#"expected ";", found "keyword `qreg`""#.into(),
    lineno: 3,
    startpos: 0,
    endpos: Some(4),
    linesrc: Some("qreg r[10];\n".into()),
    help: Some(r#"consider adding ";" before this"#.into())
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
    msg: r#"expected "]", found ";""#.into(),
    lineno: 2,
    startpos: 9,
    endpos: Some(10),
    linesrc: Some("qreg q[10;\n".into()),
    help: Some(r#"consider adding "]" before this"#.into())
  });
}

#[test]
fn test_missing_openqasm_header() {
  let source = indoc!("
    qreg q[10];
  ");
  let err = qasmsim::run(&source).unwrap_err();
  assert_eq!(err, QasmSimError::SyntaxError {
    msg: r#"expected "OPENQASM", found "keyword `qreg`""#.into(),
    lineno: 1,
    startpos: 0,
    endpos: Some(4),
    linesrc: Some("qreg q[10];\n".into()),
    help: Some(r#"consider adding "OPENQASM" before this"#.into())
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
    msg: r#"invalid token"#.into(),
    lineno: 1,
    startpos: 0,
    endpos: None,
    linesrc: Some("OEPNQASM 2.0;\n".into()),
    help: None
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
    msg: r#"expected version, found ";""#.into(),
    lineno: 1,
    startpos: 8,
    endpos: Some(9),
    linesrc: Some("OPENQASM;\n".into()),
    help: Some(r#"consider adding version before this"#.into())
  });
}
