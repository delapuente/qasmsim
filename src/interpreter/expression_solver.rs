use std::collections::HashMap;

use crate::grammar::ast;

pub struct ExpressionSolver<'bindings>(&'bindings HashMap<String, f64>);

impl<'bindings> ExpressionSolver<'bindings> {
  pub fn new(symbol_table: &'bindings HashMap<String, f64>) -> Self {
    ExpressionSolver::<'bindings>(symbol_table)
  }

  pub fn solve(&self, expression: &ast::Expression) -> Result<f64, String> {
    Ok(match expression {
      ast::Expression::Pi => std::f64::consts::PI,
      ast::Expression::Int(value) => *value as f64,
      ast::Expression::Real(value) => *value,
      ast::Expression::Minus(expr) => - self.solve(expr)?,
      ast::Expression::Op(opcode, left, right) => match opcode {
        ast::Opcode::Add => self.solve(left)? + self.solve(right)?,
        ast::Opcode::Sub => self.solve(left)? - self.solve(right)?,
        ast::Opcode::Mul => self.solve(left)? * self.solve(right)?,
        ast::Opcode::Div => self.solve(left)? / self.solve(right)?,
        ast::Opcode::Pow => self.solve(left)?.powf(self.solve(right)?)
      },
      ast::Expression::Function(funccode, expr) => match funccode {
        ast::Funccode::Sin => self.solve(expr)?.sin(),
        ast::Funccode::Cos => self.solve(expr)?.cos(),
        ast::Funccode::Tan => self.solve(expr)?.tan(),
        ast::Funccode::Exp => self.solve(expr)?.exp(),
        ast::Funccode::Ln => self.solve(expr)?.ln(),
        ast::Funccode::Sqrt => self.solve(expr)?.sqrt()
      },
      ast::Expression::Id(name) => {
        match self.0.get(name) {
          None => return Err(name.into()),
          Some(value) => *value
        }
      }
    })
  }
}

#[cfg(test)]
mod test {
  use std::f64::consts::PI;
  use std::iter::FromIterator;

  use super::*;
  use crate::grammar::ast::*;

  #[test]
  fn test_expression_solver() {
    let expression = Expression::Op(
      Opcode::Add,
      Box::new(Expression::Minus(Box::new(Expression::Pi))),
      Box::new(Expression::Op(
        Opcode::Div,
        Box::new(Expression::Op(
          Opcode::Mul,
          Box::new(Expression::Op(
            Opcode::Sub,
            Box::new(Expression::Real(1.0)),
            Box::new(Expression::Op(
              Opcode::Pow,
              Box::new(Expression::Real(2.0)),
              Box::new(Expression::Real(3.0))
            ))
          )),
          Box::new(Expression::Real(4.0))
        )),
        Box::new(Expression::Real(5.0))
      ))
    );
    let empty = HashMap::new();
    let solver = ExpressionSolver::new(&empty);
    let result = solver.solve(&expression).expect("get value of expression");
    assert_eq!(result, - PI + (1.0 - 2.0_f64.powf(3.0)) * 4.0 / 5.0);
  }

  #[test]
  fn test_expression_solver_with_functions() {
    let expression = Expression::Function(
      Funccode::Sqrt,
      Box::new(Expression::Function(
        Funccode::Ln,
        Box::new(Expression::Function(
          Funccode::Exp,
          Box::new(Expression::Function(
            Funccode::Tan,
            Box::new(Expression::Function(
              Funccode::Cos,
              Box::new(Expression::Function(
                Funccode::Sin,
                Box::new(Expression::Real(1.0))
              ))
            ))
          ))
        ))
      ))
    );
    let empty = HashMap::new();
    let solver = ExpressionSolver::new(&empty);
    let result = solver.solve(&expression).expect("get value of expression");
    assert_eq!(result, 1.0_f64.sin().cos().tan().exp().ln().sqrt());
  }

  #[test]
  fn test_expression_solver_with_symbol_substitution() {
    let expression = Expression::Op(
      Opcode::Add,
      Box::new(Expression::Id("some_name".into())),
      Box::new(Expression::Op(
        Opcode::Div,
        Box::new(Expression::Op(
          Opcode::Mul,
          Box::new(Expression::Op(
            Opcode::Sub,
            Box::new(Expression::Real(1.0)),
            Box::new(Expression::Real(2.0))
          )),
          Box::new(Expression::Real(3.0))
        )),
        Box::new(Expression::Real(4.0))
      ))
    );
    let bindings = HashMap::from_iter(vec![
      ("some_name".into(), 1.0)
    ]);
    let solver = ExpressionSolver::new(&bindings);
    let result = solver.solve(&expression).expect("get value of expression");
    assert_eq!(result, 1.0 + (1.0 - 2.0) * 3.0 / 4.0);
  }

  #[test]
  fn test_expression_solver_fails_at_symbol_substitution() {
    let expression = Expression::Op(
      Opcode::Add,
      Box::new(Expression::Id("some_name".into())),
      Box::new(Expression::Real(1.0))
    );
    let empty_bindings = HashMap::new();
    let solver = ExpressionSolver::new(&empty_bindings);
    let error = solver.solve(&expression).expect_err("fails at replacing `some_name`");
    assert_eq!(error, String::from("some_name"));
  }

}