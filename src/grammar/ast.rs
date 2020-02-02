
#[derive(Debug, PartialEq)]
pub struct OpenQasmProgram {
  pub version: String,
  pub program: Vec<Statement>
}

#[derive(Debug, PartialEq)]
pub struct OpenQasmLibrary {
  pub definitions: Vec<Statement>
}

#[derive(Debug, PartialEq)]
pub enum Statement {
  QRegDecl(String, usize),
  CRegDecl(String, usize),
  GateDecl(String, Vec<String>,  Vec<String>, Vec<GateOperation>),
  Include(String),
  OpaqueGateDecl(String, Vec<String>, Vec<String>),
  QuantumOperation(QuantumOperation),
  Conditional(Argument, u64, QuantumOperation)
}

#[derive(Debug, PartialEq, Clone)]
pub enum GateOperation {
  Unitary(UnitaryOperation),
  Barrier(Vec<String>)
}

#[derive(Debug, PartialEq)]
pub enum QuantumOperation {
  Unitary(UnitaryOperation),
  Measure(Argument, Argument),
  Reset(Argument)
}

#[derive(Debug, PartialEq, Clone)]
pub struct UnitaryOperation(pub String, pub Vec<Expression>, pub Vec<Argument>);

#[derive(Debug, PartialEq, Clone)]
pub enum Opcode {
  Add,
  Sub,
  Mul,
  Div
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
  Pi,
  Id(String),
  Real(f64),
  Int(u64),
  Op(Opcode, Box<Expression>, Box<Expression>),
  Minus(Box<Expression>)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Argument {
  Id(String),
  Item(String, usize)
}
