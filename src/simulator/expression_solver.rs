use std::collections::HashMap;

use grammar::ast;

pub struct ExpressionSolver(HashMap<String, f64>);

impl ExpressionSolver {
  pub fn new(symbol_table: HashMap<String, f64>) -> Self {
    ExpressionSolver(symbol_table)
  }

  pub fn solve(&self, expression: &ast::Expression) -> f64 {
    match expression {
      ast::Expression::Pi => std::f64::consts::PI,
      ast::Expression::Real(value) => *value,
      ast::Expression::Minus(expr) => - self.solve(expr),
      ast::Expression::Op(opcode, left, right) => match opcode {
        ast::Opcode::Add => self.solve(left) + self.solve(right),
        ast::Opcode::Sub => self.solve(left) - self.solve(right),
        ast::Opcode::Mul => self.solve(left) * self.solve(right),
        ast::Opcode::Div => self.solve(left) / self.solve(right)
      },
      ast::Expression::Id(name) => *self.0.get(name).unwrap()
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use grammar::ast::*;
  use std::f64::consts::PI;

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
            Box::new(Expression::Real(2.0))
          )),
          Box::new(Expression::Real(3.0))
        )),
        Box::new(Expression::Real(4.0))
      ))
    );
    let solver = ExpressionSolver::new(HashMap::new());
    assert_eq!(solver.solve(&expression), - PI + (1.0 - 2.0) * 3.0 / 4.0);
  }

}