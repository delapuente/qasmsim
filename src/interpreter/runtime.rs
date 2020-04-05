use std::collections::{ HashMap, VecDeque };
use std::iter::FromIterator;

use crate::api;
use crate::error::{ QasmSimError, RuntimeKind };
use crate::semantics::{ Semantics, RegisterType, extract_semantics, SemanticError };
use crate::statevector::StateVector;
use crate::grammar::{ Location, ast };
use crate::interpreter::expression_solver::ExpressionSolver;
use crate::interpreter::argument_solver::ArgumentSolver;
use crate::interpreter::computation::{ Computation, HistogramBuilder };

type BindingMappings = (HashMap<String, f64>, HashMap<String, ast::Argument>);

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
  /* ClassicalRegisterNotFound,
  QuantumRegisterNotFound,
  SymbolNotFound,
  UndefinedGate,
  WrongNumberOfRealParameters,
  WrongNumberOfQuantumParameters,
  DifferentSizeRegisters, */
  Other,
  SemanticError(SemanticError),
  IndexOutOfBounds {
    location: Location,
    symbol_name: String,
    index: usize,
    size: usize
  }
}

type Result<T> = std::result::Result<T, RuntimeError>;

impl From<SemanticError> for RuntimeError {
  fn from(semantic_error: SemanticError) -> Self {
    RuntimeError::SemanticError(semantic_error)
  }
}

struct Runtime<'program> {
  macro_stack: VecDeque<BindingMappings>,
  semantics: Semantics,
  statevector: StateVector,
  memory: HashMap<String, u64>,
  location: Option<&'program Location>
}

impl<'src, 'program> Runtime<'program> {
  pub fn new(semantics: Semantics) -> Self {
    let memory_size = semantics.quantum_memory_size;

    let mut runtime = Runtime {
      macro_stack: VecDeque::new(),
      semantics,
      statevector: StateVector::new(memory_size),
      memory: HashMap::new(),
      location: None
    };

    runtime.reset();
    runtime
  }

  pub fn reset(&mut self) {
    self.macro_stack.clear();
    self.statevector.reset();
    self.clear_memory();
  }

  fn clear_memory(&mut self) {
    self.memory.clear();
    for register in self.semantics.register_table.values() {
      if register.1 == RegisterType::C {
        self.memory.insert(register.0.clone(), 0_u64);
      }
    }
  }

  fn apply_gates(&mut self, statements: &'program Vec<ast::Span<ast::Statement>>) -> Result<()> {
    for span in statements {
      self.location = Some(&span.boundaries.0);
      match &*span.node {
        ast::Statement::QuantumOperation(operation) => {
          self.apply_quantum_operation(&operation)?;
        }
        ast::Statement::Conditional(register, test, operation) => {
          let actual_register = (register).clone();
          let register_name = self.get_register_name(&actual_register);
          self.assert_is_classical_register(register_name)?;

          let value = match actual_register {
            ast::Argument::Id(register_name) => self.memory.get(&register_name).expect("after `assert_is_classical_register()`, must exist"),
            _ => unreachable!("cannot index a register inside the condition")
          };
          if value == test {
            self.apply_quantum_operation(&operation)?;
          }
        }
        _ => ()
      };
    }
    Ok(())
  }

  fn apply_quantum_operation(&mut self, operation: &ast::QuantumOperation) -> Result<()> {
    match operation {
      ast::QuantumOperation::Unitary(unitary) => {
        self.apply_unitary(unitary)
      }
      ast::QuantumOperation::Measure(source, target) => {
        self.apply_measurement(vec![(*source).clone(), (*target).clone()])
      }
      _ => Ok(())
    }
  }

  fn apply_unitary(&mut self, unitary: &ast::UnitaryOperation) -> Result<()> {
    let name = &unitary.0;
    let real_args = &unitary.1;
    let args = &unitary.2;

    let actual_args = self.resolve_actual_args(args)?;
    self.check_all_are_quantum_registers(&actual_args)?;

    let solved_real_args = self.resolve_real_expressions(real_args)?;

    for argument_expansion in self.expand_arguments(&actual_args)? {
      self.apply_one_gate(name, &solved_real_args, &argument_expansion)?;
    }

    Ok(())
  }

  fn resolve_actual_args(&self, args: &Vec<ast::Argument>) -> Result<Vec<ast::Argument>> {
    let mut actual = Vec::new();
    if !self.is_running_macro() {
      actual = args.iter().cloned().collect();
    }
    else {
      let stack_entry = self.macro_stack.get(0).expect("if `is_running_macro()`, get first entry of the stack");
      let arg_bindings = &stack_entry.1;
      let argument_solver = ArgumentSolver::new(arg_bindings);
      for argument in args {
        let actual_argument = argument_solver.solve(argument).map_err(|_| RuntimeError::Other)?;
        actual.push(actual_argument)
      }
    }
    Ok(actual)
  }

  fn resolve_real_expressions(&self, exprs: &Vec<ast::Expression>) -> Result<Vec<f64>> {
    let mut real_bindings = &HashMap::new();
    if self.is_running_macro() {
      let stack_entry = self.macro_stack.get(0).expect("if `is_running_macro()`, get first stack entry");
      real_bindings = &stack_entry.0;
    };
    let expression_solver = ExpressionSolver::new(real_bindings);
    let mut solved = Vec::new();
    for expression in exprs {
      let value = expression_solver.solve(&expression).map_err(|_| RuntimeError::Other)?;
      solved.push(value);
    }
    Ok(solved)
  }

  fn is_running_macro(&self) -> bool {
    self.macro_stack.len() > 0
  }

  fn apply_measurement(&mut self, args: Vec<ast::Argument>) -> Result<()> {
    self.assert_is_quantum_register(self.get_register_name(&args[0]))?;
    self.assert_is_classical_register(self.get_register_name(&args[1]))?;
    for argument_expansion in self.expand_arguments(&args)? {
      self.apply_one_measurement(argument_expansion)?;
    }
    Ok(())
  }

  fn apply_one_measurement(&mut self, args: Vec<ast::Argument>) -> Result<()> {
    let classical_register_name = self.get_register_name(&args[1]);
    let source = self.get_bit_mapping(&args[0])?;
    let measurement = self.statevector.measure(source) as u64;

    let target = self.get_bit_mapping(&args[1])?;
    let value = measurement * (1 << target);
    let prev_value = *(self.memory.get(classical_register_name).expect("after `apply_measurement()`, get the entry"));
    self.memory.insert(classical_register_name.into(), prev_value + value);

    Ok(())
  }

  fn apply_one_gate(&mut self, name: &str, real_args: &Vec<f64>,
  args: &Vec<ast::Argument>) -> Result<()> {
    match name {
      "U" => {
        let theta = real_args[0];
        let phi = real_args[1];
        let lambda = real_args[2];
        let target = self.get_bit_mapping(&args[0])?;
        self.statevector.u(theta, phi, lambda, target);
      }
      "CX" => {
        let control = self.get_bit_mapping(&args[0])?;
        let target = self.get_bit_mapping(&args[1])?;
        self.statevector.cnot(control, target);
      }
      macro_name => {
        let binding_mappings = self.bind(macro_name.to_owned(), real_args, args)?;
        self.call(macro_name.to_owned(), binding_mappings)?;
      }
    };
    Ok(())
  }

  fn check_all_are_quantum_registers(&self, args: &Vec<ast::Argument>) -> Result<()> {
    for argument in args {
      let register_name = self.get_register_name(argument);
      self.assert_is_quantum_register(register_name)?;
    }
    Ok(())
  }

  fn get_register_name<'a>(&self, arg: &'a ast::Argument) -> &'a str {
    match arg {
      ast::Argument::Id(name) => name,
      ast::Argument::Item(name, _) => name
    }
  }

  fn assert_is_quantum_register(&self, name: &str) -> Result<()> {
    if !self.is_register_of_type(RegisterType::Q, name) {
      return Err(RuntimeError::Other/* QasmSimError::RuntimeError {
        kind: RuntimeKind::QuantumRegisterNotFound,
        symbol_name: name.into()
      } */);
    }
    Ok(())
  }

  fn assert_is_classical_register(&self, name: &str) -> Result<()> {
    if !self.is_register_of_type(RegisterType::C, name) {
      return Err(RuntimeError::Other/* QasmSimError::RuntimeError {
        kind: RuntimeKind::ClassicalRegisterNotFound,
        symbol_name: name.into()
      } */);
    }
    Ok(())
  }

  fn is_register_of_type(&self, rtype: RegisterType, name: &str) -> bool {
    match self.semantics.register_table.get(name) {
      Some(entry) => entry.1 == rtype,
      None => false
    }
  }

  fn apply_gate_operations(&mut self, operations: &Vec<ast::GateOperation>)
  -> Result<()> {
    for one_operation in operations {
      match one_operation {
        ast::GateOperation::Unitary(unitary) => self.apply_unitary(unitary)?,
        _ => ()
      };
    }
    Ok(())
  }

  fn expand_arguments(&self, args: &Vec<ast::Argument>)
  -> Result<Vec<Vec<ast::Argument>>> {
    let range = self.get_range(args)?;
    Ok(range.map(|index| Runtime::specify(args, index)).collect())
  }

  // TODO: Add index boundaries control
  fn get_bit_mapping(&self, argument: &ast::Argument) -> Result<usize> {
    match argument {
      ast::Argument::Item(name, index) => {
        match self.semantics.memory_map.get(name) {
          None => Err(RuntimeError::Other/* QasmSimError::RuntimeError {
            kind: RuntimeKind::QuantumRegisterNotFound,
            symbol_name: name.into()
          } */),
          Some(mapping) => {
            let size = mapping.2 - mapping.1 + 1;
            if *index >= size {
              return Err(RuntimeError::IndexOutOfBounds {
                location: self.location.expect("after `apply_gates()`, location is something").clone(),
                symbol_name: name.into(),
                index: *index,
                size: size
              });
            }
            Ok(mapping.1 + *index)
          }
        }
      }
      _ => unreachable!("after `expand_arguments()`, argument should be Argument::Item")
    }
  }

  fn get_range(&self, args: &Vec<ast::Argument>) -> Result<std::ops::Range<usize>> {
    // XXX: This is performed after validating the type of args.

    let whole_registers: Vec<&ast::Argument> = args.iter()
    .filter(|arg| match arg { ast::Argument::Id(_) => true, _ => false })
    .collect();

    // Return a one-iteration range, `specify()` takes care of ignoring Item arugments.
    if whole_registers.len() == 0 {
      return Ok(0..1);
    }

    let all_sizes: Vec<usize> = whole_registers.iter()
    .map(|arg| {
      let register_name = self.get_register_name(arg);
      let register_entry = self.semantics.register_table.get(register_name).expect("after validation, get register entry");
      register_entry.2
    })
    .collect();

    let reference_size = all_sizes[0];
    let all_the_same_size = all_sizes.iter().all(|size| *size == reference_size);

    if all_the_same_size {
      return Ok(0..reference_size);
    }

    return Err(RuntimeError::Other/* QasmSimError::RuntimeError {
      kind: RuntimeKind::DifferentSizeRegisters,
      symbol_name: self.get_register_name(whole_registers[0]).into()
    } */);
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
  -> Result<BindingMappings> {
    let definition = match self.semantics.macro_definitions.get(&macro_name) {
      None => {
        return Err(RuntimeError::Other/* QasmSimError::RuntimeError {
          kind: RuntimeKind::UndefinedGate,
          symbol_name: macro_name
        } */);
      }
      Some(definition) => definition
    };

    if real_args.len() != definition.1.len() {
      return Err(RuntimeError::Other/* QasmSimError::RuntimeError {
        kind: RuntimeKind::WrongNumberOfRealParameters,
        symbol_name: macro_name
      } */);
    }
    let real_args_mapping = HashMap::from_iter(
      definition.1.iter()
      .zip(real_args.iter()) // pair formal arguments with their float values
      .map(|(s, f)| (s.to_owned(), *f)) // convert them into proper copies
    );

    if args.len() != definition.2.len() {
      return Err(RuntimeError::Other/* QasmSimError::RuntimeError {
        kind: RuntimeKind::WrongNumberOfQuantumParameters,
        symbol_name: macro_name
      } */);
    }
    let args_mapping = HashMap::from_iter(
      definition.2.iter()
      .zip(args.iter().map(|a| a.clone())) // pair formal arguments with their registers
      .map(|(s, r)| (s.to_owned(), r.clone())) // convert them into proper copies
    );

    Ok((real_args_mapping, args_mapping))
  }

  fn call(&mut self, macro_name: String, bindings: BindingMappings)
  -> Result<()> {
    // XXX: Why clonning is necessary??
    let definition = (*self.semantics.macro_definitions.get(&macro_name).unwrap()).clone();
    self.macro_stack.push_front(bindings);
    self.apply_gate_operations(&definition.3)?;
    self.macro_stack.pop_front();
    Ok(())
  }
}

pub fn execute(program: &ast::OpenQasmProgram) -> Result<Computation> {
  let semantics = extract_semantics(program)?;
  let mut runtime = Runtime::new(semantics);
  runtime.apply_gates(&program.program)?;
  Ok(Computation::new(runtime.memory, runtime.statevector, None))
}

pub fn execute_with_shots(program: &ast::OpenQasmProgram, shots: usize) -> Result<Computation> {
  let semantics = extract_semantics(program)?;
  let mut runtime = Runtime::new(semantics);
  let mut histogram_builder = HistogramBuilder::new();
  for _ in 0..shots {
    runtime.reset();
    runtime.apply_gates(&program.program)?;
    histogram_builder.update(&runtime.memory);
  }
  Ok(Computation::new(
    runtime.memory,
    runtime.statevector,
    Some(histogram_builder.histogram())
  ))
}