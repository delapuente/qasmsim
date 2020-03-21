use std::fmt::{ self, Write };

use crate::error::QasmSimError;

pub fn humanize_error(f: &mut String, syntax_err: &QasmSimError) -> fmt::Result {
  match &syntax_err {
    QasmSimError::SyntaxError { msg, lineno, startpos, endpos, linesrc, help } => {
      let lineno_str = format!("{} ", lineno);
      let lineno_len = lineno_str.len();
      let linesrc_str = linesrc.clone().unwrap_or("".into());
      let linesrc_str_trimmed = linesrc_str.trim_end();
      let help_str = help.clone().unwrap_or(msg.clone());
      let indicator_width = if let Some(pos) = endpos { pos - startpos } else { 1 };

      writeln!(f, "error: {}", msg)?;
      writeln!(f, "{:>alignment$}|", "", alignment = lineno_len)?;
      writeln!(f, "{}| {}", lineno_str, linesrc_str_trimmed)?;
      writeln!(f, "{:>alignment$}| {:>padding$}{:^>indicator_width$} help: {}",
        "", "", "", help_str,
        alignment = lineno_str.len(), padding = startpos,
        indicator_width = indicator_width)?;

      fmt::Result::Ok(())
    }
    _ => unreachable!()
  }
}

#[cfg(test)]
mod test {
  use indoc::indoc;

  use super::*;

  #[test]
  fn test_eof_error() {
    let error = QasmSimError::SyntaxError {
      msg: r#"expected ";", found EOF"#.into(),
      lineno: 777,
      startpos: 10,
      endpos: None,
      linesrc: Some("qreg q[10]".into()),
      help: Some(r#"add ";" here"#.into())
    };
    let mut buffer = String::new();
    humanize_error(&mut buffer, &error).expect("should not fail");
    assert_eq!(buffer, indoc!(r#"
      error: expected ";", found EOF
          |
      777 | qreg q[10]
          |           ^ help: add ";" here
    "#));
  }

  #[test]
  fn test_complete_error() {
    let error = QasmSimError::SyntaxError {
      msg: r#"expected ";", found "qreg""#.into(),
      lineno: 778,
      startpos: 0,
      endpos: Some(4),
      linesrc: Some("qreg r[10]\n".into()),
      help: Some(r#"add ";" at the end of the previous line"#.into())
    };
    let mut buffer = String::new();
    humanize_error(&mut buffer, &error).expect("should not fail");
    assert_eq!(buffer, indoc!(r#"
      error: expected ";", found "qreg"
          |
      778 | qreg r[10]
          | ^^^^ help: add ";" at the end of the previous line
    "#));
  }

  #[test]
  fn test_no_hint_error() {
    let error = QasmSimError::SyntaxError {
      msg: r#"unexpected keyword `qreg` found"#.into(),
      lineno: 778,
      startpos: 0,
      endpos: Some(4),
      linesrc: Some("qreg r[10]\n".into()),
      help: None
    };
    let mut buffer = String::new();
    humanize_error(&mut buffer, &error).expect("should not fail");
    assert_eq!(buffer, indoc!(r#"
      error: unexpected keyword `qreg` found
          |
      778 | qreg r[10]
          | ^^^^ help: unexpected keyword `qreg` found
    "#));
  }

  #[test]
  fn test_trim_line_source_end() {
    let error = QasmSimError::SyntaxError {
      msg: r#"unexpected keyword `qreg` found"#.into(),
      lineno: 778,
      startpos: 0,
      endpos: Some(4),
      linesrc: Some("qreg r[10]    \n".into()),
      help: None
    };
    let mut buffer = String::new();
    humanize_error(&mut buffer, &error).expect("should not fail");
    assert_eq!(buffer, indoc!(r#"
      error: unexpected keyword `qreg` found
          |
      778 | qreg r[10]
          | ^^^^ help: unexpected keyword `qreg` found
    "#));
  }

  #[test]
  fn test_preserve_line_source_start() {
    let error = QasmSimError::SyntaxError {
      msg: r#"unexpected keyword `qreg` found"#.into(),
      lineno: 778,
      startpos: 2,
      endpos: Some(6),
      linesrc: Some("  qreg r[10]    \n".into()),
      help: None
    };
    let mut buffer = String::new();
    humanize_error(&mut buffer, &error).expect("should not fail");
    assert_eq!(buffer, indoc!(r#"
      error: unexpected keyword `qreg` found
          |
      778 |   qreg r[10]
          |   ^^^^ help: unexpected keyword `qreg` found
    "#));
  }
}
