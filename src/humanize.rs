use std::fmt::{ self, Write };

use crate::error::{ QasmSimError, ErrorKind, RuntimeKind };

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
    QasmSimError::SyntaxError {
      kind,
      source,
      lineoffset,
      lineno,
      startpos,
      endpos,
      token,
      expected
    } =>
    match kind {
      ErrorKind::InvalidToken => {
        let linesrc = get_line_src(*lineoffset, source);
        Some(HumanDescription {
          msg: "invalid token".into(),
          lineno: *lineno,
          startpos: *startpos,
          endpos: *endpos,
          linesrc: Some(linesrc.into()),
          help: None
        })
      }
      ErrorKind::UnexpectedEOF => {
        let expectation = expectation(&expected);
        let linesrc = get_line_src(*lineoffset, source);
        Some(HumanDescription {
          msg: format!("{}, found EOF", &expectation),
          lineno: *lineno,
          startpos: *startpos,
          endpos: *endpos,
          linesrc: Some(linesrc.into()),
          help: Some(format!("{} here", hint(&expected)))
        })
      }
      ErrorKind::UnexpectedToken => {
        let token = token.as_ref().unwrap();
        let linesrc = get_line_src(*lineoffset, source);
        let endpos = std::cmp::min(endpos.unwrap(), linesrc.len());

        let mut msg = format!("unexpected \"{}\" found", &token);
        let mut help = None;
        if expected.len() > 0 {
          let expectation = expectation(&expected);
          msg = format!("{}, found \"{}\"", &expectation, &token);
          help =  Some(format!("{} before this", hint(&expected)));
        }

        Some(HumanDescription {
          msg,
          lineno: *lineno,
          startpos: *startpos,
          endpos: Some(endpos),
          linesrc: Some(linesrc.into()),
          help
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
      let description: HumanDescription = get_human_description(error).expect("some human description");
      humanize(buffer, &description)
    }
    QasmSimError::SemanticError { symbol_name } => {
      write!(buffer, "symbol `{}` is declared twice", &symbol_name)
    }
    QasmSimError::LinkerError { libpath } => {
      write!(buffer, "cannot find library `{}`", &libpath)
    }
    QasmSimError::RuntimeError { kind, symbol_name } => match kind {
      RuntimeKind::ClassicalRegisterNotFound => {
        write!(buffer, "classical register `{}` not found in this scope", &symbol_name)
      }
      RuntimeKind::QuantumRegisterNotFound => {
        write!(buffer, "quantum register `{}` not found in this scope", &symbol_name)
      }
      RuntimeKind::SymbolNotFound => {
        write!(buffer, "symbol `{}` not found in this scope", &symbol_name)
      }
      RuntimeKind::UndefinedGate => {
        write!(buffer, "error calling gate `{}`: gate not found in this scope", &symbol_name)
      }
      RuntimeKind::WrongNumberOfQuantumParameters => {
        write!(buffer, "error calling gate `{}`: the number of quantum parameters is wrong", &symbol_name)
      }
      RuntimeKind::WrongNumberOfRealParameters => {
        write!(buffer, "error calling gate `{}`: the number of real parameters is wrong", &symbol_name)
      }
      RuntimeKind::IndexOutOfBounds => {
        write!(buffer, "index out of bounds for register `{}`", &symbol_name)
      },
      RuntimeKind::DifferentSizeRegisters => {
        write!(buffer, "cannot apply gate to registers of different size")
      }
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

fn get_line_src(linestart: usize, doc: &str) -> &str {
  assert!(linestart <= doc.len(),
    "linestart={} must in the range 0..=doc.len()={}", linestart, doc.len());

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

  &doc[linestart..lineend]
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

  use super::get_line_src;

  macro_rules! test_get_line_src {
    ($source:expr, $( $name:ident: $offset:expr => $expected:expr ),*) => {
      $(
        #[test]
        fn $name() {
          assert_eq!(get_line_src($offset, &$source), $expected);
        }
      )*
    };
  }

  test_get_line_src!(indoc!("
      line 1
      line 2
      line 3"
    ),
    test_beginning_of_source: 0 => "line 1\n",
    test_middle_of_source: 7 => "line 2\n",
    test_last_character: 14 => "line 3"
  );
}