//! Contain the data structures for creating OPENQASM ASTs. The module is
//! **unstable**.
//!
//! # Notes
//!
//! Although OPENQASM 2.0 is stable enough, this module is not. Providing better
//! errors would require an extensive use of the `Span` structure beyond
//! statements, and adding new features to the language would require the
//! modification os certain layouts.
//!
//! The module would remain in the public API for enabling the users to perform
//! code manipulations at the abstract level.

use crate::grammar::lexer::Location;

/// Represent a OPENQASM program. A valid program contains a version string
/// and a list of instructions.
///
/// # Examples
///
/// The AST corresponding to the following program:
///
/// ```qasm
/// OPENQASM 2.0;
/// qreg q[1];
/// U(pi/2, 0, pi) q[0];
/// ```
///
/// Can be built programmatically with:
///
/// ```
/// let program = OpenQasmProgram {
///     version: "2.0".to_string(),
///     program: vec![
///         Span {
///             boundaries: (Location(14), Location(24)),
///             node: Box::new(
///                 Statement::QRegDecl(
///                     "q".to_string(),
///                     1
///                 )
///             )
///         },
///         Span {
///             boundaries: (Location(25), Location(45)),
///             node: Box::new(
///                 Statement::QuantumOperation(
///                     QuantumOperation::Unitary(
///                         UnitaryOperation(
///                             "U".to_string(),
///                             vec![
///                                 Expression::Op(
///                                     Opcode::Div,
///                                     Box::new(Expression::Pi),
///                                     Box::new(Int(2))
///                                 ),
///                                 Expression::Int(0),
///                                 Expression::Pi
///                             ],
///                             vec![
///                                 Argument::Item("q".to_string(), 0)
///                             ]
///                         )
///                     )
///                 )
///             )
///         }
///     ]
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct OpenQasmProgram {
    /// The version of the language as in `X.Y`. Current supported version is
    /// `2.0`.
    pub version: String,
    /// List of statements conforming the program body.
    pub program: Vec<Span<Statement>>,
}

/// Represent a OPENQASM library. OPENQASM libraries can contain gate
/// declarations only.
///
/// # Examples
///
/// The AST corresponding to the following library:
///
/// ```qasm
/// gate h {
///     U(pi/2, 0, pi) q[0];
/// }
/// ```
///
/// Can be built programmatically with:
///
/// ```
/// let library = OpenQasmLibrary {
///     definitions: vec![
///         Span {
///             boundaries: (Location(0), Location(35)),
///             node: Box::new(
///                 Statement::QuantumOperation(
///                     QuantumOperation::Unitary(
///                         UnitaryOperation(
///                             "U".to_string(),
///                             vec![
///                                 Expression::Op(
///                                     Opcode::Div,
///                                     Box::new(Expression::Pi),
///                                     Box::new(Int(2))
///                                 ),
///                                 Expression::Int(0),
///                                 Expression::Pi
///                             ],
///                             vec![
///                                 Argument::Item("q".to_string(), 0)
///                             ]
///                         )
///                     )
///                 )
///             )
///         }
///     ]
/// };
#[derive(Debug, Clone, PartialEq)]
pub struct OpenQasmLibrary {
    /// List of gate declarations. Although the type allows for the contruction
    /// of a library with arbitrary statements, this would not constitute a
    /// valid OPENQASM library and the linker would panic at runtime.
    pub definitions: Vec<Statement>,
}

// TODO: This should not be part of the grammar. It is a directive for
// the optimizer or compiler.

/// A pragma for a potential gate optimizer to prevent the combination of the
/// gates at both sides of the barrier. The barrier takes a list of registers
/// or qubits arguments.
///
/// # Examples
///
/// The AST corresponding to the `barrier` of the following program:
///
/// ```qasm
/// barrir q;
/// ```
///
/// Corresponds to:
///
/// ```
/// let barrier = BarrierPragma(vec![Argument::Id("q".to_string())]);
/// ```
///
/// A `BarrierPragma` cannot compound a valid [`OpenQasmProgram`]. It needs
/// to be enclosed in a [`Statement`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BarrierPragma(pub Vec<Argument>);

/// Each of the statements you can find in a OPENQASM program.
///
/// OPENQASM programs are made exclusively of list of statements. The statements
/// can be wrappers for more complex structures and take parameters representing
/// these structures.
///
/// # Examples
///
/// The following OPENQASM code:
///
/// ```qasm
/// barrier q;
/// ```
///
/// Is represented, as statement, like:
///
/// ```
/// let barrier_stmt = Statement::Barrier(
///     BarrierPragma(vec![Argument::Id("q".to_string())])
/// );
/// ```
///
/// This statement can be aggregated to form a valid [`OpenQasmProgram`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Quantum register declaration with name and size.
    QRegDecl(String, usize),
    /// Classical register declaration with name and size.
    CRegDecl(String, usize),
    /// Quantum gate declaration with name, list of formal real parameters,
    /// list of formal quantum registers, and a list of [`GateOperation`]
    /// representing the body of the gate.
    GateDecl(String, Vec<String>, Vec<String>, Vec<GateOperation>),
    /// Include statement for linking with gate libraries.
    Include(String),
    /// A wrapper for the barrier pragma.
    Barrier(BarrierPragma),
    /// Opaque gate declaration with name and formal lists of real parameters
    /// and quantum registers. Opaque declarations have no body.
    OpaqueGateDecl(String, Vec<String>, Vec<String>),
    /// A wrapper for a quantum operation.
    QuantumOperation(QuantumOperation),
    /// A wrapper for making a quantum operation to execute just if certain
    /// equality condition holds. The wrapper takes the left-side of the
    /// comparison, the right side, and the operation to perform.
    Conditional(Argument, u64, QuantumOperation),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span<S> {
    pub boundaries: (Location, Location),
    pub node: Box<S>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GateOperation {
    Unitary(UnitaryOperation),
    Barrier(BarrierPragma),
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuantumOperation {
    Unitary(UnitaryOperation),
    Measure(Argument, Argument),
    Reset(Argument),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitaryOperation(pub String, pub Vec<Expression>, pub Vec<Argument>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Opcode {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Funccode {
    Sin,
    Cos,
    Tan,
    Exp,
    Ln,
    Sqrt,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Pi,
    Id(String),
    Real(f64),
    Int(u64),
    Op(Opcode, Box<Expression>, Box<Expression>),
    Function(Funccode, Box<Expression>),
    Minus(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Argument {
    Id(String),
    Item(String, usize),
}
