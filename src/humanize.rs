use std::fmt::{ self, Write };

use crate::error::QasmSimError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HumanDescription {
  msg: String,
  lineno: usize,
  startpos: usize,
  endpos: Option<usize>,
  linesrc: String,
  help: Option<String>
}

fn human_description(error: &QasmSimError) -> Option<HumanDescription> {
  match error {
    QasmSimError::InvalidToken {
      source,
      lineno,
      startpos,
      endpos,
      ..
    } => {
      Some(HumanDescription {
        msg: "invalid token".into(),
        lineno: *lineno,
        startpos: *startpos,
        endpos: *endpos,
        linesrc: (*source).into(),
        help: None
      })
    }
    QasmSimError::UnexpectedEOF {
      source,
      lineno,
      startpos,
      endpos,
      expected,
      ..
    } => {
      let expectation = expectation(&expected);
      Some(HumanDescription {
        msg: format!("{}, found EOF", &expectation),
        lineno: *lineno,
        startpos: *startpos,
        endpos: *endpos,
        linesrc: (*source).into(),
        help: Some(format!("{} here", hint(&expected)))
      })
    }
    QasmSimError::UnexpectedToken {
      source,
      lineno,
      startpos,
      endpos,
      token,
      expected
    } => {
      let token = token.as_ref().unwrap();
      let endpos = std::cmp::min(endpos.unwrap(), source.len());

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
        linesrc: (*source).into(),
        help
      })
    }
    QasmSimError::SemanticError {
      source,
      lineno,
      previous_lineno,
      symbol_name
    } => {
      Some(HumanDescription {
        msg: format!("cannot declare symbol `{}` twice", symbol_name),
        lineno: *lineno,
        startpos: 0,
        endpos: None,
        linesrc: (*source).into(),
        help: Some(format!("first declaration happens in line {}", *previous_lineno))
      })
    }
    QasmSimError::IndexOutOfBounds {
      symbol_name,
      source,
      lineno,
      index,
      size
    } => {
      Some(HumanDescription {
        msg: format!("index out of bounds"),
        lineno: *lineno,
        startpos: 0,
        endpos: None,
        linesrc: (*source).into(),
        help: Some(format!("indices of register `{}` range from 0 to {} but the index is {}", symbol_name, size - 1, index))
      })
    }
    QasmSimError::SymbolNotFound {
      source,
      symbol_name,
      lineno,
      expected
    } => {
      Some(HumanDescription {
        msg: format!("cannot find the {} `{}` in this scope", expected, symbol_name),
        lineno: *lineno,
        startpos: 0,
        endpos: None,
        linesrc: (*source).into(),
        help: None
      })
    }
    QasmSimError::TypeMismatch {
      source,
      symbol_name,
      lineno,
      expected
    } => {
      Some(HumanDescription {
        msg: format!("mismatched types for symbol `{}`: expected \"{}\"", symbol_name, expected),
        linesrc: (*source).into(),
        lineno: *lineno,
        startpos: 0,
        endpos: None,
        help: None
      })
    }
    QasmSimError::RegisterSizeMismatch {
      source,
      symbol_name,
      lineno,
      sizes,
    } => {
      let sizes_str: Vec<String> = sizes.iter().map(|size| format!("{}", size)).collect();
      let msg = if symbol_name == "measure" {
        "cannot apply a measurement between registers of different sizes".into()
      }
      else {
        format!("cannot apply gate `{}` to registers of differen size", symbol_name)
      };
      Some(HumanDescription {
        msg,
        linesrc: (*source).into(),
        lineno: *lineno,
        startpos: 0,
        endpos: None,
        help: Some(format!("expected registers of same size, found registers of sizes {}", sizes_str.join(", ")))
      })
    }
    QasmSimError::WrongNumberOfParameters {
      source,
      symbol_name,
      are_registers,
      lineno,
      expected,
      given
    } => {
      let qualifier = if *are_registers { "quantum registers" } else { "real parameters" };
      Some(HumanDescription {
        msg: format!("wrong number of {} passed to gate `{}`", qualifier, symbol_name),
        linesrc: (*source).into(),
        lineno: *lineno,
        startpos: 0,
        endpos: None,
        help: Some(format!("expected {} {}, given {}", expected, qualifier, given))
      })
    }
    QasmSimError::UndefinedGate {
      source,
      symbol_name,
      lineno,
    } => {
      Some(HumanDescription {
        msg: format!("cannot find gate `{}` in this scope", symbol_name),
        linesrc: (*source).into(),
        lineno: *lineno,
        startpos: 0,
        endpos: None,
        help: None
      })
    }
    QasmSimError::LibraryNotFound {
      source,
      lineno,
      libpath
    } => {
      Some(HumanDescription {
        msg: format!("cannot find library `{}`", libpath),
        linesrc: (*source).into(),
        lineno: *lineno,
        startpos: 0,
        endpos: None,
        help: None
      })
    }
    _ => None
  }
}

pub fn humanize_error<W: Write>(buffer: &mut W, error: &QasmSimError) -> fmt::Result {
  match error {
    QasmSimError::UnknownError(msg) => write!(buffer, "{}", msg),
    _ => {
      let description: HumanDescription = human_description(error).expect("some human description");
      humanize(buffer, &description)
    }
  }
}

fn humanize<W: Write>(buffer: &mut W, descripition: &HumanDescription) -> fmt::Result {
  match descripition {
    HumanDescription { msg, lineno, startpos, endpos, linesrc, help } => {
      let lineno_str = format!("{} ", lineno);
      let lineno_len = lineno_str.len();
      let linesrc_str: String = linesrc.into();
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
      linesrc: "qreg q[10]".into(),
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
      linesrc: "qreg r[10]\n".into(),
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
      linesrc: "qreg r[10]\n".into(),
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
      linesrc: "qreg r[10]    \n".into(),
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
      linesrc: "  qreg r[10]    \n".into(),
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
