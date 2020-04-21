
use crate::grammar::Location;

#[derive(Debug, Clone, PartialEq)]
pub struct OpenQasmProgram {
  pub version: String,
  pub program: Vec<Span<Statement>>
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpenQasmLibrary {
  pub definitions: Vec<Statement>
}

// TODO: This should not be part of the grammar. It is a directive for
// the optimizer or compiler.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BarrierPragma(pub Vec<Argument>);

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
  QRegDecl(String, usize),
  CRegDecl(String, usize),
  GateDecl(String, Vec<String>,  Vec<String>, Vec<GateOperation>),
  Include(String),
  Barrier(BarrierPragma),
  OpaqueGateDecl(String, Vec<String>, Vec<String>),
  QuantumOperation(QuantumOperation),
  Conditional(Argument, u64, QuantumOperation)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span<S> {
  pub boundaries: (Location, Location),
  pub node: Box<S>
}

#[derive(Debug, Clone, PartialEq)]
pub enum GateOperation {
  Unitary(UnitaryOperation),
  Barrier(BarrierPragma)
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuantumOperation {
  Unitary(UnitaryOperation),
  Measure(Argument, Argument),
  Reset(Argument)
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitaryOperation(pub String, pub Vec<Expression>, pub Vec<Argument>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Opcode {
  Add,
  Sub,
  Mul,
  Div,
  Pow
}

#[derive(Debug, Clone, PartialEq)]
pub enum Funccode {
  Sin,
  Cos,
  Tan,
  Exp,
  Ln,
  Sqrt
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
  Pi,
  Id(String),
  Real(f64),
  Int(u64),
  Op(Opcode, Box<Expression>, Box<Expression>),
  Function(Funccode, Box<Expression>),
  Minus(Box<Expression>)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Argument {
  Id(String),
  Item(String, usize)
}
