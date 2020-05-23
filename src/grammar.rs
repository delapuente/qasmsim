//! Contain the machinery for parsing and processing OPENQASM programs. This
//! module is **unstable**.
//!
//! # Notes
//!
//! Be warned: the API of this module can change from release to release
//! independently of the semantic versioning of the rest of the library.
//!
//! The module would remain in the public API for enabling the users to perform
//! code manipulations at the abstract level.

use lalrpop_util::{self, lalrpop_mod};

pub mod ast;
pub mod lexer;
lalrpop_mod!(
    #[allow(clippy::all)]
    open_qasm2,
    "/grammar/open_qasm2.rs"
);

use self::ast::{Expression, OpenQasmLibrary, OpenQasmProgram, Span, Statement};
use self::lexer::Lexer;
use crate::error::QasmSimError;

macro_rules! parse_functions {
    ($($(#[$attr:meta])* $vis:vis fn $funcname:ident ($param:ident) -> $rettype:ty => $parser:ty);*) => {
        $(
            $(#[$attr])* $vis fn $funcname(
                $param: &str
            ) -> Result<$rettype, QasmSimError> {
                let lexer = Lexer::new($param);
                let parser = <$parser>::new();
                parser.parse(lexer).map_err(|err| ($param, err).into())
            }
        )*
    };
}

parse_functions! {
    /// Parse `source` into a [`Expression`] AST.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use qasmsim::{QasmSimError, grammar::parse_expression};
    /// use qasmsim::grammar::ast::{Expression, OpCode};
    ///
    /// let expr_ast = parse_expression("pi/2")?;
    ///
    /// assert_eq!(expr_ast, Expression::Op(
    ///     OpCode::Div,
    ///     Box::new(Expression::Pi),
    ///     Box::new(Expression::Real(2.0))
    /// ));
    /// # Ok::<(), QasmSimError>(())
    /// ```
    ///
    /// [`Expression`]: ./ast/enum.Expression.html
    pub fn parse_expression(source) -> Expression => open_qasm2::ExprParser;

    /// Parse `source` into a [`OpenQasmLib`] AST.
    ///
    /// The main difference between this method and [`parse_program()`] is that
    /// a library can only contain gate definitions and other statements are
    /// forbidden. Also, the list of statements in a program is colated to
    /// the source code via [`Span`]. Definitions in a library are not.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use qasmsim::{QasmSimError, grammar::parse_library};
    /// use qasmsim::grammar::lexer::Location;
    /// use qasmsim::grammar::ast::{
    ///     Argument,
    ///     Expression,
    ///     GateOperation,
    ///     OpenQasmLibrary,
    ///     Span,
    ///     Statement,
    ///     UnitaryOperation
    /// };
    ///
    /// let library_ast = parse_library(r"
    /// gate idle q {
    ///   U(0, 0, 0) q;
    /// }")?;
    ///
    /// assert_eq!(library_ast, OpenQasmLibrary{
    ///     definitions: vec![
    ///         Statement::GateDecl(
    ///             "idle".to_string(),
    ///             vec![],
    ///             vec!["q".to_string()],
    ///             vec![
    ///                 GateOperation::Unitary(
    ///                     UnitaryOperation(
    ///                         "U".to_string(),
    ///                         vec![
    ///                             Expression::Real(0.0),
    ///                             Expression::Real(0.0),
    ///                             Expression::Real(0.0)
    ///                         ],
    ///                         vec![Argument::Id("q".to_string())]
    ///                     )
    ///                 )
    ///             ]
    ///         )
    ///     ]
    /// });
    /// # Ok::<(), QasmSimError>(())
    /// ```
    ///
    /// Compare this example with the result of [`parse_program()`],
    /// [`parse_program_body()`] or [`parse_statement()`].
    ///
    /// [`Statement`]: ./ast/enum.Statement.html
    /// [`Span`]: ./ast/struct.Span.html
    /// [`parse_program()`]: ./fn.parse_program.html
    /// [`parse_program_body()`]: ./fn.parse_program_body.html
    /// [`parse_statement()`]: ./fn.parse_statement.html
    /// [`OpenQasmLib`]: ./ast/struct.OpenQasmLibrary.html
    pub fn parse_library(source) -> OpenQasmLibrary => open_qasm2::OpenQasmLibraryParser;

    /// Parse `source` into a [`OpenQasmProgram`] AST.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use qasmsim::{QasmSimError, grammar::parse_program};
    /// use qasmsim::grammar::lexer::Location;
    /// use qasmsim::grammar::ast::{
    ///     Argument,
    ///     Expression,
    ///     GateOperation,
    ///     OpenQasmProgram,
    ///     Span,
    ///     Statement,
    ///     UnitaryOperation
    /// };
    ///
    /// let program_ast = parse_program(r"
    /// OPENQASM 2.0;
    /// gate idle q {
    ///   U(0, 0, 0) q;
    /// }")?;
    ///
    /// assert_eq!(program_ast, OpenQasmProgram{
    ///     version: "2.0".to_string(),
    ///     program: vec![Span{
    ///         boundaries: (Location(15), Location(46)),
    ///         node: Box::new(Statement::GateDecl(
    ///             "idle".to_string(),
    ///             vec![],
    ///             vec!["q".to_string()],
    ///             vec![
    ///                 GateOperation::Unitary(
    ///                     UnitaryOperation(
    ///                         "U".to_string(),
    ///                         vec![
    ///                             Expression::Real(0.0),
    ///                             Expression::Real(0.0),
    ///                             Expression::Real(0.0)
    ///                         ],
    ///                         vec![Argument::Id("q".to_string())]
    ///                     )
    ///                 )
    ///             ]
    ///         ))
    ///     }]
    /// });
    /// # Ok::<(), QasmSimError>(())
    /// ```
    ///
    /// Compare this example with the result of [`parse_library()`],
    /// [`parse_program_body()`] or [`parse_statement()`].
    ///
    /// [`Statement`]: ./ast/enum.Statement.html
    /// [`parse_library()`]: ./fn.parse_library.html
    /// [`parse_program_body()`]: ./fn.parse_program_body.html
    /// [`parse_statement()`]: ./fn.parse_statement.html
    /// [`OpenQasmProgram`]: ./ast/struct.OpenQasmProgram.html
    pub fn parse_program(source) -> OpenQasmProgram => open_qasm2::OpenQasmProgramParser;

    /// Parse `source` into a list of [`Statement`]s.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use qasmsim::{QasmSimError, grammar::parse_program_body};
    /// use qasmsim::grammar::lexer::Location;
    /// use qasmsim::grammar::ast::{
    ///     Argument,
    ///     Expression,
    ///     GateOperation,
    ///     Span,
    ///     Statement,
    ///     UnitaryOperation
    /// };
    ///
    /// let statement_list = parse_program_body("gate idle q { U(0, 0, 0) q; }")?;
    ///
    /// assert_eq!(statement_list, vec![Span{
    ///     boundaries: (Location(0), Location(29)),
    ///     node: Box::new(Statement::GateDecl(
    ///         "idle".to_string(),
    ///         vec![],
    ///         vec!["q".to_string()],
    ///         vec![
    ///             GateOperation::Unitary(
    ///                 UnitaryOperation(
    ///                     "U".to_string(),
    ///                     vec![
    ///                         Expression::Real(0.0),
    ///                         Expression::Real(0.0),
    ///                         Expression::Real(0.0)
    ///                     ],
    ///                     vec![Argument::Id("q".to_string())]
    ///                 )
    ///             )
    ///         ]
    ///     ))
    /// }]);
    /// # Ok::<(), QasmSimError>(())
    /// ```
    ///
    /// Compare this example with the result of [`parse_library()`],
    /// [`parse_program()`] or [`parse_statement()`].
    ///
    /// [`Statement`]: ./ast/enum.Statement.html
    /// [`parse_library()`]: ./fn.parse_library.html
    /// [`parse_program()`]: ./fn.parse_program.html
    /// [`parse_statement()`]: ./fn.parse_statement.html
    pub fn parse_program_body(source) -> Vec<Span<Statement>> => open_qasm2::ProgramBodyParser;

    /// Parse `source` into a [`Statement`] AST.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use qasmsim::{QasmSimError, grammar::parse_statement};
    /// use qasmsim::grammar::ast::{
    ///     Argument,
    ///     Expression,
    ///     GateOperation,
    ///     Statement,
    ///     UnitaryOperation
    /// };
    ///
    /// let statement_ast = parse_statement("gate idle q { U(0, 0, 0) q; }")?;
    ///
    /// assert_eq!(statement_ast, Statement::GateDecl(
    ///     "idle".to_string(),
    ///     vec![],
    ///     vec!["q".to_string()],
    ///     vec![
    ///         GateOperation::Unitary(
    ///             UnitaryOperation(
    ///                 "U".to_string(),
    ///                 vec![
    ///                     Expression::Real(0.0),
    ///                     Expression::Real(0.0),
    ///                     Expression::Real(0.0)
    ///                 ],
    ///                 vec![Argument::Id("q".to_string())]
    ///             )
    ///         )
    ///     ]
    /// ));
    /// # Ok::<(), QasmSimError>(())
    /// ```
    ///
    /// Compare this example with the result of [`parse_library()`],
    /// [`parse_program()`] or [`parse_program_body()`].
    ///
    /// [`Statement`]: ./ast/enum.Statement.html
    /// [`parse_library()`]: ./fn.parse_library.html
    /// [`parse_program()`]: ./fn.parse_program.html
    /// [`parse_program_body()`]: ./fn.parse_program_body.html
    pub fn parse_statement(source) -> Statement => open_qasm2::StatementParser
}

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
                        OpCode::Div,
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
                OpCode::Add,
                Box::new(Expression::Minus(Box::new(Expression::Pi))),
                Box::new(Expression::Op(
                    OpCode::Div,
                    Box::new(Expression::Op(
                        OpCode::Mul,
                        Box::new(Expression::Op(
                            OpCode::Sub,
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
