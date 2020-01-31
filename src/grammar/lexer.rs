pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Debug)]
pub enum LexicalError {
  // Not possible
}

use std::str::CharIndices;

pub struct Lexer<'input> {
  chars: CharIndices<'input>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
  Semi,
  LBrace,
  RBrace,
  LBracket,
  RBracket,
  LParent,
  RParent,
  Add,
  Minus,
  Mult,
  Div,
  ConstPi,
  Comman,
  U,
  CX,
  Opaque,
  Gate,
  Include,
  QReg,
  CReg,
  Measure,
  Reset,
  Barrier,
  Arrow,
  QASMHeader,
  Version { version: String },
  Id { name: String },
  Int { value: usize },
  Float { value: f64 },
  Str { value: String }
}

impl<'input> Lexer<'input> {
  pub fn new(input: &'input str) -> Self {
    Lexer {
      chars: input.char_indices(),
    }
  }
}

impl<'input> Iterator for Lexer<'input> {
  type Item = Spanned<Tok, usize, LexicalError>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if self.chars.next().is_none() {
        return None;
      }
    }
  }
}