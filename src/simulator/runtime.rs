use std::io;
use std::error::Error;

use statevector::StateVector;
use grammar::open_qasm2::ast;
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
  let size = extract_size(program)?;
  let mut runtime = Runtime::new(size);
  runtime = apply_gates(&program, runtime);
  Ok(runtime.statevector)
}

fn extract_size(program: &ast::OpenQasmProgram) -> Result<usize, &'static str> {
  for statement in &program.program {
    match statement {
      ast::Statement::QRegDecl(_, v) => { return Ok(*v); }
      _ => ()
    };
  }
  Err("No quantum register declaration found.")
}

fn apply_gates(program: &ast::OpenQasmProgram, mut runtime: Runtime)
-> Runtime {
  for statement in &program.program {
    match statement {
      ast::Statement::QuantumOperation(operation) => {
        runtime = apply_quantum_operation(operation, runtime);
      }
      _ => ()
    };
  }
  runtime
}

fn apply_quantum_operation(operation: &ast::QuantumOperation, mut runtime: Runtime)
-> Runtime {
  match operation {
    ast::QuantumOperation::Unitary(unitary) => {
      runtime = apply_unitary(unitary, runtime)
    }
    _ => ()
  };
  runtime
}

fn apply_unitary(unitary: &ast::UnitaryOperation, mut runtime: Runtime)
-> Runtime {
  match unitary {
    ast::UnitaryOperation::GateExpansion(name, real_args, args) => {
      runtime = apply_one_gate(name, real_args, args, runtime)
    }
    _ => ()
  }
  runtime
}

fn apply_one_gate(name: &str, real_args: &Vec<ast::Expression>, args: &Vec<ast::Argument>, mut runtime: Runtime)
-> Runtime {
  match name {
    "h" => {
      let index = get_bit_index(&args[0]);
      runtime.statevector = gatelib::h(index, runtime.statevector);
    }
    "cx" => {
      let control = get_bit_index(&args[0]);
      let target = get_bit_index(&args[1]);
      runtime.statevector = gatelib::cx(control, target, runtime.statevector);
    }
    _ => ()
  }
  runtime
}

fn get_bit_index(argument: &ast::Argument) -> usize {
  match argument {
    ast::Argument::Id(_) => 0,
    ast::Argument::Item(_, index) => *index
  }
}