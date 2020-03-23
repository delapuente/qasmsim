use lalrpop_util::{ self, lalrpop_mod };

mod lexer;
pub mod ast;
lalrpop_mod!(pub open_qasm2, "/grammar/open_qasm2.rs");

use std::convert;

use crate::error::{
  ErrorKind,
  QasmSimError
};

pub use lexer::{
   Lexer,
   Location,
   Tok,
   LexicalError
};

pub type ParseError =
  lalrpop_util::ParseError<Location, lexer::Tok, lexer::LexicalError<Location>>;

type ErrAndSrc<'src> = (ParseError, &'src str);

impl<'src> convert::From<ErrAndSrc<'src>> for QasmSimError<'src> {
  fn from(err_and_src: ErrAndSrc<'src>) -> Self {
    let (error, source) = err_and_src;
    match error {
      ParseError::InvalidToken { location } => {
        QasmSimError::SyntaxError {
          kind: ErrorKind::InvalidToken,
          source,
          lineoffset: location.lineoffset,
          lineno: location.lineno,
          startpos: location.linepos,
          endpos: None,
          token: None,
          expected: Vec::new(),
        }
      }
      ParseError::UnrecognizedEOF { location, expected } => {
        QasmSimError::SyntaxError {
          kind: ErrorKind::UnexpectedEOF,
          source,
          lineoffset: location.lineoffset,
          lineno: location.lineno,
          startpos: location.linepos,
          endpos: None,
          token: None,
          expected
        }
      }
      ParseError::UnrecognizedToken { token, expected } => {
        QasmSimError::SyntaxError {
          kind: ErrorKind::UnexpectedToken,
          source,
          lineoffset: token.0.lineoffset,
          lineno: token.0.lineno,
          startpos: token.0.linepos,
          endpos: Some(token.2.linepos),
          token: Some(token.1),
          expected
        }
      }
      ParseError::ExtraToken { token } => {
        QasmSimError::SyntaxError {
          kind: ErrorKind::UnexpectedToken,
          source,
          lineoffset: token.0.lineoffset,
          lineno: token.0.lineno,
          startpos: token.0.linepos,
          endpos: Some(token.2.linepos),
          token: Some(token.1),
          expected: Vec::new()
        }
      }
      ParseError::User { error: lexer_error } => {
        QasmSimError::SyntaxError {
          kind: ErrorKind::InvalidToken, // XXX: Actually, this should be "InvalidInput"
          source,
          lineoffset: lexer_error.location.lineoffset,
          lineno: lexer_error.location.lineno,
          startpos: lexer_error.location.linepos,
          endpos: None,
          token: None,
          expected: Vec::new()
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::grammar::ast::*;
  use crate::grammar::lexer::Lexer;
  use super::open_qasm2;

  #[test]
  fn test_parse_open_qasm() {
    let source = "
    OPENQASM 2.0;
    qreg q[2];
    creg c[2];
    ";
    let lexer = Lexer::new(source);
    let parser = open_qasm2::OpenQasmProgramParser::new();
    let tree = parser.parse(lexer).unwrap();
    assert_eq!(tree, OpenQasmProgram{
      version: "2.0".to_string(),
      program: vec![
        Statement::QRegDecl("q".to_string(), 2),
        Statement::CRegDecl("c".to_string(), 2)
      ]
    });
  }

  #[test]
  fn test_parse_id_gate_macro() {
    let source = "
    gate id q {}
    ";
    let lexer = Lexer::new(source);
    let parser = open_qasm2::StatementParser::new();
    let tree = parser.parse(lexer).unwrap();
    assert_eq!(tree, Statement::GateDecl(
      "id".to_string(), vec![], vec!["q".to_string()], vec![]
    ));
  }

  #[test]
  fn test_parse_id_gate_macro_with_parenthesis() {
    let source = "
    gate id () q {}
    ";
    let lexer = Lexer::new(source);
    let parser = open_qasm2::StatementParser::new();
    let tree = parser.parse(lexer).unwrap();
    assert_eq!(tree, Statement::GateDecl(
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
    let lexer = Lexer::new(source);
    let parser = open_qasm2::StatementParser::new();
    let tree = parser.parse(lexer).unwrap();
    assert_eq!(tree, Statement::GateDecl(
      "cx".to_string(), vec![], vec!["c".to_string(), "t".to_string()], vec![
        GateOperation::Unitary(UnitaryOperation(
          "CX".to_owned(),
          vec![],
          vec![
            Argument::Id("c".to_owned()),
            Argument::Id("t".to_owned())
          ]
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
    let lexer = Lexer::new(source);
    let parser = open_qasm2::StatementParser::new();
    let tree = parser.parse(lexer).unwrap();
    assert_eq!(tree, Statement::GateDecl(
      "u".to_string(),
      vec!["theta".to_string(), "phi".to_string(), "lambda".to_string()],
      vec!["q".to_string()],
      vec![
        GateOperation::Unitary(UnitaryOperation(
          "U".to_owned(),
          vec![
            Expression::Id("theta".to_owned()),
            Expression::Id("phi".to_owned()),
            Expression::Id("lambda".to_owned()),
          ],
          vec![Argument::Id("q".to_owned())]
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
    let lexer = Lexer::new(source);
    let parser = open_qasm2::StatementParser::new();
    let tree = parser.parse(lexer).unwrap();
    assert_eq!(tree, Statement::GateDecl(
      "rz".to_string(),
      vec!["phi".to_string()],
      vec!["a".to_string()],
      vec![
        GateOperation::Unitary(UnitaryOperation(
          "u1".to_string(),
          vec![Expression::Id("phi".to_string())],
          vec![Argument::Id("a".to_string())]
        ))
      ]
    ));
  }

  #[test]
  fn test_parse_expressions_in_arguments() {
    let source = "
    U(pi/2, 0, pi) q;
    ";
    let lexer = Lexer::new(source);
    let parser = open_qasm2::StatementParser::new();
    let tree = parser.parse(lexer).unwrap();
    assert_eq!(tree, Statement::QuantumOperation(
      QuantumOperation::Unitary(
        UnitaryOperation(
          "U".to_owned(),
          vec![
            Expression::Op(
              Opcode::Div,
              Box::new(Expression::Pi),
              Box::new(Expression::Real(2.0))
            ),
            Expression::Real(0.0),
            Expression::Pi,
          ],
          vec![Argument::Id("q".to_owned())]
        )
      )
    ));
  }

  #[test]
  fn test_operator_precedence() {
    let source = "
    -pi + (1 - 2) * 3 / 4
    ";
    let lexer = Lexer::new(source);
    let parser = open_qasm2::ExprParser::new();
    let tree = parser.parse(lexer).unwrap();
    assert_eq!(tree, Expression::Op(
      Opcode::Add,
      Box::new(Expression::Minus(Box::new(Expression::Pi))),
      Box::new(Expression::Op(
        Opcode::Div,
        Box::new(Expression::Op(
          Opcode::Mul,
          Box::new(Expression::Op(
            Opcode::Sub,
            Box::new(Expression::Real(1.0)),
            Box::new(Expression::Real(2.0))
          )),
          Box::new(Expression::Real(3.0))
        )),
        Box::new(Expression::Real(4.0))
      ))
    ));
  }

  #[test]
  fn test_parse_program_without_version_string() {
    let source = "
    qreg q[1];
    creg c[1];
    h q;
    ";
    let lexer = Lexer::new(source);
    let parser = open_qasm2::ProgramParser::new();
    let tree = parser.parse(lexer).unwrap();
    assert_eq!(tree, vec![
      Statement::QRegDecl("q".to_string(), 1),
      Statement::CRegDecl("c".to_string(), 1),
      Statement::QuantumOperation(
        QuantumOperation::Unitary(
          UnitaryOperation(
            "h".to_string(), vec![], vec![Argument::Id("q".to_string())])
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
    let lexer = Lexer::new(source);
    let parser = open_qasm2::ProgramParser::new();
    let tree = parser.parse(lexer).unwrap();
    assert_eq!(tree, vec![
      Statement::QRegDecl("q".to_string(), 1),
      Statement::CRegDecl("c".to_string(), 1),
      Statement::QuantumOperation(
        QuantumOperation::Unitary(
          UnitaryOperation(
            "h".to_string(), vec![], vec![Argument::Id("q".to_string())])
        )
      ),
      Statement::QuantumOperation(
        QuantumOperation::Measure(
          Argument::Id("q".to_string()),
          Argument::Id("c".to_string())
        )
      ),
      Statement::QuantumOperation(
        QuantumOperation::Reset(Argument::Id("q".to_string()))
      )
    ]);
  }

  #[test]
  fn test_comments() {
    let source = "
    // Comment 1
    OPENQASM 2.0;
    // Comment 2

    // Comment 3
    gate id q {} // Comment 4
    // Comment 5
    ";
    let lexer = Lexer::new(source);
    let parser = open_qasm2::OpenQasmProgramParser::new();
    let tree = parser.parse(lexer).unwrap();
    assert_eq!(tree, OpenQasmProgram{
      version: "2.0".to_string(),
      program: vec![
        Statement::GateDecl(String::from("id"), vec![], vec![String::from("q")], vec![]),
      ]
    });
  }

  #[test]
  fn test_conditional_application() {
    let source = "
    if (c==5) cx c, t;
    ";
    let lexer = Lexer::new(source);
    let parser = open_qasm2::StatementParser::new();
    let tree = parser.parse(lexer).unwrap();
    assert_eq!(tree, Statement::Conditional(
      Argument::Id(String::from("c")),
      5_u64,
      QuantumOperation::Unitary(
        UnitaryOperation(
          String::from("cx"),
          vec![],
          vec![
            Argument::Id(String::from("c")),
            Argument::Id(String::from("t"))
          ]
        )
      )
    ));
  }
}