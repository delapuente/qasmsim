use std::collections::{ HashMap, VecDeque };
use std::iter::FromIterator;

use semantics::{ Semantics, RegisterType, extract_semantics };
use statevector::StateVector;
use grammar::ast;
use interpreter::expression_solver::ExpressionSolver;
use interpreter::argument_solver::ArgumentSolver;
use interpreter::computation::Computation;

type BindingMappings = (HashMap<String, f64>, HashMap<String, ast::Argument>);

struct Runtime {
  macro_stack: VecDeque<BindingMappings>,
  semantics: Semantics,
  statevector: StateVector,
  memory: HashMap<String, u64>
}

impl Runtime {
  pub fn new(semantics: Semantics) -> Self {
    let memory_size = semantics.quantum_memory_size;
    let mut initial_memory = HashMap::new();
    for register in semantics.register_table.values() {
      if register.1 == RegisterType::C {
        initial_memory.insert(register.0.clone(), 0_u64);
      }
    }
    Runtime {
      macro_stack: VecDeque::new(),
      semantics,
      statevector: StateVector::new(memory_size),
      memory: initial_memory
    }
  }

  fn apply_gates(&mut self, statements: &Vec<ast::Statement>) {
    for statement in statements {
      match statement {
        ast::Statement::QuantumOperation(operation) => {
          self.apply_quantum_operation(operation);
        },
        ast::Statement::Conditional(register, test, operation) => {
          // In program execution, do not replace symbols
          let actual_register: ast::Argument = if self.macro_stack.len() == 0 {
            (*register).clone()
          }
          // In macro execution, replace formal arguments with actual arguments
          else {
            let arg_bindings = &self.macro_stack.get(0).unwrap().1;
            let argument_solver = ArgumentSolver::new(arg_bindings);
            argument_solver.solve(&register)
          };
          let value = match actual_register {
            ast::Argument::Id(register_name) => self.memory.get(&register_name).unwrap(),
            _ => unreachable!()
          };
          if value == test {
            self.apply_quantum_operation(operation);
          }
        }
        _ => ()
      };
    }
  }

  fn apply_quantum_operation(&mut self, operation: &ast::QuantumOperation) {
    match operation {
      ast::QuantumOperation::Unitary(unitary) => {
        self.apply_unitary(unitary)
      },
      ast::QuantumOperation::Measure(source, target) => {
        self.apply_measurement(vec![(*source).clone(), (*target).clone()])
      },
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

  fn apply_measurement(&mut self, args: Vec<ast::Argument>) {
    for argument_expansion in self.expand_arguments(&args) {
      self.apply_one_measurement(argument_expansion)
    }
  }

  fn apply_one_measurement(&mut self, args: Vec<ast::Argument>) {
    let classical_register_name = match &args[1] {
      ast::Argument::Item(name, _) => name.clone(),
      _ => unreachable!()
    };
    let source = self.get_bit_mapping(&args[0]);
    let measurement = self.statevector.measure(source) as u64;

    if !self.memory.contains_key(&classical_register_name) {
      unreachable!();
    }

    let target = self.get_bit_mapping(&args[1]);
    let value = measurement * (1 << target);
    let prev_value = *(self.memory.get(&classical_register_name).unwrap());
    self.memory.insert(classical_register_name, prev_value + value);
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
-> Result<Computation, String> {
  let semantics = extract_semantics(program)?;
  let mut runtime = Runtime::new(semantics);
  runtime.apply_gates(&program.program);
  Ok(Computation::new(runtime.memory, runtime.statevector))
}