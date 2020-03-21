use std::convert;
use std::fmt::{ self, Write };

use crate::error::QasmSimError;
use crate::grammar::{ ParseError, Location };

#[derive(Debug, Clone, PartialEq)]
pub struct HumanDescription {
  msg: String,
  lineno: usize,
  startpos: usize,
  endpos: Option<usize>,
  linesrc: Option<String>,
  help: Option<String>
}

fn get_human_description(error: &QasmSimError) -> Option<HumanDescription> {
  match error {
    QasmSimError::SyntaxError { error, source } => match error {
      ParseError::InvalidToken { location } => {
        let (lineno, startpos, linesrc) = into_doc_coords(location, source);
        Some(HumanDescription {
          msg: "invalid token".into(),
          lineno,
          startpos,
          endpos: None,
          linesrc: Some(linesrc.into()),
          help: None
        })
      }
      ParseError::UnrecognizedEOF { location, expected } => {
        let expectation = expectation(&expected);
        let (lineno, startpos, linesrc) = into_doc_coords(location, source);
        Some(HumanDescription {
          msg: format!("{}, found EOF", &expectation),
          lineno,
          startpos,
          endpos: None,
          linesrc: Some(linesrc.into()),
          help: Some(format!("{} here", hint(&expected)))
        })
      }
      ParseError::UnrecognizedToken { token, expected } => {
        let (start, token, end) = token;
        let expectation = expectation(&expected);
        let (lineno, startpos, linesrc) = into_doc_coords(start, source);
        let endpos = if end.linepos >= linesrc.len() {
          linesrc.len()
        }
        else {
          end.linepos
        };
        Some(HumanDescription {
          msg: format!("{}, found \"{}\"", &expectation, &token),
          lineno,
          startpos,
          endpos: Some(endpos),
          linesrc: Some(linesrc.into()),
          help: Some(format!("{} before this", hint(&expected)))
        })
      }
      ParseError::ExtraToken { token } => {
        let (start, token, end) = token;
        let (lineno, startpos, linesrc) = into_doc_coords(start, source);
        let (_, endpos, _) = into_doc_coords(end, source);
        Some(HumanDescription {
          msg: format!("unexpected \"{}\" found", &token),
          lineno,
          startpos,
          endpos: Some(endpos),
          linesrc: Some(linesrc.into()),
          help: None
        })
      }
      ParseError::User { error } => {
        // Transform into InvalidToken and launch the conversion again
        get_human_description(&QasmSimError::SyntaxError {
          source,
          error: ParseError::InvalidToken{ location: error.location.clone() },
        })
      }
    }
    _ => None
  }
}

pub fn humanize_error(buffer: &mut String, error: &QasmSimError) -> fmt::Result {
  match error {
    QasmSimError::UnknownError(msg) => write!(buffer, "{}", msg),
    QasmSimError::SyntaxError { .. } => {
      let description: HumanDescription = get_human_description(error).expect("other cases should be covered");
      humanize(buffer, &description)
    }
  }
}

fn humanize(buffer: &mut String, descripition: &HumanDescription) -> fmt::Result {
  match descripition {
    HumanDescription { msg, lineno, startpos, endpos, linesrc, help } => {
      let lineno_str = format!("{} ", lineno);
      let lineno_len = lineno_str.len();
      let linesrc_str = linesrc.clone().unwrap_or("".into());
      let linesrc_str_trimmed = linesrc_str.trim_end();
      let help_str = help.clone().unwrap_or(msg.clone());
      let indicator_width = if let Some(pos) = endpos { pos - startpos } else { 1 };

      writeln!(buffer, "error: {}", msg)?;
      writeln!(buffer, "{:>alignment$}|", "", alignment = lineno_len)?;
      writeln!(buffer, "{}| {}", lineno_str, linesrc_str_trimmed)?;
      writeln!(buffer, "{:>alignment$}| {:>padding$}{:^>indicator_width$} help: {}",
        "", "", "", help_str,
        alignment = lineno_str.len(), padding = startpos,
        indicator_width = indicator_width)?;

      fmt::Result::Ok(())
    }
  }
}

// TODO: Just used to extract the src line. Before, used to translate from a
// source offset into a source coordinates (line number, pos in line). Now this
// information is the Location struct.
fn into_doc_coords<'loc, 'src>(pos: &'loc Location, doc: &'src str) -> (usize, usize, &'src str) {
  assert!(pos.lineoffset + pos.linepos <= doc.len(),
    "pos.lineoffset + pos.linepos={} must in the range 0..=doc.len()={}",
    pos.lineoffset + pos.linepos, doc.len());

  let lineno = pos.lineno;
  let startpos = pos.linepos;
  let linestart = pos.lineoffset;
  let mut lineend = linestart + 1;

  for c in doc[linestart..].chars() {
    if c == '\n' {
      break;
    }
    lineend += 1;
  }

  if lineend > doc.len() {
    lineend = doc.len();
  }
  (lineno, startpos, &doc[linestart..lineend])
}

fn expectation(expected: &Vec<String>) -> String {
  let choices = list_of_choices(expected).expect("len() is greater than 0");
  format!("expected {}", choices)
}

fn hint(expected: &Vec<String>) -> String {
  let choices = list_of_choices(expected).expect("len() is greater than 0");
  format!("consider adding {}{}",
    if choices.len() == 1 { "one of " } else { "" }, choices)
}

fn list_of_choices(choices: &Vec<String>) -> Option<String> {
  let len = choices.len();
  match len {
    0 => None,
    1 => Some(choices.first().unwrap().clone()),
    _ => Some({
      let last = choices.last().unwrap();
      let except_last: Vec<String> =
        choices.iter().take(len - 1).map(|item| (*item).clone()).collect();
      format!("{}, or {}", except_last.join(", "), last)
    })
  }
}

#[cfg(test)]
mod test_humanize_error {
  use indoc::indoc;

  use super::*;

  #[test]
  fn test_eof_error() {
    let error = HumanDescription {
      msg: r#"expected ";", found EOF"#.into(),
      lineno: 777,
      startpos: 10,
      endpos: None,
      linesrc: Some("qreg q[10]".into()),
      help: Some(r#"add ";" here"#.into())
    };
    let mut buffer = String::new();
    humanize(&mut buffer, &error).expect("should not fail");
    assert_eq!(buffer, indoc!(r#"
      error: expected ";", found EOF
          |
      777 | qreg q[10]
          |           ^ help: add ";" here
    "#));
  }

  #[test]
  fn test_complete_error() {
    let error = HumanDescription {
      msg: r#"expected ";", found "qreg""#.into(),
      lineno: 778,
      startpos: 0,
      endpos: Some(4),
      linesrc: Some("qreg r[10]\n".into()),
      help: Some(r#"add ";" at the end of the previous line"#.into())
    };
    let mut buffer = String::new();
    humanize(&mut buffer, &error).expect("should not fail");
    assert_eq!(buffer, indoc!(r#"
      error: expected ";", found "qreg"
          |
      778 | qreg r[10]
          | ^^^^ help: add ";" at the end of the previous line
    "#));
  }

  #[test]
  fn test_no_hint_error() {
    let error = HumanDescription {
      msg: r#"unexpected keyword `qreg` found"#.into(),
      lineno: 778,
      startpos: 0,
      endpos: Some(4),
      linesrc: Some("qreg r[10]\n".into()),
      help: None
    };
    let mut buffer = String::new();
    humanize(&mut buffer, &error).expect("should not fail");
    assert_eq!(buffer, indoc!(r#"
      error: unexpected keyword `qreg` found
          |
      778 | qreg r[10]
          | ^^^^ help: unexpected keyword `qreg` found
    "#));
  }

  #[test]
  fn test_trim_line_source_end() {
    let error = HumanDescription {
      msg: r#"unexpected keyword `qreg` found"#.into(),
      lineno: 778,
      startpos: 0,
      endpos: Some(4),
      linesrc: Some("qreg r[10]    \n".into()),
      help: None
    };
    let mut buffer = String::new();
    humanize(&mut buffer, &error).expect("should not fail");
    assert_eq!(buffer, indoc!(r#"
      error: unexpected keyword `qreg` found
          |
      778 | qreg r[10]
          | ^^^^ help: unexpected keyword `qreg` found
    "#));
  }

  #[test]
  fn test_preserve_line_source_start() {
    let error = HumanDescription {
      msg: r#"unexpected keyword `qreg` found"#.into(),
      lineno: 778,
      startpos: 2,
      endpos: Some(6),
      linesrc: Some("  qreg r[10]    \n".into()),
      help: None
    };
    let mut buffer = String::new();
    humanize(&mut buffer, &error).expect("should not fail");
    assert_eq!(buffer, indoc!(r#"
      error: unexpected keyword `qreg` found
          |
      778 |   qreg r[10]
          |   ^^^^ help: unexpected keyword `qreg` found
    "#));
  }
}

#[cfg(test)]
mod test_into_doc_coords {
  use indoc::indoc;

  use super::{ into_doc_coords, Location };

  macro_rules! test_into_doc_coords {
    ($source:expr, $( $name:ident: $offset:expr => $expected:expr ),*) => {
      $(
        #[test]
        fn $name() {
          assert_eq!(into_doc_coords($offset, &$source), $expected);
        }
      )*
    };
  }

  test_into_doc_coords!(indoc!("
      line 1
      line 2
      line 3"
    ),
    test_beginning_of_source:
      &Location { lineno: 1, linepos: 0, lineoffset: 0 } => (1, 0, "line 1\n"),
    test_middle_of_source:
      &Location { lineno: 2, linepos: 4, lineoffset: 7 } => (2, 4, "line 2\n"),
    test_last_character:
      &Location { lineno: 3, linepos: 6, lineoffset: 14 } => (3, 6, "line 3"),
    test_end_of_source:
      &Location { lineno: 3, linepos: 6, lineoffset: 14 } => (3, 6, "line 3")
  );
}