//! Contain utilities for combining multiple AST spread into several locations.
//! The module is **unstable**.

use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::grammar::ast;
use crate::grammar::lexer::Location;
use crate::grammar::parse_library;

/// Represent a filure during linkage.
///
/// # Examples
///
/// The following typo in OPENQASM:
///
/// ```qasm
/// OPENQASM 2.0;
/// include "qlib.inc"  // instead of "qelib1.inc"
/// ```
///
/// Would cause the following error:
///
/// ```qasm
/// use qasmsim::grammar::lexer::Location;
/// use qasmsim::linker::LinkerError;
///
/// LinkerError::LibraryNotFound {
///     location: Location(14),
///     libpath: "qlib.inc".to_string()
/// }
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LinkerError {
    /// The `include` directive at `location` failed while importing `libpath`.
    LibraryNotFound {
        /// Location of the `include` directive.
        location: Location,
        /// Library path passed to the `include` directive.
        libpath: String,
    },
}

#[derive(Debug, Clone, Default)]
pub struct Linker {
    embedded: HashMap<String, String>,
}

type Result<T> = std::result::Result<T, LinkerError>;

impl Linker {
    /// Create a new linker with a hashmap relating paths with embedded sources.
    pub fn with_embedded(embedded: HashMap<String, String>) -> Self {
        Linker { embedded }
    }

    /// Look into `tree` for `include` statements, parse the referred libraries,
    /// and integrate their ASTs into `tree`, effectively modifying `tree`.
    pub fn link(&self, mut tree: ast::OpenQasmProgram) -> Result<ast::OpenQasmProgram> {
        let mut to_embed = vec![];
        for (index, span) in tree.program.iter().enumerate() {
            if let ast::Statement::Include(libpath) = &*span.node {
                let source = self
                    .sources(libpath)
                    .map_err(|_| LinkerError::LibraryNotFound {
                        location: span.boundaries.0,
                        libpath: libpath.into(),
                    })?;
                let library_tree = parse_library(&source).unwrap();
                to_embed.push((index, span.boundaries, library_tree.definitions));
            }
        }
        to_embed.reverse();
        for (index, boundaries, statements) in to_embed {
            let mut inner_spans = vec![];
            for one_statement in statements {
                inner_spans.push(ast::Span {
                    boundaries,
                    node: Box::new(one_statement),
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

    use crate::grammar::{ast::Span, parse_program};
    use crate::linker::Location;

    use super::*;

    macro_rules! span {
        ($left:expr, $node:expr, $right:expr) => {
            Span {
                boundaries: (Location($left), Location($right)),
                node: Box::new($node),
            }
        };
    }

    #[test]
    fn test_linker_loads_embedded_libraries() {
        let source = indoc!(
            "
    OPENQASM 2.0;
    include \"test.inc\";
    "
        );
        let linker = Linker::with_embedded(HashMap::from_iter(vec![(
            "test.inc".to_owned(),
            "gate test () q {}".to_owned(),
        )]));
        let tree = parse_program(source).unwrap();
        let linked_tree = linker.link(tree).unwrap();
        assert_eq!(
            linked_tree,
            ast::OpenQasmProgram {
                version: "2.0".to_owned(),
                program: vec![span!(
                    14,
                    ast::Statement::GateDecl {
                        signature: ("test".to_owned(), vec![], vec!["q".to_string()], vec![]),
                        docstring: None
                    },
                    33
                )]
            }
        )
    }
}
