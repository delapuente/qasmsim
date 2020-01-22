use std::collections::HashMap;

use open_qasm2;
use grammar::ast;

pub struct Linker {
  embedded: HashMap<String, String>
}

impl Linker {
  pub fn with_embedded(embedded: HashMap<String, String>) -> Self {
    Linker{embedded}
  }

  pub fn link(&self, mut tree: ast::OpenQasmProgram) -> Result<ast::OpenQasmProgram, String> {
    let mut to_embed = vec![];
    for (index, statement) in tree.program.iter().enumerate() {
      match statement {
        ast::Statement::Include(libpath) => {
          let parser = open_qasm2::OpenQasmLibraryParser::new();
          let source = self.get_sources(libpath)?;
          let library_tree = parser.parse(&source).unwrap();
          to_embed.push((index, library_tree.definitions));
        },
        _ => ()
      }
    }
    to_embed.reverse();
    for (index, statements) in to_embed {
      tree.program.splice(index..index+1, statements);
    }
    Ok(tree)
  }

  fn get_sources(&self, libpath: &str) -> Result<String, String> {
    if self.embedded.contains_key(libpath) {
      return Ok(self.embedded.get(libpath).unwrap().clone());
    }
    Err(format!("Library `{}` not found", libpath))
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  use std::iter::FromIterator;

  #[test]
  fn test_linker_loads_embedded_libraries() {
    let source = "
    OPENQASM 2.0;
    include \"test.inc\";
    ";
    let linker = Linker::with_embedded(HashMap::from_iter(vec![
      ("test.inc".to_owned(), "gate test () q {}".to_owned())
    ]));
    let parser = open_qasm2::OpenQasmProgramParser::new();
    let tree = parser.parse(&source).unwrap();
    let linked_tree = linker.link(tree).unwrap();
    assert_eq!(linked_tree, ast::OpenQasmProgram{
      version: "2.0".to_owned(),
      program: vec![
        ast::Statement::GateDecl(
          "test".to_owned(),
          vec![], vec!["q".to_string()], vec![]
        )
      ]
    })
  }
}