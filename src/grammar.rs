use lalrpop_util::{self, lalrpop_mod};

pub mod ast;
pub mod lexer;
lalrpop_mod!(#[allow(clippy::all)] pub open_qasm2, "/grammar/open_qasm2.rs");

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::grammar::lexer::Lexer;
    use crate::grammar::open_qasm2;
    use crate::grammar::{ast::*, lexer::Location};

    macro_rules! span {
        ($left:expr, $node:expr, $right:expr) => {
            Span {
                boundaries: (Location($left), Location($right)),
                node: Box::new($node),
            }
        };
    }

    #[test]
    fn test_parse_open_qasm() {
        let source = indoc!(
            "
    OPENQASM 2.0;
    qreg q[2];
    creg c[2];
    "
        );
        let lexer = Lexer::new(source);
        let parser = open_qasm2::OpenQasmProgramParser::new();
        let tree = parser.parse(lexer).unwrap();
        assert_eq!(
            tree,
            OpenQasmProgram {
                version: "2.0".to_string(),
                program: vec![
                    span!(14, Statement::QRegDecl("q".to_string(), 2), 24),
                    span!(25, Statement::CRegDecl("c".to_string(), 2), 35)
                ]
            }
        );
    }

    #[test]
    fn test_parse_id_gate_macro() {
        let source = "
    gate id q {}
    ";
        let lexer = Lexer::new(source);
        let parser = open_qasm2::StatementParser::new();
        let tree = parser.parse(lexer).unwrap();
        assert_eq!(
            tree,
            Statement::GateDecl("id".to_string(), vec![], vec!["q".to_string()], vec![])
        );
    }

    #[test]
    fn test_parse_id_gate_macro_with_parenthesis() {
        let source = "
    gate id () q {}
    ";
        let lexer = Lexer::new(source);
        let parser = open_qasm2::StatementParser::new();
        let tree = parser.parse(lexer).unwrap();
        assert_eq!(
            tree,
            Statement::GateDecl("id".to_string(), vec![], vec!["q".to_string()], vec![])
        );
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
        assert_eq!(
            tree,
            Statement::GateDecl(
                "cx".to_string(),
                vec![],
                vec!["c".to_string(), "t".to_string()],
                vec![GateOperation::Unitary(UnitaryOperation(
                    "CX".to_owned(),
                    vec![],
                    vec![Argument::Id("c".to_owned()), Argument::Id("t".to_owned())]
                ))]
            )
        );
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
        assert_eq!(
            tree,
            Statement::GateDecl(
                "u".to_string(),
                vec!["theta".to_string(), "phi".to_string(), "lambda".to_string()],
                vec!["q".to_string()],
                vec![GateOperation::Unitary(UnitaryOperation(
                    "U".to_owned(),
                    vec![
                        Expression::Id("theta".to_owned()),
                        Expression::Id("phi".to_owned()),
                        Expression::Id("lambda".to_owned()),
                    ],
                    vec![Argument::Id("q".to_owned())]
                ))]
            )
        );
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
        assert_eq!(
            tree,
            Statement::GateDecl(
                "rz".to_string(),
                vec!["phi".to_string()],
                vec!["a".to_string()],
                vec![GateOperation::Unitary(UnitaryOperation(
                    "u1".to_string(),
                    vec![Expression::Id("phi".to_string())],
                    vec![Argument::Id("a".to_string())]
                ))]
            )
        );
    }

    #[test]
    fn test_parse_expressions_in_arguments() {
        let source = "
    U(pi/2, 0, pi) q;
    ";
        let lexer = Lexer::new(source);
        let parser = open_qasm2::StatementParser::new();
        let tree = parser.parse(lexer).unwrap();
        assert_eq!(
            tree,
            Statement::QuantumOperation(QuantumOperation::Unitary(UnitaryOperation(
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
            )))
        );
    }

    #[test]
    fn test_operator_precedence() {
        let source = "
    -pi + (1 - 2) * 3 / 4
    ";
        let lexer = Lexer::new(source);
        let parser = open_qasm2::ExprParser::new();
        let tree = parser.parse(lexer).unwrap();
        assert_eq!(
            tree,
            Expression::Op(
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
            )
        );
    }

    #[test]
    fn test_parse_program_without_version_string() {
        let source = indoc!(
            "
    qreg q[1];
    creg c[1];
    h q;
    "
        );
        let lexer = Lexer::new(source);
        let parser = open_qasm2::ProgramBodyParser::new();
        let tree = parser.parse(lexer).unwrap();
        assert_eq!(
            tree,
            vec![
                span!(0, Statement::QRegDecl("q".to_string(), 1), 10),
                span!(11, Statement::CRegDecl("c".to_string(), 1), 21),
                span!(
                    22,
                    Statement::QuantumOperation(QuantumOperation::Unitary(UnitaryOperation(
                        "h".to_string(),
                        vec![],
                        vec![Argument::Id("q".to_string())]
                    ))),
                    26
                )
            ]
        );
    }

    #[test]
    fn test_program_with_measure_and_reset() {
        let source = indoc!(
            "
    qreg q[1];
    creg c[1];
    h q;
    measure q -> c;
    reset q;
    "
        );
        let lexer = Lexer::new(source);
        let parser = open_qasm2::ProgramBodyParser::new();
        let tree = parser.parse(lexer).unwrap();
        assert_eq!(
            tree,
            vec![
                span!(0, Statement::QRegDecl("q".to_string(), 1), 10),
                span!(11, Statement::CRegDecl("c".to_string(), 1), 21),
                span!(
                    22,
                    Statement::QuantumOperation(QuantumOperation::Unitary(UnitaryOperation(
                        "h".to_string(),
                        vec![],
                        vec![Argument::Id("q".to_string())]
                    ))),
                    26
                ),
                span!(
                    27,
                    Statement::QuantumOperation(QuantumOperation::Measure(
                        Argument::Id("q".to_string()),
                        Argument::Id("c".to_string())
                    )),
                    42
                ),
                span!(
                    43,
                    Statement::QuantumOperation(QuantumOperation::Reset(Argument::Id(
                        "q".to_string()
                    ))),
                    51
                )
            ]
        );
    }

    #[test]
    fn test_comments() {
        let source = indoc!(
            "
    // Comment 1
    OPENQASM 2.0;
    // Comment 2

    // Comment 3
    gate id q {} // Comment 4
    // Comment 5
    "
        );
        let lexer = Lexer::new(source);
        let parser = open_qasm2::OpenQasmProgramParser::new();
        let tree = parser.parse(lexer).unwrap();
        assert_eq!(
            tree,
            OpenQasmProgram {
                version: "2.0".to_string(),
                program: vec![span!(
                    54,
                    Statement::GateDecl(
                        String::from("id"),
                        vec![],
                        vec![String::from("q")],
                        vec![]
                    ),
                    66
                ),]
            }
        );
    }

    #[test]
    fn test_conditional_application() {
        let source = "
    if (c==5) cx c, t;
    ";
        let lexer = Lexer::new(source);
        let parser = open_qasm2::StatementParser::new();
        let tree = parser.parse(lexer).unwrap();
        assert_eq!(
            tree,
            Statement::Conditional(
                Argument::Id(String::from("c")),
                5_u64,
                QuantumOperation::Unitary(UnitaryOperation(
                    String::from("cx"),
                    vec![],
                    vec![
                        Argument::Id(String::from("c")),
                        Argument::Id(String::from("t"))
                    ]
                ))
            )
        );
    }
}
