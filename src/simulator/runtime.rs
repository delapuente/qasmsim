use statevector::StateVector;
use grammar::open_qasm2::ast;
use complex::Complex;
use std::io;
use gatelib;

pub fn execute(program: &ast::OpenQasmProgram) -> StateVector {
  let size = extract_size(program).unwrap();
  let mut state_vector = Vec::with_capacity(10);
  for _ in 0..2_u32.pow(size as u32) {
    state_vector.push(Complex(0.0,0.0))
  }
  state_vector[0].0 = 1.0;
  apply_gates(&program, state_vector)
}

fn extract_size(program: &ast::OpenQasmProgram) -> Result<usize, io::Error> {
  for statement in &program.program {
    match statement {
      ast::Statement::QRegDecl(_, v) => { return Ok(*v); }
      _ => ()
    };
  }
  panic!("There is no quantum register.")
}

fn apply_gates(program: &ast::OpenQasmProgram, mut state_vector: StateVector)
-> StateVector {
  for statement in &program.program {
    match statement {
      ast::Statement::QuantumOperation(operation) => {
        state_vector = apply_quantum_operation(operation, state_vector);
      }
      _ => ()
    };
  }
  state_vector
}

fn apply_quantum_operation(operation: &ast::QuantumOperation, mut state_vector: StateVector)
-> StateVector {
  match operation {
    ast::QuantumOperation::Unitary(unitary) => {
      state_vector = apply_unitary(unitary, state_vector)
    }
    _ => ()
  };
  state_vector
}

fn apply_unitary(unitary: &ast::UnitaryOperation, mut state_vector: StateVector)
-> StateVector {
  match unitary {
    ast::UnitaryOperation::GateExpansion(name, real_args, args) => {
      state_vector = apply_one_gate(name, real_args, args, state_vector)
    }
    _ => ()
  }
  state_vector
}

fn apply_one_gate(name: &str, real_args: &Vec<ast::Expression>, args: &Vec<ast::Argument>, mut state_vector: StateVector)
-> StateVector {
  match name {
    "h" => {
      let index = get_bit_index(&args[0]);
      state_vector = gatelib::h(index, state_vector);
    }
    "cx" => {
      let control = get_bit_index(&args[0]);
      let target = get_bit_index(&args[1]);
      state_vector = gatelib::cx(control, target, state_vector);
    }
    _ => ()
  }
  state_vector
}

fn get_bit_index(argument: &ast::Argument) -> usize {
  match argument {
    ast::Argument::Id(_) => 0,
    ast::Argument::Item(_, index) => *index
  }
}