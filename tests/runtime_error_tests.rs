#![cfg(test)]

extern crate qasmsim;

use indoc::indoc;

use qasmsim::{ QasmSimError, RuntimeKind };

#[test]
fn test_calling_a_non_existing_gate() {
  let source = indoc!("
  OPENQASM 2.0;
  qreg q[2];
  xxx q;
  ");
  let error = qasmsim::run(source, None).expect_err("should fail");
  assert_eq!(error, QasmSimError::UndefinedGate {
    source: "xxx q;\n",
    symbol_name: "xxx".into(),
    lineno: 3
  });
}

#[test]
fn test_using_a_quantum_register_while_expecting_classical() {
  let source = "
  OPENQASM 2.0;
  qreg q[2];
  creg c[2];
  measure q -> q;
  ";
  let error = qasmsim::run(source, None).expect_err("should fail");
  assert_eq!(error, QasmSimError::RuntimeError {
    kind: RuntimeKind::ClassicalRegisterNotFound,
    symbol_name: "q".into()
  });
}

#[test]
fn test_using_a_classical_register_when_expecting_quantum() {
  let source = "
  OPENQASM 2.0;
  qreg q[2];
  creg c[2];
  measure c -> c;
  ";
  let error = qasmsim::run(source, None).expect_err("should fail");
  assert_eq!(error, QasmSimError::RuntimeError {
    kind: RuntimeKind::QuantumRegisterNotFound,
    symbol_name: "c".into()
  });
}

#[test]
fn test_passing_a_classical_register_when_expecting_quantum() {
  let source = r#"
  OPENQASM 2.0;
  include "qelib1.inc";
  creg c[2];
  h c;
  "#;
  let error = qasmsim::run(source, None).expect_err("should fail");
  assert_eq!(error, QasmSimError::RuntimeError {
    kind: RuntimeKind::QuantumRegisterNotFound,
    symbol_name: "c".into()
  });
}

#[test]
fn test_passing_an_unexistent_register() {
  let source = r#"
  OPENQASM 2.0;
  include "qelib1.inc";
  creg c[2];
  h t;
  "#;
  let error = qasmsim::run(source, None).expect_err("should fail");
  assert_eq!(error, QasmSimError::RuntimeError {
    kind: RuntimeKind::QuantumRegisterNotFound,
    symbol_name: "t".into()
  });
}

#[test]
fn test_passing_an_unexistent_real_parameter() {
  let source = indoc!(r#"
  OPENQASM 2.0;
  include "qelib1.inc";
  qreg q[2];
  u1(xxx) q;
  "#);
  let error = qasmsim::run(source, None).expect_err("should fail");
  assert_eq!(error, QasmSimError::SymbolNotFound {
    symbol_name: "xxx".into(),
    source: "u1(xxx) q;\n",
    lineno: 4
  });
}

#[test]
fn test_passing_a_register_instead_of_real_parameter() {
  let source = indoc!(r#"
  OPENQASM 2.0;
  include "qelib1.inc";
  qreg q[2];
  u1(q) q;
  "#);
  let error = qasmsim::run(source, None).expect_err("should fail");
  assert_eq!(error, QasmSimError::SymbolNotFound {
    symbol_name: "q".into(),
    source: "u1(q) q;\n",
    lineno: 4
  });
}

#[test]
fn test_pass_more_real_arguments_than_expected() {
  let source = indoc!(r#"
  OPENQASM 2.0;
  include "qelib1.inc";
  qreg q[2];
  u1(pi, pi, pi) q;
  "#);
  let error = qasmsim::run(source, None).expect_err("should fail");
  assert_eq!(error, QasmSimError::WrongNumberOfParameters {
    are_registers: false,
    source: "u1(pi, pi, pi) q;\n",
    symbol_name: "u1".into(),
    lineno: 4,
    expected: 1,
    given: 3
  });
}

#[test]
fn test_pass_more_registers_than_expected() {
  let source = indoc!(r#"
  OPENQASM 2.0;
  include "qelib1.inc";
  qreg q[2];
  u1(pi) q, q, q;
  "#);
  let error = qasmsim::run(source, None).expect_err("should fail");
  assert_eq!(error, QasmSimError::WrongNumberOfParameters {
    are_registers: true,
    source: "u1(pi) q, q, q;\n",
    symbol_name: "u1".into(),
    lineno: 4,
    expected: 1,
    given: 3
  });
}

#[test]
fn test_index_out_of_bounds() {
  let source = indoc!(r#"
  OPENQASM 2.0;
  include "qelib1.inc";
  qreg q[2];
  h q[3];
  "#);
  let error = qasmsim::run(source, None).expect_err("should fail");
  assert_eq!(error, QasmSimError::IndexOutOfBounds {
    source: "h q[3];\n",
    symbol_name: "q".into(),
    lineno: 4,
    size: 2,
    index: 3
  });
}

#[test]
fn test_argument_expansion_with_different_size_registers() {
  let source = r#"
  OPENQASM 2.0;
  include "qelib1.inc";
  qreg q[1];
  qreg r[2];
  cx q, r;
  "#;
  let error = qasmsim::run(source, None).expect_err("should fail");
  assert_eq!(error, QasmSimError::RuntimeError {
    kind: RuntimeKind::DifferentSizeRegisters,
    symbol_name: "q".into()
  });
}

#[test]
fn test_quantum_register_in_conditional() {
  let source = r#"
  OPENQASM 2.0;
  include "qelib1.inc";
  qreg q[2];
  creg c[2];
  if (q==3) h q;
  "#;
  let error = qasmsim::run(source, None).expect_err("should fail");
  assert_eq!(error, QasmSimError::RuntimeError {
    kind: RuntimeKind::ClassicalRegisterNotFound,
    symbol_name: "q".into()
  });
}

#[test]
fn test_non_existent_register_in_conditional() {
  let source = r#"
  OPENQASM 2.0;
  include "qelib1.inc";
  qreg q[2];
  creg c[2];
  if (d==3) h q;
  "#;
  let error = qasmsim::run(source, None).expect_err("should fail");
  assert_eq!(error, QasmSimError::RuntimeError {
    kind: RuntimeKind::ClassicalRegisterNotFound,
    symbol_name: "d".into()
  });
}

#[test]
fn test_include_non_existent_lib() {
  let source = indoc!(r#"
  OPENQASM 2.0;
  include "nonexist.inc";
  "#);
  let error = qasmsim::run(source, None).expect_err("should fail");
  assert_eq!(error, QasmSimError::LibraryNotFound {
    source: "include \"nonexist.inc\";\n",
    lineno: 2,
    libpath: "nonexist.inc".into()
  });
}
