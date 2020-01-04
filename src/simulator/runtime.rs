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
-> Result<StateVector, Box<dyn Error>> {
  let semantics = extract_semantics(program)?;
  let size = semantics.quantum_memory_size;
  let mut runtime = Runtime::new(size);
  runtime = apply_gates(&semantics, &program, runtime);
  Ok(runtime.statevector)
}

fn apply_gates(semantics: &Semantics, program: &ast::OpenQasmProgram, mut runtime: Runtime)
-> Runtime {
  for statement in &program.program {
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
      runtime = apply_one_gate(semantics, name, real_args, args, runtime)
    }
    _ => ()
  }
  runtime
}

fn apply_one_gate(semantics: &Semantics, name: &str, real_args: &Vec<ast::Expression>, args: &Vec<ast::Argument>, mut runtime: Runtime)
-> Runtime {
  match name {
    "h" => {
      let indices = get_bit_indices(semantics, &args[0]);
      for index in indices {
        runtime.statevector = gatelib::h(index, runtime.statevector);
      }
    }
    "cx" => {
      let controls = get_bit_indices(semantics, &args[0]);
      let targets = get_bit_indices(semantics, &args[1]);
      for (control, target) in controls.iter().zip(targets) {
        runtime.statevector = gatelib::cx(*control, target, runtime.statevector);
      }
    }
    _ => ()
  }
  runtime
}

fn get_bit_indices(semantics: &Semantics, argument: &ast::Argument) -> Vec<usize> {
  match argument {
    ast::Argument::Id(name) => {
      let mapping = semantics.memory_map.get(name).unwrap();
      (mapping.1..mapping.2 + 1).collect()
    },
    ast::Argument::Item(name, index) => {
      let mapping = semantics.memory_map.get(name).unwrap();
      vec!(mapping.1 + *index)
    }
  }
}