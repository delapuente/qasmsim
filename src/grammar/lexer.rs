use std::str::CharIndices;
use std::collections::VecDeque;
use std::collections::HashMap;

use regex::Regex;

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Debug, Clone, PartialEq)]
pub struct LexicalError;

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
  Add,
  Minus,
  Mult,
  Div,
  LBracket,
  RBracket,
  LBrace,
  RBrace,
  LParent,
  RParent,
  Semi,
  Comma,
  Arrow,
  Equal,
  ConstPi,
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
  If,
  QASMHeader,
  Version { repr: String },
  Id { repr: String },
  Int { repr: String },
  Real { repr: String },
  Str { repr: String }
}

fn get_keywords() -> HashMap<String, Tok> {
  let mut kw = HashMap::new();
  kw.insert(String::from("pi"), Tok::ConstPi);
  kw.insert(String::from("opaque"), Tok::Opaque);
  kw.insert(String::from("gate"), Tok::Gate);
  kw.insert(String::from("include"), Tok::Include);
  kw.insert(String::from("qreg"), Tok::QReg);
  kw.insert(String::from("creg"), Tok::CReg);
  kw.insert(String::from("measure"), Tok::Measure);
  kw.insert(String::from("reset"), Tok::Reset);
  kw.insert(String::from("barrier"), Tok::Barrier);
  kw.insert(String::from("if"), Tok::If);
  kw
}

#[derive(Debug, Clone, PartialEq)]
enum Mode {
  Base,
  Version,
  Comment,
  Str
}

pub struct Lexer<'input> {
  mode: VecDeque<Mode>,
  offset: usize,
  input: &'input str,
  keywords: HashMap<String, Tok>,
  chars: std::iter::Peekable<CharIndices<'input>>
}

impl<'input> Lexer<'input> {
  pub fn new(input: &'input str) -> Self {
    Lexer {
      mode: VecDeque::from(vec![Mode::Base]),
      offset: 0,
      input,
      keywords: get_keywords(),
      chars: input.char_indices().peekable(),
    }
  }

  fn try_pattern(&mut self, re: &Regex) -> Option<String> {
    match re.captures(&self.input[self.offset..]) {
      None => None,
      Some(captured) => {
        let matching = &captured.get(0).unwrap();
        let len = matching.end(); // same as length since we search from the start.
        self.advance_offset(len);
        return Some(String::from(matching.as_str()));
      }
    }
  }

  fn advance_offset(&mut self, count: usize) {
    for _ in 0..count { self.chars.next(); }
    self.offset += count;
  }
}

impl<'input> Iterator for Lexer<'input> {
  type Item = Spanned<Tok, usize, LexicalError>;

  // XXX: The function is not split since I'm trying to distinguish a pattern
  // for creating a macro to autogenerate a stack-based lexer with matching
  // rules specific per mode.
  //
  // Proposed syntax (if possible): #[modes(mode1, mode2,...)]
  fn next(&mut self) -> Option<Self::Item> {
    lazy_static! {
      static ref ALL_THE_LINE: Regex = Regex::new(r"^[^\n]*").unwrap();
      static ref BLANK: Regex = Regex::new(r"^\s+").unwrap();
      static ref GATE: Regex = Regex::new(r"^(CX|U)\b").unwrap();
      static ref OPENQASM: Regex = Regex::new(r"^OPENQASM\b").unwrap();
      static ref VERSION: Regex = Regex::new(r"^([0-9]+\.[0-9]+)").unwrap();
      static ref ID: Regex = Regex::new(r"^([a-z][A-Za-z0-9_]*)").unwrap();
      static ref INTEGER: Regex = Regex::new(r"^([1-9]+[0-9]*|0)").unwrap();
      static ref REAL: Regex = Regex::new(r"^([0-9]+\.[0-9]*|[0-9]*\.[0-9]+)([eE][+-]?[0-9])?").unwrap();
      static ref SYMBOL: Regex = Regex::new(r"^(->|==|//|[+\-\*/\[\]\{\}\(\);,])").unwrap();
    }

    loop {
      if self.chars.peek().is_none() {
        return None;
      }

      // #[modes(all)]
      if let Some(_blank) = self.try_pattern(&BLANK) {
        continue;
      }

      let start = self.offset;

      // #[modes(Base)]
      match self.mode.get(0) {
        Some(Mode::Base) => {
          if let Some((_, c)) = self.chars.peek() {
            if *c == '"' {
              self.mode.push_front(Mode::Str);
              self.advance_offset(1);
              continue;
            }
          }
        }
        _ => ()
      }

      // #[modes(Str)]
      match self.mode.get(0) {
        Some(Mode::Str) => {
          loop {
            match self.chars.next() {
              None => { return None; }
              Some((_, '\\')) => { self.chars.next(); } // ignore next char
              Some((end, '"')) => {
                self.mode.pop_front();
                self.offset = end + 1;
                let content = &self.input[start..end];
                return Some(Ok((start - 1, Tok::Str{ repr: String::from(content) }, end + 1)));
              }
              _ => ()
            }
          }
        }
        _ => ()
      }

      // #[modes(Comment)]
      match self.mode.get(0) {
        Some(Mode::Comment) => {
          if let Some(_) = self.try_pattern(&ALL_THE_LINE) {
            self.mode.pop_front();
            continue;
          }
        }
        _ => ()
      }

      // #[modes(all)]
      if let Some(repr) = self.try_pattern(&OPENQASM) {
        self.mode.push_front(Mode::Version);
        let end = start + repr.len();
        return Some(Ok((start, Tok::QASMHeader, end)));
      }

      // #[modes(all)]
      if let Some(gate) = self.try_pattern(&GATE) {
        let end = start + gate.len();
        return Some(match gate.as_str() {
          "U" => Ok((start, Tok::U, end)),
          "CX" => Ok((start, Tok::CX, end)),
          _ => unreachable!()
        })
      }

      // #[modes(all)]
      if let Some(repr) = self.try_pattern(&ID) {
        let end = start + repr.len();
        return Some(match self.keywords.get(&repr) {
          None => Ok((start, Tok::Id{ repr }, end)),
          Some(token) => Ok((start, (*token).clone(), end))
        })
      }

      // #[modes(Base)]
      match self.mode.get(0) {
        Some(Mode::Base) => {
          if let Some(repr) = self.try_pattern(&REAL) {
            let end = start + repr.len();
            return Some(Ok((start, Tok::Real{ repr }, end)));
          }
        }
        _ => ()
      }

      // #[modes(Version)]
      match self.mode.get(0) {
        Some(Mode::Version) => {
          if let Some(repr) = self.try_pattern(&VERSION) {
            let end = start + repr.len();
            self.mode.pop_front();
            return Some(Ok((start, Tok::Version{ repr }, end)));
          }
        }
        _ => ()
      }

      // #[modes(all)]
      if let Some(repr) = self.try_pattern(&INTEGER) {
        let end = start + repr.len();
        return Some(Ok((start, Tok::Int{ repr }, end)));
      }

      // #[modes(all)]
      if let Some(symbol) = self.try_pattern(&SYMBOL) {
        let end = start + symbol.len();
        let token = match symbol.as_str() {
          "+" => Tok::Add,
          "-" => Tok::Minus,
          "*" => Tok::Mult,
          "/" => Tok::Div,
          "[" => Tok::LBracket,
          "]" => Tok::RBracket,
          "{" => Tok::LBrace,
          "}" => Tok::RBrace,
          "(" => Tok::LParent,
          ")" => Tok::RParent,
          ";" => Tok::Semi,
          "," => Tok::Comma,
          "->" => Tok::Arrow,
          "==" => Tok::Equal,
          "//" => { self.mode.push_front(Mode::Comment); continue },
          _ => unreachable!()
        };
        return Some(Ok((start, token, end)));
      }

      return Some(Err(LexicalError));
    }
  }
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
  fn test_literals() {
    let source = "0 1 20 .3 .4e5 0.6E-7 \"8910\"";
    let lexer = Lexer::new(source);
    assert_eq!(lexer.collect::<Vec<_>>(), vec![
      Ok((0, Tok::Int { repr: String::from("0") }, 1)),
      Ok((2, Tok::Int { repr: String::from("1") }, 3)),
      Ok((4, Tok::Int { repr: String::from("20") }, 6)),
      Ok((7, Tok::Real { repr: String::from(".3") }, 9)),
      Ok((10, Tok::Real { repr: String::from(".4e5") }, 14)),
      Ok((15, Tok::Real { repr: String::from("0.6E-7") }, 21)),
      Ok((22, Tok::Str { repr: String::from("8910") }, 28)),
    ]);
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
      Ok((14, Tok::Version{ repr: String::from("2.0") }, 17)),
      Ok((17, Tok::Semi, 18))
    ]);
  }

  #[test]
  fn test_simple_symbols() {
    let source = "+-*/[]{}();,";
    let lexer = Lexer::new(source);
    assert_eq!(lexer.collect::<Vec<_>>(), vec![
      Ok((0, Tok::Add, 1)),
      Ok((1, Tok::Minus, 2)),
      Ok((2, Tok::Mult, 3)),
      Ok((3, Tok::Div, 4)),
      Ok((4, Tok::LBracket, 5)),
      Ok((5, Tok::RBracket, 6)),
      Ok((6, Tok::LBrace, 7)),
      Ok((7, Tok::RBrace, 8)),
      Ok((8, Tok::LParent, 9)),
      Ok((9, Tok::RParent, 10)),
      Ok((10, Tok::Semi, 11)),
      Ok((11, Tok::Comma, 12))
    ]);
  }

  #[test]
  fn test_composite_symbols() {
    let source = "->==//";
    let lexer = Lexer::new(source);
    assert_eq!(lexer.collect::<Vec<_>>(), vec![
      Ok((0, Tok::Arrow, 2)),
      Ok((2, Tok::Equal, 4))
    ]);
  }

  #[test]
  fn test_keywords() {
    for (keyword, token) in get_keywords() {
      let lexer = Lexer::new(&keyword);
      assert_eq!(
        lexer.collect::<Vec<_>>(), vec![Ok((0, token, keyword.len()))]);
    }
  }

  #[test]
  fn test_gates() {
    let source = "CX U";
    let lexer = Lexer::new(source);
    assert_eq!(lexer.collect::<Vec<_>>(), vec![
      Ok((0, Tok::CX, 2)),
      Ok((3, Tok::U, 4))
    ]);
  }

  mod regressions {
    use super::*;

    #[test]
    fn test_gate_call() {
      let source = "U(pi/2, 0, pi) q;";
      let lexer = Lexer::new(source);
      assert_eq!(lexer.collect::<Vec<_>>(), vec![
        Ok((0, Tok::U, 1)),
        Ok((1, Tok::LParent, 2)),
        Ok((2, Tok::ConstPi, 4)),
        Ok((4, Tok::Div, 5)),
        Ok((5, Tok::Int { repr: String::from("2")}, 6)),
        Ok((6, Tok::Comma, 7)),
        Ok((8, Tok::Int { repr: String::from("0")}, 9)),
        Ok((9, Tok::Comma, 10)),
        Ok((11, Tok::ConstPi, 13)),
        Ok((13, Tok::RParent, 14)),
        Ok((15, Tok::Id { repr: String::from("q") }, 16)),
        Ok((16, Tok::Semi, 17)),
      ]);
    }
  }
}