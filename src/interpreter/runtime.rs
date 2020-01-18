use std::collections::{ HashMap, VecDeque };
use std::iter::FromIterator;

use semantics::{ Semantics, extract_semantics };
use statevector::StateVector;
use grammar::ast;
use interpreter::expression_solver::ExpressionSolver;
use interpreter::argument_solver::ArgumentSolver;

type BindingMappings = (HashMap<String, f64>, HashMap<String, ast::Argument>);

struct Runtime {
  macro_stack: VecDeque<BindingMappings>,
  semantics: Semantics,
  statevector: StateVector
}

impl Runtime {
  pub fn new(semantics: Semantics) -> Self {
    let memory_size = semantics.quantum_memory_size;
    Runtime {
      macro_stack: VecDeque::new(),
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
    let name = &unitary.0;
    let real_args = &unitary.1;
    let args = &unitary.2;

    // In program execution, do not replace symbols
    let actual_args: Vec<ast::Argument> = if self.macro_stack.len() == 0 {
      args.iter().map(|arg| arg.clone()).collect()
    }
    // In macro execution, replace formal arguments with actual arguments
    else {
      let arg_bindings = &self.macro_stack.get(0).unwrap().1;
      let argument_solver = ArgumentSolver::new(arg_bindings);
      args.iter().map(|arg| argument_solver.solve(&arg)).collect()
    };

    let empty = HashMap::new();
    let real_bindings = if self.macro_stack.len() == 0 {
      &empty
    }
    else {
      &self.macro_stack.get(0).unwrap().0
    };
    let solver = ExpressionSolver::new(real_bindings);
    let solved_real_args: Vec<f64> = real_args.iter().map(|arg| solver.solve(&arg)).collect();

    for argument_expansion in self.expand_arguments(&actual_args) {
      self.apply_one_gate(name, &solved_real_args, &argument_expansion)
    }
  }

  fn apply_one_gate(&mut self, name: &str, real_args: &Vec<f64>,
  args: &Vec<ast::Argument>) {
    match name {
      "U" => {
        let theta = real_args[0];
        let phi = real_args[1];
        let lambda = real_args[2];
        let target = self.get_bit_mapping(&args[0]);
        self.statevector.u(theta, phi, lambda, target);
      }
      "CX" => {
        let control = self.get_bit_mapping(&args[0]);
        let target = self.get_bit_mapping(&args[1]);
        self.statevector.cnot(control, target);
      }
      macro_name => {
        let binding_mappings = self.bind(macro_name.to_owned(), real_args, args);
        self.call(macro_name.to_owned(), binding_mappings).unwrap();
      }
    };
  }

  fn apply_gate_operations(&mut self, operations: &Vec<ast::GateOperation>)
  -> Result<(), String> {
    for one_operation in operations {
      match one_operation {
        ast::GateOperation::Unitary(unitary) => self.apply_unitary(unitary),
        _ => ()
      };
    }
    Ok(())
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

  fn bind(&mut self, macro_name: String, real_args: &Vec<f64>, args: &Vec<ast::Argument>)
  -> BindingMappings {
    let definition = self.semantics.macro_definitions.get(&macro_name).unwrap();
    let real_args_mapping = HashMap::from_iter(
      definition.1.iter()
      .zip(real_args.iter()) // pair formal arguments with their float values
      .map(|(s, f)| (s.to_owned(), *f)) // convert them into proper copies
    );
    let args_mapping = HashMap::from_iter(
      definition.2.iter()
      .zip(args.iter().map(|a| a.clone())) // pair formal arguments with their registers
      .map(|(s, r)| (s.to_owned(), r.clone())) // convert them into proper copies
    );
    (real_args_mapping, args_mapping)
  }

  fn call(&mut self, macro_name: String, bindings: BindingMappings)
  -> Result<(), String> {
    // XXX: Why clonning is necessary??
    let definition = (*self.semantics.macro_definitions.get(&macro_name).unwrap()).clone();
    self.macro_stack.push_front(bindings);
    self.apply_gate_operations(&definition.3)?;
    self.macro_stack.pop_front();
    Ok(())
  }
}

pub fn execute(program: &ast::OpenQasmProgram)
-> Result<StateVector, String> {
  let semantics = extract_semantics(program)?;
  let mut runtime = Runtime::new(semantics);
  runtime.apply_gates(&program.program);
  Ok(runtime.statevector)
}