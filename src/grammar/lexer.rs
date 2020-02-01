use std::str::CharIndices;
use std::collections::HashMap;

use regex::Regex;

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Debug, Clone, PartialEq)]
pub enum LexicalError {
  // Not possible
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
  Version { repr: String },
  Id { repr: String },
  Int { repr: String },
  Float { repr: String },
  Str { repr: String }
}

fn get_keywords() -> HashMap<String, Tok> {
  let mut kw = HashMap::new();
  kw.insert(String::from("U"), Tok::U);
  kw.insert(String::from("CX"), Tok::CX);
  kw.insert(String::from("opaque"), Tok::Opaque);
  kw.insert(String::from("gate"), Tok::Gate);
  kw.insert(String::from("indlude"), Tok::Include);
  kw.insert(String::from("qreg"), Tok::QReg);
  kw.insert(String::from("creg"), Tok::CReg);
  kw.insert(String::from("measure"), Tok::Measure);
  kw.insert(String::from("reset"), Tok::Reset);
  kw.insert(String::from("barrier"), Tok::Barrier);
  kw.insert(String::from("OPENQASM"), Tok::QASMHeader);
  kw
}

pub struct Lexer<'input> {
  input: &'input str,
  keywords: HashMap<String, Tok>,
  chars: std::iter::Peekable<CharIndices<'input>>
}

impl<'input> Lexer<'input> {
  pub fn new(input: &'input str) -> Self {
    Lexer {
      input,
      keywords: get_keywords(),
      chars: input.char_indices().peekable(),
    }
  }

  pub fn next_token(&mut self, start: usize, c: char)
  -> Spanned<Tok, usize, LexicalError> {
    if is_word_start(c) {
      let end = self.find_end_of_word();
      let word = &self.input[start..end];
      match self.keywords.get(word) {
        None => return Ok((start, Tok::Id{ repr: String::from(word) }, end)),
        Some(token) => return Ok((start, (*token).clone(), end))
      }
    }

    if is_number_start(c) {
      match self.try_number_kinds(start) {
        None => (),
        Some(spanned) => return spanned
      }
    }
    Ok((start, Tok::Add, start+1))
  }

  pub fn find_end_of_word(&mut self) -> usize {
    loop {
      match self.chars.peek() {
        None => return self.input.len(),
        Some((i, c)) => {
          if *c != '_' && !c.is_alphanumeric() {
            return *i;
          }
          self.chars.next();
        }
      }
    }
  }

  pub fn try_number_kinds(&mut self, start: usize)
  -> Option<Spanned<Tok, usize, LexicalError>> {
    lazy_static! {
      static ref FLOAT: Regex = Regex::new(r"([0-9]+\.[0-9]*|[0-9]*\.[0-9]+)([eE][+-]?[0-9])?").unwrap();
    }
    match FLOAT.captures(&self.input[start..]) {
      None => (),
      Some(captured) => {
        let number = captured.get(0).unwrap().as_str();
        for _ in 0..number.len() { self.chars.next(); }
        return Some(Ok((
          start,
          Tok::Float{ repr: String::from(number) },
          start + number.len()
        )));
      }
    }
    None
  }
}

impl<'input> Iterator for Lexer<'input> {
  type Item = Spanned<Tok, usize, LexicalError>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      match self.chars.next() {
        None => return None,
        Some((_, ' ')) | Some((_, '\t')) | Some((_, '\n')) => continue,
        Some((i, c)) => return Some(self.next_token(i, c))
      }
    }
  }
}

pub fn is_word_start(c: char) -> bool {
  c == '_' || c.is_alphabetic()
}

pub fn is_number_start(c: char) -> bool {
  c == '.' || c.is_numeric()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_all_blankspace() {
    let source = "  \t\t\n\n\n\t\t  ";
    let lexer = Lexer::new(source);
    assert_eq!(lexer.collect::<Vec<_>>(), vec![]);
  }

  #[test]
  fn test_some_blankspace() {
    let source = "

    OPENQASM
    \t
    ";
    let lexer = Lexer::new(source);
    assert_eq!(lexer.collect::<Vec<_>>(), vec![
      Ok((6, Tok::QASMHeader, 14)),
    ]);
  }

  #[test]
  fn test_openqasm_header_sequence() {
    let source = "
    OPENQASM 2.0;
    ";
    let lexer = Lexer::new(source);
    assert_eq!(lexer.collect::<Vec<_>>(), vec![
      Ok((5, Tok::QASMHeader, 13)),
      Ok((14, Tok::Float{ repr: String::from("2.0") }, 17)),
    ]);
  }
}