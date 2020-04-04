use std::collections::HashMap;

use crate::api;
use crate::error::QasmSimError;
use crate::grammar::open_qasm2;
use crate::grammar::ast;
use crate::grammar::Lexer;

pub struct Linker {
  embedded: HashMap<String, String>
}

impl<'src> Linker {
  pub fn with_embedded(embedded: HashMap<String, String>) -> Self {
    Linker{embedded}
  }

  pub fn link(&self, mut tree: ast::OpenQasmProgram) -> api::Result<'src, ast::OpenQasmProgram> {
    let mut to_embed = vec![];
    for (index, span) in tree.program.iter().enumerate() {
      match &*span.node {
        ast::Statement::Include(libpath) => {
          let source = self.get_sources(&libpath)?;
          let lexer = Lexer::new(&source);
          let parser = open_qasm2::OpenQasmLibraryParser::new();
          let library_tree = parser.parse(lexer).unwrap();
          to_embed.push((index, span.boundaries.clone(), library_tree.definitions));
        },
        _ => ()
      }
    }
    to_embed.reverse();
    for (index, boundaries, statements) in to_embed {
      let mut inner_spans = vec![];
      for one_statement in statements {
        inner_spans.push(ast::Span {
          boundaries: boundaries.clone(),
          node: Box::new(one_statement)
        })
      }
      tree.program.splice(index..index+1, inner_spans);
    }
    Ok(tree)
  }

  fn get_sources(&self, libpath: &str) -> api::Result<'src, String> {
    if self.embedded.contains_key(libpath) {
      return Ok(self.embedded.get(libpath).unwrap().clone());
    }
    Err(QasmSimError::LinkerError { libpath: libpath.into() })
  }
}


#[cfg(test)]
mod tests {
  use std::iter::FromIterator;

  use indoc::indoc;

  use crate::grammar::{ ast::Span, Location };

  use super::*;

  macro_rules! span {
    ($left:expr, $node:expr, $right:expr) => {
      Span {
        boundaries: (Location($left), Location($right)),
        node: Box::new($node)
      }
    };
  }


  #[test]
  fn test_linker_loads_embedded_libraries() {
    let source = indoc!("
    OPENQASM 2.0;
    include \"test.inc\";
    ");
    let linker = Linker::with_embedded(HashMap::from_iter(vec![
      ("test.inc".to_owned(), "gate test () q {}".to_owned())
    ]));
    let lexer = Lexer::new(&source);
    let parser = open_qasm2::OpenQasmProgramParser::new();
    let tree = parser.parse(lexer).unwrap();
    let linked_tree = linker.link(tree).unwrap();
    assert_eq!(linked_tree, ast::OpenQasmProgram{
      version: "2.0".to_owned(),
      program: vec![
        span!(14, ast::Statement::GateDecl(
          "test".to_owned(),
          vec![], vec!["q".to_string()], vec![]
        ), 33)
      ]
    })
  }
}