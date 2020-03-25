use std::str::CharIndices;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::fmt;

use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Location {
  pub lineno: usize,
  pub linepos: usize,
  pub lineoffset: usize
}

impl fmt::Display for Location {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "L{}C{}", self.lineno, self.lineoffset)
  }
}

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Debug, Clone, PartialEq)]
pub struct LexicalError<Loc> { pub location: Loc }

impl<Loc> fmt::Display for LexicalError<Loc>
where Loc: fmt::Display {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "invalid token at {}", self.location)
  }
}

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

impl fmt::Display for Tok {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let repr: String = match self {
      Tok::Add => "+".into(),
      Tok::Minus => "-".into(),
      Tok::Mult => "*".into(),
      Tok::Div => "/".into(),
      Tok::LBracket => "[".into(),
      Tok::RBracket => "]".into(),
      Tok::LBrace => "{".into(),
      Tok::RBrace => "}".into(),
      Tok::LParent => "(".into(),
      Tok::RParent => ")".into(),
      Tok::Semi => ";".into(),
      Tok::Comma => ",".into(),
      Tok::Arrow => "=>".into(),
      Tok::Equal => "==".into(),
      Tok::ConstPi => "constant `pi`".into(),
      Tok::U => "primitive gate `U`".into(),
      Tok::CX => "primitive gate `CX`".into(),
      Tok::Opaque => "keyword `opaque`".into(),
      Tok::Gate => "keyword `gate`".into(),
      Tok::Include => "keyword `include`".into(),
      Tok::QReg => "keyword `qreg`".into(),
      Tok::CReg => "keyword `creg`".into(),
      Tok::Measure => "keyword `measure`".into(),
      Tok::Reset => "keyword `reset`".into(),
      Tok::Barrier => "keyword `barrier`".into(),
      Tok::If => "keyword `if`".into(),
      Tok::QASMHeader => "qasm header `OPENQASM`".into(),
      Tok::Version { repr } => format!("open qasm version `{}`", &repr),
      Tok::Id { repr } => format!("identifier `{}`", &repr),
      Tok::Int { repr } => format!("integer literal `{}`", &repr),
      Tok::Real { repr } => format!("real literal `{}`", &repr),
      Tok::Str { repr } => format!("string literal `\"{}\"`", &repr),
    };
    write!(f, "{}", repr)
  }
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
  lineno: usize,
  lineoffset: usize,
  offset: usize,
  input: &'input str,
  keywords: HashMap<String, Tok>,
  chars: std::iter::Peekable<CharIndices<'input>>,
  errored: bool
}

impl<'input> Lexer<'input> {
  pub fn new(input: &'input str) -> Self {
    Lexer {
      mode: VecDeque::from(vec![Mode::Base]),
      lineno: 1,
      lineoffset: 0,
      offset: 0,
      input,
      keywords: get_keywords(),
      chars: input.char_indices().peekable(),
      errored: false
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

  fn location(&self, offset: usize) -> Location {
    Location {
      lineno: self.lineno,
      linepos: offset - self.lineoffset,
      lineoffset: self.lineoffset
    }
  }
}

impl<'input> Iterator for Lexer<'input> {
  type Item = Spanned<Tok, Location, LexicalError<Location>>;

  // XXX: The function is not split since I'm trying to distinguish a pattern
  // for creating a macro to autogenerate a stack-based lexer with matching
  // rules specific per mode.
  //
  // Proposed syntax (if possible): #[modes(mode1, mode2,...)]
  fn next(&mut self) -> Option<Self::Item> {
    lazy_static! {
      static ref NEW_LINE: Regex = Regex::new(r"^\n").unwrap();
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
      if self.errored || self.chars.peek().is_none() {
        return None;
      }

      if let Some(_new_line) = self.try_pattern(&NEW_LINE) {
        self.lineno += 1;
        self.lineoffset = self.offset;
        // TODO: Should I force a new loop? It seems consistent with a
        // line-oriented tokenization. If generalizing the lexer, I should
        // consider enabling/disabling multiline support and, if disables,
        // treat `\n` as a regular character.
        continue;
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
                return Some(Ok((
                  self.location(start - 1),
                  Tok::Str{ repr: String::from(content) },
                  self.location(end + 1)
                )));
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
        return Some(Ok((
          self.location(start),
          Tok::QASMHeader,
          self.location(end)
        )));
      }

      // #[modes(all)]
      if let Some(gate) = self.try_pattern(&GATE) {
        let end = start + gate.len();
        return Some(match gate.as_str() {
          "U" => Ok((
            self.location(start),
            Tok::U,
            self.location(end)
          )),
          "CX" => Ok((
            self.location(start),
            Tok::CX,
            self.location(end)
          )),
          _ => unreachable!()
        })
      }

      // #[modes(all)]
      if let Some(repr) = self.try_pattern(&ID) {
        let end = start + repr.len();
        return Some(match self.keywords.get(&repr) {
          None => Ok((
            self.location(start),
            Tok::Id{ repr }, self.location(end)
          )),
          Some(token) => Ok((
            self.location(start),
            (*token).clone(),
            self.location(end)
          ))
        })
      }

      // #[modes(Base)]
      match self.mode.get(0) {
        Some(Mode::Base) => {
          if let Some(repr) = self.try_pattern(&REAL) {
            let end = start + repr.len();
            return Some(Ok((
              self.location(start),
              Tok::Real{ repr },
              self.location(end)
            )));
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
            return Some(Ok((
              self.location(start),
              Tok::Version{ repr },
              self.location(end)
            )));
          }
        }
        _ => ()
      }

      // #[modes(all)]
      if let Some(repr) = self.try_pattern(&INTEGER) {
        let end = start + repr.len();
        return Some(Ok((
          self.location(start),
          Tok::Int{ repr },
          self.location(end)
        )));
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
        return Some(Ok((
          self.location(start),
          token,
          self.location(end)
        )));
      }

      self.errored = true;
      return Some(Err(LexicalError { location: self.location(start) }));
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
      Ok((
        Location{ lineno: 1, linepos: 0, lineoffset: 0 },
        Tok::Int { repr: String::from("0") },
        Location{ lineno: 1, linepos: 1, lineoffset: 0 }
      )),
      Ok((
        Location{ lineno: 1, linepos: 2, lineoffset: 0 },
        Tok::Int { repr: String::from("1") },
        Location{ lineno: 1, linepos: 3, lineoffset: 0 },
      )),
      Ok((
        Location{ lineno: 1, linepos: 4, lineoffset: 0 },
        Tok::Int { repr: String::from("20") },
        Location{ lineno: 1, linepos: 6, lineoffset: 0 },
      )),
      Ok((
        Location{ lineno: 1, linepos: 7, lineoffset: 0 },
        Tok::Real { repr: String::from(".3") },
        Location{ lineno: 1, linepos: 9, lineoffset: 0 },
      )),
      Ok((
        Location{ lineno: 1, linepos: 10, lineoffset: 0 },
        Tok::Real { repr: String::from(".4e5") },
        Location{ lineno: 1, linepos: 14, lineoffset: 0 }
      )),
      Ok((
        Location{ lineno: 1, linepos: 15, lineoffset: 0 },
        Tok::Real { repr: String::from("0.6E-7") },
        Location{ lineno: 1, linepos: 21, lineoffset: 0 }
      )),
      Ok((
        Location{ lineno: 1, linepos: 22, lineoffset: 0 },
        Tok::Str { repr: String::from("8910") },
        Location{ lineno: 1, linepos: 28, lineoffset: 0 }
      )),
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
      Ok((
        Location{ lineno: 3, linepos: 4, lineoffset: 2 },
        Tok::QASMHeader,
        Location{ lineno: 3, linepos: 12, lineoffset: 2 }
      )),
    ]);
  }

  #[test]
  fn test_openqasm_header_sequence() {
    let source = "
    OPENQASM 2.0;
    ";
    let lexer = Lexer::new(source);
    assert_eq!(lexer.collect::<Vec<_>>(), vec![
      Ok((
        Location{ lineno: 2, linepos: 4, lineoffset: 1 },
        Tok::QASMHeader,
        Location{ lineno: 2, linepos: 12, lineoffset: 1 }
      )),
      Ok((
        Location{ lineno: 2, linepos: 13, lineoffset: 1 },
        Tok::Version{ repr: String::from("2.0") },
        Location{ lineno: 2, linepos: 16, lineoffset: 1 }
      )),
      Ok((
        Location{ lineno: 2, linepos: 16, lineoffset: 1 },
        Tok::Semi,
        Location{ lineno: 2, linepos: 17, lineoffset: 1 }
      ))
    ]);
  }

  #[test]
  fn test_simple_symbols() {
    let source = "+-*/[]{}();,";
    let lexer = Lexer::new(source);
    assert_eq!(lexer.collect::<Vec<_>>(), vec![
      Ok((
        Location{ lineno: 1, linepos: 0, lineoffset: 0 },
        Tok::Add,
        Location{ lineno: 1, linepos: 1, lineoffset: 0 }
      )),
      Ok((
        Location{ lineno: 1, linepos: 1, lineoffset: 0 },
        Tok::Minus,
        Location{ lineno: 1, linepos: 2, lineoffset: 0 }
      )),
      Ok((
        Location{ lineno: 1, linepos: 2, lineoffset: 0 },
        Tok::Mult,
        Location{ lineno: 1, linepos: 3, lineoffset: 0 }
      )),
      Ok((
        Location{ lineno: 1, linepos: 3, lineoffset: 0 },
        Tok::Div,
        Location{ lineno: 1, linepos: 4, lineoffset: 0 }
      )),
      Ok((
        Location{ lineno: 1, linepos: 4, lineoffset: 0 },
        Tok::LBracket,
        Location{ lineno: 1, linepos: 5, lineoffset: 0 }
      )),
      Ok((
        Location{ lineno: 1, linepos: 5, lineoffset: 0 },
        Tok::RBracket,
        Location{ lineno: 1, linepos: 6, lineoffset: 0 }
      )),
      Ok((
        Location{ lineno: 1, linepos: 6, lineoffset: 0 },
        Tok::LBrace,
        Location{ lineno: 1, linepos: 7, lineoffset: 0 }
      )),
      Ok((
        Location{ lineno: 1, linepos: 7, lineoffset: 0 },
        Tok::RBrace,
        Location{ lineno: 1, linepos: 8, lineoffset: 0 }
      )),
      Ok((
        Location{ lineno: 1, linepos: 8, lineoffset: 0 },
        Tok::LParent,
        Location{ lineno: 1, linepos: 9, lineoffset: 0 }
      )),
      Ok((
        Location{ lineno: 1, linepos: 9, lineoffset: 0 },
        Tok::RParent,
        Location{ lineno: 1, linepos: 10, lineoffset: 0 }
      )),
      Ok((
        Location{ lineno: 1, linepos: 10, lineoffset: 0 },
        Tok::Semi,
        Location{ lineno: 1, linepos: 11, lineoffset: 0 }
      )),
      Ok((
        Location{ lineno: 1, linepos: 11, lineoffset: 0 },
        Tok::Comma,
        Location{ lineno: 1, linepos: 12, lineoffset: 0 }
      ))
    ]);
  }

  #[test]
  fn test_composite_symbols() {
    let source = "->==//";
    let lexer = Lexer::new(source);
    assert_eq!(lexer.collect::<Vec<_>>(), vec![
      Ok((
        Location{ lineno: 1, linepos: 0, lineoffset: 0 },
        Tok::Arrow,
        Location{ lineno: 1, linepos: 2, lineoffset: 0 }
      )),
      Ok((
        Location{ lineno: 1, linepos: 2, lineoffset: 0 },
        Tok::Equal,
        Location{ lineno: 1, linepos: 4, lineoffset: 0 }
      ))
    ]);
  }

  #[test]
  fn test_keywords() {
    for (keyword, token) in get_keywords() {
      let lexer = Lexer::new(&keyword);
      assert_eq!(
        lexer.collect::<Vec<_>>(), vec![
          Ok((
            Location{ lineno: 1, linepos: 0, lineoffset: 0 },
            token,
            Location{ lineno: 1, linepos: keyword.len(), lineoffset: 0 }
          ))
        ]);
    }
  }

  #[test]
  fn test_gates() {
    let source = "CX U";
    let lexer = Lexer::new(source);
    assert_eq!(lexer.collect::<Vec<_>>(), vec![
      Ok((
        Location{ lineno: 1, linepos: 0, lineoffset: 0 },
        Tok::CX,
        Location{ lineno: 1, linepos: 2, lineoffset: 0 }
      )),
      Ok((
        Location{ lineno: 1, linepos: 3, lineoffset: 0 },
        Tok::U,
        Location{ lineno: 1, linepos: 4, lineoffset: 0 }
      ))
    ]);
  }

  #[test]
  fn test_ids() {
    let source = "a b c";
    let lexer = Lexer::new(source);
    assert_eq!(lexer.collect::<Vec<_>>(), vec![
      Ok((
        Location{ lineno: 1, linepos: 0, lineoffset: 0 },
        Tok::Id { repr: "a".into() },
        Location{ lineno: 1, linepos: 1, lineoffset: 0 }
      )),
      Ok((
        Location{ lineno: 1, linepos: 2, lineoffset: 0 },
        Tok::Id { repr: "b".into() },
        Location{ lineno: 1, linepos: 3, lineoffset: 0 }
      )),
      Ok((
        Location{ lineno: 1, linepos: 4, lineoffset: 0 },
        Tok::Id { repr: "c".into() },
        Location{ lineno: 1, linepos: 5, lineoffset: 0 }
      ))
    ]);
  }

  #[test]
  fn test_dont_admit_all_mayus_ids() {
    let source = "a B c";
    let lexer = Lexer::new(source);
    assert_eq!(lexer.collect::<Vec<_>>(), vec![
      Ok((
        Location{ lineno: 1, linepos: 0, lineoffset: 0 },
        Tok::Id { repr: "a".into() },
        Location{ lineno: 1, linepos: 1, lineoffset: 0 }
      )),
      Err(LexicalError { location: Location {
        lineno: 1,
        linepos: 2,
        lineoffset: 0
      }})
    ]);
  }

  mod regressions {
    use super::*;

    #[test]
    fn test_gate_call() {
      let source = "U(pi/2, 0, pi) q;";
      let lexer = Lexer::new(source);
      assert_eq!(lexer.collect::<Vec<_>>(), vec![
        Ok((
          Location{ lineno: 1, linepos: 0, lineoffset: 0 },
          Tok::U,
          Location{ lineno: 1, linepos: 1, lineoffset: 0 }
        )),
        Ok((
          Location{ lineno: 1, linepos: 1, lineoffset: 0 },
          Tok::LParent,
          Location{ lineno: 1, linepos: 2, lineoffset: 0 }
        )),
        Ok((
          Location{ lineno: 1, linepos: 2, lineoffset: 0 },
          Tok::ConstPi,
          Location{ lineno: 1, linepos: 4, lineoffset: 0 }
        )),
        Ok((
          Location{ lineno: 1, linepos: 4, lineoffset: 0 },
          Tok::Div,
          Location{ lineno: 1, linepos: 5, lineoffset: 0 }
        )),
        Ok((
          Location{ lineno: 1, linepos: 5, lineoffset: 0 },
          Tok::Int { repr: String::from("2")},
          Location{ lineno: 1, linepos: 6, lineoffset: 0 }
        )),
        Ok((
          Location{ lineno: 1, linepos: 6, lineoffset: 0 },
          Tok::Comma,
          Location{ lineno: 1, linepos: 7, lineoffset: 0 }
        )),
        Ok((
          Location{ lineno: 1, linepos: 8, lineoffset: 0 },
          Tok::Int { repr: String::from("0")},
          Location{ lineno: 1, linepos: 9, lineoffset: 0 }
        )),
        Ok((
          Location{ lineno: 1, linepos: 9, lineoffset: 0 },
          Tok::Comma,
          Location{ lineno: 1, linepos: 10, lineoffset: 0 }
        )),
        Ok((
          Location{ lineno: 1, linepos: 11, lineoffset: 0 },
          Tok::ConstPi,
          Location{ lineno: 1, linepos: 13, lineoffset: 0 }
        )),
        Ok((
          Location{ lineno: 1, linepos: 13, lineoffset: 0 },
          Tok::RParent,
          Location{ lineno: 1, linepos: 14, lineoffset: 0 }
        )),
        Ok((
          Location{ lineno: 1, linepos: 15, lineoffset: 0 },
          Tok::Id { repr: String::from("q") },
          Location{ lineno: 1, linepos: 16, lineoffset: 0 }
        )),
        Ok((
          Location{ lineno: 1, linepos: 16, lineoffset: 0 },
          Tok::Semi,
          Location{ lineno: 1, linepos: 17, lineoffset: 0 }
        )),
      ]);
    }

    #[test]
    fn test_error_at_the_begining() {
      let source = "XXX"; // unrecognized ID (all caps), and not a keyword.
      let lexer = Lexer::new(source);
      assert_eq!(lexer.collect::<Vec<_>>(), vec![
        Err(LexicalError { location: Location {
          lineno: 1,
          linepos: 0,
          lineoffset: 0
        }})
      ]);
    }
  }
}