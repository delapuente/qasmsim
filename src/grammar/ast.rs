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

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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
/// use qasmsim::grammar::ast::{
///     OpenQasmProgram,
///     Span,
///     Statement,
///     QuantumOperation,
///     UnitaryOperation,
///     Opcode,
///     Expression,
///     Argument
/// };
/// use qasmsim::grammar::lexer::Location;
///
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
///                                     Box::new(Expression::Int(2))
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
/// gate h q {
///     U(pi/2, 0, pi) q[0];
/// }
/// ```
///
/// Can be built programmatically with:
///
/// ```
/// use qasmsim::grammar::ast::{
///     OpenQasmLibrary,
///     Statement,
///     GateOperation,
///     UnitaryOperation,
///     Opcode,
///     Expression,
///     Argument
/// };
/// use qasmsim::grammar::lexer::Location;
///
/// let library = OpenQasmLibrary {
///     definitions: vec![
///         Statement::GateDecl(
///             "h".to_string(),
///             vec![],
///             vec!["q".to_string()],
///             vec![
///                 GateOperation::Unitary(
///                     UnitaryOperation(
///                        "U".to_string(),
///                         vec![
///                             Expression::Op(
///                                 Opcode::Div,
///                                 Box::new(Expression::Pi),
///                                 Box::new(Expression::Int(2))
///                             ),
///                             Expression::Int(0),
///                             Expression::Pi
///                         ],
///                         vec![
///                             Argument::Item("q".to_string(), 0)
///                         ]
///                     )
///                 )
///             ]
///         )
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
/// barrier q;
/// ```
///
/// Corresponds to:
///
/// ```
/// use qasmsim::grammar::ast::{BarrierPragma, Argument};
///
/// let barrier = BarrierPragma(vec![Argument::Id("q".to_string())]);
/// ```
///
/// A `BarrierPragma` cannot compound a valid [`OpenQasmProgram`]. It needs
/// to be enclosed in a [`Statement`].
///
/// [`OpenQasmProgram`]: ./struct.OpenQasmProgram.html
/// [`Statement`]: ./enum.Statement.html
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
/// use qasmsim::grammar::ast::{Statement, BarrierPragma, Argument};
///
/// let barrier_stmt = Statement::Barrier(
///     BarrierPragma(vec![Argument::Id("q".to_string())])
/// );
/// ```
///
/// Enclose a statement inside a [span] and aggregate them in a list to form
/// a valid [`OpenQasmProgram`].
///
/// [span]: ./enum.Span.html
/// [`OpenQasmProgram`]: ./struct.OpenQasmProgram.html
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    /// A wrapper for making a quantum operation to simulate just if certain
    /// equality condition holds. The wrapper takes the left-side of the
    /// comparison, the right side, and the operation to perform.
    Conditional(Argument, u64, QuantumOperation),
}

/// Relates a node with the fragment of source code where the node appears.
///
/// # Examples
///
/// Consider the following program:
///
/// ```qasm
/// OPENQASM 2.0;
/// qreg q[10];
/// ```
///
/// The span for the second line statement is as follows:
///
/// ```
/// use qasmsim::grammar::ast::{Span, Statement};
/// use qasmsim::grammar::lexer::Location;
///
/// let barrier_span = Span {
///     boundaries: (Location(14), Location(25)),
///     node: Box::new(
///         Statement::QRegDecl(
///             "q".to_string(),
///             1
///         )
///     )
/// };
/// ```
///
/// Boundaries run from characters 14 to 25 corresponding to the starting-0
/// character index of the source code.
///
/// Right, now, only statements are tied to spans making impossible to
/// accurately localize inner AST nodes.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Span<S> {
    /// Pair of source locations where the AST node can be found.
    pub boundaries: (Location, Location),
    /// Boxed AST node.
    pub node: Box<S>,
}

/// Any of the statements that can appear inside a gate definition.
///
/// # Examples
///
/// See [`OpenQasmLibrary`] for a complete example.
///
/// A gate in OPENQASM is always reversible so gates can only be compound of
/// barriers (which are no-op actually) and other gate invocations.
///
/// [`OpenQasmLibrary`]: ./struct.OpenQasmLibrary.html
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GateOperation {
    /// A gate invocation.
    Unitary(UnitaryOperation),
    /// A barrier pragma.
    Barrier(BarrierPragma),
}

/// Any of the operations that actuates over quantum registers.
///
/// # Examples
///
/// See [`OpenQasmProgram`] for a complete examples.
///
/// [`OpenQasmProgram`]: ./struct.OpenQasmProgram.html
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum QuantumOperation {
    /// A gate invocation.
    Unitary(UnitaryOperation),
    /// A measurement on a quantum register to a classical register.
    Measure(Argument, Argument),
    /// A reset operation on a quantum register.
    Reset(Argument),
}

/// A gate "invocation".
///
/// The name comes after the fact that all quantum gates are [unitary]
/// operators. Calling a gate consists on applying it on some quantum
/// registers.
///
/// # Examples
///
/// See [`OpenQasmProgram`] for a complete example.
///
/// [`OpenQasmProgram`]: ./struct.OpenQasmProgram.html
/// [unitary]: https://en.wikipedia.org/wiki/Unitary_operator
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UnitaryOperation(pub String, pub Vec<Expression>, pub Vec<Argument>);

/// Any of the operators that can appear in an expression.
///
/// # Examples
///
/// Notice the different `Expression` instances in the [`OpenQasmLibrary`]
/// example.
///
/// [`OpenQasmLibrary`]: ./struct.OpenQasmLibrary.html
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Opcode {
    /// Code for the addition operator `+`.
    Add,
    /// Code for the substraction operator `-`.
    Sub,
    /// Code for the multiplication operator `*`.
    Mul,
    /// Code for the division operator `/`.
    Div,
    /// Code for the power operator `^`.
    Pow,
}

/// Any of the functions that can appear in an expression.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Funccode {
    /// Function sinus `sin`.
    Sin,
    /// Function cosinus `cos`.
    Cos,
    /// Function tangent `tan`.
    Tan,
    /// Function exponential `exp`.
    Exp,
    /// Function natural logarithm `ln`.
    Ln,
    /// Function square root `sqrt`.
    Sqrt,
}

/// Any of the subexpressions that can appear inside a expression.
///
/// # Examples
///
/// See [`OpenQasmLibrary`] for an example.
///
/// [`OpenQasmLibrary`]: ./struct.OpenQasmLibrary.html
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Expression {
    /// The pi constant `pi`.
    Pi,
    /// A valid OPENQASM identifier.
    Id(String),
    /// A real number.
    Real(f64),
    /// An integer number.
    Int(u64),
    /// A binary operation.
    Op(Opcode, Box<Expression>, Box<Expression>),
    /// A call to a function.
    Function(Funccode, Box<Expression>),
    /// A negation of an expression.
    Minus(Box<Expression>),
}

/// A reference to a register or register component.
///
/// # Examples
///
/// Look at these barrier statements:
///
/// ```qasm
/// barrier q;
/// barrier q[0];
/// ```
///
/// They differ on the quantum register argument and can be built with:
///
/// ```
/// use qasmsim::grammar::ast::{BarrierPragma, Argument};
///
/// let on_the_whole_register = BarrierPragma(
///     vec![Argument::Id("q".to_string())]
/// );
/// let on_the_first_qubit = BarrierPragma(
///     vec![Argument::Item("q".to_string(), 0)]
/// );
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Argument {
    /// An entire register like `q`.
    Id(String),
    /// One of the bits/qubits of a register `q[0]`.
    Item(String, usize),
}
