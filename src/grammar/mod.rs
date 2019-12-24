pub mod ast;

#[cfg(test)]
mod tests {
  use super::*;
  use open_qasm2;

  #[test]
  fn test_parse_open_qasm() {
    let source = "
  OPENQASM 2.0;
  qreg q[2];
  creg c[2];
  ";
    let parser = open_qasm2::OpenQasmProgramParser::new();
    let tree = parser.parse(source).unwrap();
    assert_eq!(tree, Box::new(ast::OpenQasmProgram{
      version: "2.0".to_string(),
      program: vec![
        ast::Statement::QRegDecl("q".to_string(), 2),
        ast::Statement::CRegDecl("c".to_string(), 2)
      ]
    }));
  }

  #[test]
  fn test_parse_id_gate_macro() {
    let source = "
  gate id q {}
  ";
    let parser = open_qasm2::StatementParser::new();
    let tree = parser.parse(source).unwrap();
    assert_eq!(tree, ast::Statement::GateDecl(
      "id".to_string(), vec![], vec!["q".to_string()], vec![]
    ));
  }

  #[test]
  fn test_parse_id_gate_macro_with_parenthesis() {
    let source = "
  gate id () q {}
  ";
    let parser = open_qasm2::StatementParser::new();
    let tree = parser.parse(source).unwrap();
    assert_eq!(tree, ast::Statement::GateDecl(
      "id".to_string(), vec![], vec!["q".to_string()], vec![]
    ));
  }

  #[test]
  fn test_parse_cx_gate_macro() {
    let source = "
  gate cx c, t {
    CX c, t;
  }
  ";
    let parser = open_qasm2::StatementParser::new();
    let tree = parser.parse(source).unwrap();
    assert_eq!(tree, ast::Statement::GateDecl(
      "cx".to_string(), vec![], vec!["c".to_string(), "t".to_string()], vec![
        ast::GateOperation::Unitary(ast::UnitaryOperation::CX(
          ast::Argument::Id("c".to_string()),
          ast::Argument::Id("t".to_string())
        ))
      ]
    ));
  }

  #[test]
  fn test_parse_u_gate_macro() {
    let source = "
  gate u (theta, phi, lambda) q {
    U (theta, phi, lambda) q;
  }
  ";
    let parser = open_qasm2::StatementParser::new();
    let tree = parser.parse(source).unwrap();
    assert_eq!(tree, ast::Statement::GateDecl(
      "u".to_string(),
      vec!["theta".to_string(), "phi".to_string(), "lambda".to_string()],
      vec!["q".to_string()],
      vec![
        ast::GateOperation::Unitary(ast::UnitaryOperation::U(
          ast::Expression::Id("theta".to_string()),
          ast::Expression::Id("phi".to_string()),
          ast::Expression::Id("lambda".to_string()),
          ast::Argument::Id("q".to_string())
        ))
      ]
    ));
  }

  #[test]
  fn test_parse_gate_macro_with_gate_expansion() {
    let source = "
  gate rz (phi) a {
    u1 (phi) a;
  }
  ";
    let parser = open_qasm2::StatementParser::new();
    let tree = parser.parse(source).unwrap();
    assert_eq!(tree, ast::Statement::GateDecl(
      "rz".to_string(),
      vec!["phi".to_string()],
      vec!["a".to_string()],
      vec![
        ast::GateOperation::Unitary(ast::UnitaryOperation::GateExpansion(
          "u1".to_string(),
          vec![ast::Expression::Id("phi".to_string())],
          vec![ast::Argument::Id("a".to_string())]
        ))
      ]
    ));
  }

  #[test]
  fn test_parse_program_without_version_string() {
    let source = "
  qreg q[1];
  creg c[1];
  h q;
  ";
    let parser = open_qasm2::ProgramParser::new();
    let tree = parser.parse(source).unwrap();
    assert_eq!(tree, vec![
      ast::Statement::QRegDecl("q".to_string(), 1),
      ast::Statement::CRegDecl("c".to_string(), 1),
      ast::Statement::QuantumOperation(
        ast::QuantumOperation::Unitary(
          ast::UnitaryOperation::GateExpansion(
            "h".to_string(), vec![], vec![ast::Argument::Id("q".to_string())])
        )
      )
    ]);
  }

  #[test]
  fn test_program_with_measure_and_reset() {
    let source = "
  qreg q[1];
  creg c[1];
  h q;
  measure q -> c;
  reset q;
  ";
    let parser = open_qasm2::ProgramParser::new();
    let tree = parser.parse(source).unwrap();
    assert_eq!(tree, vec![
      ast::Statement::QRegDecl("q".to_string(), 1),
      ast::Statement::CRegDecl("c".to_string(), 1),
      ast::Statement::QuantumOperation(
        ast::QuantumOperation::Unitary(
          ast::UnitaryOperation::GateExpansion(
            "h".to_string(), vec![], vec![ast::Argument::Id("q".to_string())])
        )
      ),
      ast::Statement::QuantumOperation(
        ast::QuantumOperation::Measure(
          ast::Argument::Id("q".to_string()),
          ast::Argument::Id("c".to_string())
        )
      ),
      ast::Statement::QuantumOperation(
        ast::QuantumOperation::Reset(ast::Argument::Id("q".to_string()))
      )
    ]);
  }
}