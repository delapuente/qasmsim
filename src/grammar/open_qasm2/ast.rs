
#[derive(Debug)]
pub struct OpenQasmProgram {
  pub version: String,
  pub program: Vec<Statement>
}

#[derive(Debug)]
pub enum Statement {
  QRegDecl(String, usize),
  CRegDecl(String, usize)
}

