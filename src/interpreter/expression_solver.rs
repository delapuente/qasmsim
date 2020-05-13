use std::collections::HashMap;

use crate::grammar::ast;

#[derive(Debug, Clone, PartialEq)]
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
            ast::Expression::Minus(expr) => -self.solve(expr)?,
            ast::Expression::Op(op_code, left, right) => match op_code {
                ast::OpCode::Add => self.solve(left)? + self.solve(right)?,
                ast::OpCode::Sub => self.solve(left)? - self.solve(right)?,
                ast::OpCode::Mul => self.solve(left)? * self.solve(right)?,
                ast::OpCode::Div => self.solve(left)? / self.solve(right)?,
                ast::OpCode::Pow => self.solve(left)?.powf(self.solve(right)?),
            },
            ast::Expression::Function(func_code, expr) => match func_code {
                ast::FuncCode::Sin => self.solve(expr)?.sin(),
                ast::FuncCode::Cos => self.solve(expr)?.cos(),
                ast::FuncCode::Tan => self.solve(expr)?.tan(),
                ast::FuncCode::Exp => self.solve(expr)?.exp(),
                ast::FuncCode::Ln => self.solve(expr)?.ln(),
                ast::FuncCode::Sqrt => self.solve(expr)?.sqrt(),
            },
            ast::Expression::Id(name) => match self.0.get(name) {
                None => return Err(name.into()),
                Some(value) => *value,
            },
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
    #[allow(clippy::float_cmp)]
    fn test_expression_solver() {
        let expression = Expression::Op(
            OpCode::Add,
            Box::new(Expression::Minus(Box::new(Expression::Pi))),
            Box::new(Expression::Op(
                OpCode::Div,
                Box::new(Expression::Op(
                    OpCode::Mul,
                    Box::new(Expression::Op(
                        OpCode::Sub,
                        Box::new(Expression::Real(1.0)),
                        Box::new(Expression::Op(
                            OpCode::Pow,
                            Box::new(Expression::Real(2.0)),
                            Box::new(Expression::Real(3.0)),
                        )),
                    )),
                    Box::new(Expression::Real(4.0)),
                )),
                Box::new(Expression::Real(5.0)),
            )),
        );
        let empty = HashMap::new();
        let solver = ExpressionSolver::new(&empty);
        let result = solver.solve(&expression).expect("get value of expression");
        assert_eq!(result, -PI + (1.0 - 2.0_f64.powf(3.0)) * 4.0 / 5.0);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_expression_solver_with_functions() {
        let expression = Expression::Function(
            FuncCode::Sqrt,
            Box::new(Expression::Function(
                FuncCode::Ln,
                Box::new(Expression::Function(
                    FuncCode::Exp,
                    Box::new(Expression::Function(
                        FuncCode::Tan,
                        Box::new(Expression::Function(
                            FuncCode::Cos,
                            Box::new(Expression::Function(
                                FuncCode::Sin,
                                Box::new(Expression::Real(1.0)),
                            )),
                        )),
                    )),
                )),
            )),
        );
        let empty = HashMap::new();
        let solver = ExpressionSolver::new(&empty);
        let result = solver.solve(&expression).expect("get value of expression");
        assert_eq!(result, 1.0_f64.sin().cos().tan().exp().ln().sqrt());
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_expression_solver_with_symbol_substitution() {
        let expression = Expression::Op(
            OpCode::Add,
            Box::new(Expression::Id("some_name".into())),
            Box::new(Expression::Op(
                OpCode::Div,
                Box::new(Expression::Op(
                    OpCode::Mul,
                    Box::new(Expression::Op(
                        OpCode::Sub,
                        Box::new(Expression::Real(1.0)),
                        Box::new(Expression::Real(2.0)),
                    )),
                    Box::new(Expression::Real(3.0)),
                )),
                Box::new(Expression::Real(4.0)),
            )),
        );
        let bindings = HashMap::from_iter(vec![("some_name".into(), 1.0)]);
        let solver = ExpressionSolver::new(&bindings);
        let result = solver.solve(&expression).expect("get value of expression");
        assert_eq!(result, 1.0 + (1.0 - 2.0) * 3.0 / 4.0);
    }

    #[test]
    fn test_expression_solver_fails_at_symbol_substitution() {
        let expression = Expression::Op(
            OpCode::Add,
            Box::new(Expression::Id("some_name".into())),
            Box::new(Expression::Real(1.0)),
        );
        let empty_bindings = HashMap::new();
        let solver = ExpressionSolver::new(&empty_bindings);
        let error = solver
            .solve(&expression)
            .expect_err("fails at replacing `some_name`");
        assert_eq!(error, String::from("some_name"));
    }
}
