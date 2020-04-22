use std::collections::HashMap;

use crate::grammar::open_qasm2;
use crate::grammar::ast;
use crate::grammar::{ Location, Lexer };

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LinkerError {
  LibraryNotFound { location: Location, libpath: String }
}

#[derive(Debug, Clone, Default)]
pub struct Linker {
  embedded: HashMap<String, String>
}

type Result<T> = std::result::Result<T, LinkerError>;

impl Linker {
  pub fn new() -> Self {
    Linker{embedded: Default::default()}
  }

  pub fn with_embedded(embedded: HashMap<String, String>) -> Self {
    Linker{embedded}
  }

  pub fn link(&self, mut tree: ast::OpenQasmProgram) -> Result<ast::OpenQasmProgram> {
    let mut to_embed = vec![];
    for (index, span) in tree.program.iter().enumerate() {
      if let ast::Statement::Include(libpath) = &*span.node {
        let source = self.sources(&libpath).map_err(|_| {
          LinkerError::LibraryNotFound { location: span.boundaries.0.clone(), libpath: libpath.into() }
        })?;
        let lexer = Lexer::new(&source);
        let parser = open_qasm2::OpenQasmLibraryParser::new();
        let library_tree = parser.parse(lexer).unwrap();
        to_embed.push((index, span.boundaries.clone(), library_tree.definitions));
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
      tree.program.splice(index..=index, inner_spans);
    }
    Ok(tree)
  }

  fn sources(&self, libpath: &str) -> std::result::Result<String, ()> {
    if self.embedded.contains_key(libpath) {
      return Ok(self.embedded.get(libpath).unwrap().clone());
    }
    Err(())
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