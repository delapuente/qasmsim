
#[derive(Debug, PartialEq)]
pub struct OpenQasmProgram {
  pub version: String,
  pub program: Vec<Statement>
}

#[derive(Debug, PartialEq)]
pub enum Statement {
  QRegDecl(String, usize),
  CRegDecl(String, usize),
  GateDecl(String, Vec<String>,  Vec<String>, Vec<GateOperation>)
}

#[derive(Debug, PartialEq)]
pub enum GateOperation {
  Unitary(UnitaryOperation),
  Barrier(Vec<String>)
}

#[derive(Debug, PartialEq)]
pub enum UnitaryOperation {
  U(Expression, Expression, Expression, Argument),
  CX(Argument, Argument),
  GateExpansion(String, Vec<Expression>, Vec<Argument>),
  Measure(Argument, Argument),
  Reset(Argument)
}

#[derive(Debug, PartialEq)]
pub enum Expression {
  Id(String),
  Real(f64)
}

#[derive(Debug, PartialEq)]
pub enum Argument {
  Id(String),
  Item(String, usize)
}
