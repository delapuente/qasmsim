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
  use grammar::ast::*;
  use std::f64::consts::PI;

}