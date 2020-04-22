use std::collections::HashMap;

use crate::grammar::ast;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentSolver<'bindings>(&'bindings HashMap<String, ast::Argument>);

impl<'bindings> ArgumentSolver<'bindings> {
  pub fn new(argument_table: &'bindings HashMap<String, ast::Argument>) -> Self {
    ArgumentSolver::<'bindings>(argument_table)
  }

  pub fn solve(&self, arg: &ast::Argument) -> Result<ast::Argument, String> {
    match arg {
      ast::Argument::Id(name) => {
        match self.0.get(name) {
          None => Err(name.clone()),
          Some(argument) => Ok(argument.clone())
        }
      }
      _ => unreachable!("while solving, only valid argument style is Argument::Id")
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use std::iter::FromIterator;

  #[test]
  fn test_replace_formal_parameter_with_actual_parameter() {
    let actual_argument = ast::Argument::Item("actual".to_owned(), 0);
    let bindings = HashMap::from_iter(vec![
      ("formal".to_owned(), actual_argument.clone())
    ]);
    let solver = ArgumentSolver::new(&bindings);
    let formal_argument = ast::Argument::Id("formal".to_owned());
    let argument = solver.solve(&formal_argument).expect("get actual argument");
    assert_eq!(argument, actual_argument);
  }

  #[test]
  fn test_actual_parameter_not_found() {
    let actual_argument = ast::Argument::Item("actual".to_owned(), 0);
    let bindings = HashMap::from_iter(vec![
      ("formal".to_owned(), actual_argument.clone())
    ]);
    let solver = ArgumentSolver::new(&bindings);
    let formal_argument = ast::Argument::Id("fmal".to_owned());
    let error = solver.solve(&formal_argument).expect_err("actual argument not found");
    assert_eq!(error, String::from("fmal"));
  }
}