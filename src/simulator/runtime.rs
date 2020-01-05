use std::error::Error;

use semantics::{ Semantics, RegisterType, extract_semantics};
use statevector::StateVector;
use grammar::ast;
use gatelib;

struct Runtime {
  memory_size: usize,
  statevector: StateVector
}

impl Runtime {
  pub fn new(memory_size: usize) -> Self {
    Runtime { memory_size, statevector: StateVector::new(memory_size) }
  }
}

pub fn execute(program: &ast::OpenQasmProgram)
-> Result<StateVector, String> {
  let semantics = extract_semantics(program)?;
  let size = semantics.quantum_memory_size;
  let mut runtime = Runtime::new(size);
  runtime = apply_gates(&semantics, &program.program, runtime);
  Ok(runtime.statevector)
}

fn apply_gates(semantics: &Semantics, statements: &Vec<ast::Statement>, mut runtime: Runtime)
-> Runtime {
  for statement in statements {
    match statement {
      ast::Statement::QuantumOperation(operation) => {
        runtime = apply_quantum_operation(semantics, operation, runtime);
      }
      _ => ()
    };
  }
  runtime
}

fn apply_quantum_operation(semantics: &Semantics, operation: &ast::QuantumOperation, mut runtime: Runtime)
-> Runtime {
  match operation {
    ast::QuantumOperation::Unitary(unitary) => {
      runtime = apply_unitary(semantics, unitary, runtime)
    }
    _ => ()
  };
  runtime
}

fn apply_unitary(semantics: &Semantics, unitary: &ast::UnitaryOperation, mut runtime: Runtime)
-> Runtime {
  match unitary {
    ast::UnitaryOperation::GateExpansion(name, real_args, args) => {
      for argument_expansion in expand_arguments(semantics, args) {
        runtime = apply_one_gate(semantics, name, real_args, &argument_expansion, runtime)
      }
    }
    _ => ()
  }
  runtime
}

fn apply_one_gate(semantics: &Semantics, name: &str, real_args: &Vec<ast::Expression>, args: &Vec<ast::Argument>, mut runtime: Runtime)
-> Runtime {
  match name {
    "h" => {
      let index = get_bit_mapping(semantics, &args[0]);
      runtime.statevector = gatelib::h(index, runtime.statevector);
    }
    "cx" => {
      let control = get_bit_mapping(semantics, &args[0]);
      let target = get_bit_mapping(semantics, &args[1]);
      runtime.statevector = gatelib::cx(control, target, runtime.statevector);
    }
    macro_name => {
      // let solved_real_args = solve(real_args);
      // let binding = bind(semantics, macro_name, solved_real_args, args);
      // runtime = apply_gates(semantics, binding.program, runtime);
    }
  }
  runtime
}

fn get_bit_mapping(semantics: &Semantics, argument: &ast::Argument) -> usize {
  match argument {
    ast::Argument::Item(name, index) => {
      let mapping = semantics.memory_map.get(name).unwrap();
      mapping.1 + *index
    }
    _ => unreachable!()
  }
}

fn expand_arguments(semantics: &Semantics, args: &Vec<ast::Argument>)
-> Vec<Vec<ast::Argument>> {
  let range = get_range(semantics, args);
  range.map(|index| specify(args, index)).collect()
}

fn get_range(semantics: &Semantics, args: &Vec<ast::Argument>) -> std::ops::Range<usize> {
  for arg in args {
    match arg {
      ast::Argument::Id(name) => {
        return 0..semantics.register_table.get(name).unwrap().2;
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