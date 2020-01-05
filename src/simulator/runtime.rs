use std::collections::HashMap;

use semantics::{ Semantics, extract_semantics};
use statevector::StateVector;
use grammar::ast;
use gatelib;
use simulator::expression_solver::ExpressionSolver;

struct Runtime {
  semantics: Semantics,
  statevector: StateVector
}

impl Runtime {
  pub fn new(semantics: Semantics) -> Self {
    let memory_size = semantics.quantum_memory_size;
    Runtime {
      semantics,
      statevector: StateVector::new(memory_size)
    }
  }

  fn apply_gates(&mut self, statements: &Vec<ast::Statement>) {
    for statement in statements {
      match statement {
        ast::Statement::QuantumOperation(operation) => {
          self.apply_quantum_operation(operation);
        }
        _ => ()
      };
    }
  }

  fn apply_quantum_operation(&mut self, operation: &ast::QuantumOperation) {
    match operation {
      ast::QuantumOperation::Unitary(unitary) => {
        self.apply_unitary(unitary)
      }
      _ => ()
    };
  }

  fn apply_unitary(&mut self, unitary: &ast::UnitaryOperation) {
    match unitary {
      ast::UnitaryOperation::GateExpansion(name, real_args, args) => {
        for argument_expansion in self.expand_arguments(args) {
          self.apply_one_gate(name, real_args, &argument_expansion)
        }
      }
      _ => ()
    };
  }

  fn apply_one_gate(&mut self, name: &str, real_args: &Vec<ast::Expression>,
  args: &Vec<ast::Argument>) {
    match name {
      "h" => {
        let index = self.get_bit_mapping(&args[0]);
        gatelib::h(index, &mut self.statevector);
      }
      "cx" => {
        let control = self.get_bit_mapping(&args[0]);
        let target = self.get_bit_mapping(&args[1]);
        gatelib::cx(control, target, &mut self.statevector);
      }
      macro_name => {
        let solver = ExpressionSolver::new(HashMap::new());
        let solved_real_args: Vec<f64> = real_args.iter().map(|arg| solver.solve(&arg)).collect();
        // let binding = bind(semantics, macro_name, solved_real_args, args);
        // runtime = apply_gates(semantics, binding.program, runtime);
      }
    };
  }

  fn expand_arguments(&self, args: &Vec<ast::Argument>)
  -> Vec<Vec<ast::Argument>> {
    let range = self.get_range(args);
    range.map(|index| Runtime::specify(args, index)).collect()
  }

  fn get_bit_mapping(&self, argument: &ast::Argument) -> usize {
    match argument {
      ast::Argument::Item(name, index) => {
        let mapping = self.semantics.memory_map.get(name).unwrap();
        mapping.1 + *index
      }
      _ => unreachable!()
    }
  }

  fn get_range(&self, args: &Vec<ast::Argument>) -> std::ops::Range<usize> {
    for arg in args {
      match arg {
        ast::Argument::Id(name) => {
          return 0..self.semantics.register_table.get(name).unwrap().2;
        }
        _ => ()
      }
    }
    0..1
  }

  fn specify(args: &Vec<ast::Argument>, index: usize) -> Vec<ast::Argument> {
    let mut result = vec!();
    for arg in args {
      match arg {
        ast::Argument::Id(name) => result.push(ast::Argument::Item(name.clone(), index)),
        other => result.push(other.clone())
      }
    }
    result
  }
}

pub fn execute(program: &ast::OpenQasmProgram)
-> Result<StateVector, String> {
  let semantics = extract_semantics(program)?;
  let mut runtime = Runtime::new(semantics);
  runtime.apply_gates(&program.program);
  Ok(runtime.statevector)
}