use std::collections::HashMap;

use grammar::ast;

pub struct ArgumentSolver<'a>(&'a HashMap<String, ast::Argument>);

impl<'a> ArgumentSolver<'a> {
  pub fn new(argument_table: &'a HashMap<String, ast::Argument>) -> Self {
    ArgumentSolver::<'a>(argument_table)
  }

  pub fn solve(&self, arg: &ast::Argument) -> ast::Argument {
    match arg {
      ast::Argument::Id(name) => (*self.0.get(name).unwrap()).clone(),
      _ => unreachable!()
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
    assert_eq!(solver.solve(&formal_argument), actual_argument);
  }
}