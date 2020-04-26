#![cfg(test)]

extern crate indoc;
extern crate qasmsim;

use indoc::indoc;
use qasmsim::grammar::lexer::Tok;
use qasmsim::QasmSimError;

#[test]
fn test_missing_semicolon_at_eof() {
    let source = indoc!(
        "
    OPENQASM 2.0;
    qreg q[10]"
    );
    let err = qasmsim::run(&source, None).unwrap_err();
    assert_eq!(
        err,
        QasmSimError::UnexpectedEOF {
            source: "qreg q[10]",
            lineno: 2,
            startpos: 10,
            endpos: None,
            token: None,
            expected: vec!["\";\"".into()]
        }
    );
}

#[test]
fn test_missing_semicolon_almost_at_eof() {
    let source = indoc!(
        "
    OPENQASM 2.0;
    qreg q[10]
  "
    );
    let err = qasmsim::run(&source, None).unwrap_err();
    assert_eq!(
        err,
        QasmSimError::UnexpectedEOF {
            source: "qreg q[10]\n",
            lineno: 2,
            startpos: 10,
            endpos: None,
            token: None,
            expected: vec!["\";\"".into()]
        }
    );
}

#[test]
fn test_missing_semicolon_between_two_instructions() {
    let source = indoc!(
        "
    OPENQASM 2.0;
    qreg q[10]
    qreg r[10];
  "
    );
    let err = qasmsim::run(&source, None).unwrap_err();
    assert_eq!(
        err,
        QasmSimError::UnexpectedToken {
            source: "qreg r[10];\n",
            lineno: 3,
            startpos: 0,
            endpos: Some(4),
            token: Some(Tok::QReg),
            expected: vec!["\";\"".into()]
        }
    );
}

#[test]
fn test_missing_bracket() {
    let source = indoc!(
        "
    OPENQASM 2.0;
    qreg q[10;
  "
    );
    let err = qasmsim::run(&source, None).unwrap_err();
    assert_eq!(
        err,
        QasmSimError::UnexpectedToken {
            source: "qreg q[10;\n",
            lineno: 2,
            startpos: 9,
            endpos: Some(10),
            token: Some(Tok::Semi),
            expected: vec!["\"]\"".into()]
        }
    );
}

#[test]
fn test_missing_openqasm_header() {
    let source = indoc!(
        "
    qreg q[10];
  "
    );
    let err = qasmsim::run(&source, None).unwrap_err();
    assert_eq!(
        err,
        QasmSimError::UnexpectedToken {
            source: "qreg q[10];\n",
            lineno: 1,
            startpos: 0,
            endpos: Some(4),
            token: Some(Tok::QReg),
            expected: vec!["\"OPENQASM\"".into()]
        }
    );
}

#[test]
fn test_misspelling_openqasm_header() {
    let source = indoc!(
        "
    OEPNQASM 2.0;
    qreg q[10];
  "
    );
    let err = qasmsim::run(&source, None).unwrap_err();
    assert_eq!(
        err,
        QasmSimError::InvalidToken {
            source: "OEPNQASM 2.0;\n",
            lineno: 1,
            startpos: 0,
            endpos: None,
            token: None,
            expected: Vec::new()
        }
    );
}

#[test]
fn test_missing_openqasm_version() {
    let source = indoc!(
        "
    OPENQASM;
    qreg q[10];
  "
    );
    let err = qasmsim::run(&source, None).unwrap_err();
    assert_eq!(
        err,
        QasmSimError::UnexpectedToken {
            source: "OPENQASM;\n",
            lineno: 1,
            startpos: 8,
            endpos: Some(9),
            token: Some(Tok::Semi),
            expected: vec!["version".into()]
        }
    );
}

#[test]
#[should_panic]
fn test_missing_arrow() {
    let source = indoc!(
        "
    OPENQASM 2.0;
    qreg q[1];
    creg c[1];
    measure q c;
  "
    );
    // XXX: I have no idea why lalrpop is expecting something different than an
    // arrow here.
    let err = qasmsim::run(&source, None).unwrap_err();
    assert_eq!(
        err,
        QasmSimError::UnexpectedToken {
            source: "measure q c;\n",
            lineno: 4,
            startpos: 10,
            endpos: Some(11),
            token: Some(Tok::Id { repr: "c".into() }),
            expected: vec!["\"->\"".into()]
        }
    );
}
