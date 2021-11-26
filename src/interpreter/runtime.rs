use std::collections::{HashMap, VecDeque};
use std::error;
use std::fmt;
use std::iter::FromIterator;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::grammar::{ast, lexer::Location};
use crate::interpreter::argument_solver::ArgumentSolver;
use crate::interpreter::computation::{Computation, HistogramBuilder};
use crate::interpreter::expression_solver::ExpressionSolver;
use crate::semantics::{extract_semantics, QasmType, RegisterType, SemanticError, Semantics};
use crate::statevector::StateVector;

type BindingMappings = (HashMap<String, f64>, HashMap<String, ast::Argument>);

/// Represent one of the possible errors that can happen during runtime.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RuntimeError {
    /// An unknown error.
    Other,
    /// A semantic error.
    SemanticError(SemanticError),
    /// Use of register index that does not fit the register size.
    IndexOutOfBounds {
        /// Abstract location in the code.
        location: Location,
        /// Name of the unknown gate.
        symbol_name: String,
        /// Index tried to access.
        index: usize,
        /// Size of the register.
        size: usize,
    },
    /// Use of an unknown/undeclared symbol.
    SymbolNotFound {
        /// Abstract location in the code.
        location: Location,
        /// Name of the unknown gate.
        symbol_name: String,
        /// The expected type.
        expected: QasmType,
    },
    /// The attempt of applying an operation passing the wrong number of
    /// parameters.
    WrongNumberOfParameters {
        /// Indicate if the parameters are registers or real values.
        are_registers: bool,
        /// Abstract location in the code.
        location: Location,
        /// Name of the unknown gate.
        symbol_name: String,
        /// The number of expected parameters.
        expected: usize,
        /// The number of passed parameters.
        given: usize,
    },
    /// Use of an unknown/undeclared symbol.
    UndefinedGate {
        /// Abstract location in the code.
        location: Location,
        /// Name of the unknown gate.
        symbol_name: String,
    },
    /// Found an unexpected type of value.
    TypeMismatch {
        /// Abstract location in the code.
        location: Location,
        /// Name of the unknown gate.
        symbol_name: String,
        /// Expected type.
        expected: QasmType,
    },
    /// Use of an unknown/undeclared symbol.
    RegisterSizeMismatch {
        /// Abstract location in the code.
        location: Location,
        /// Name of the unknown gate.
        symbol_name: String,
        /// Sizes of the different registers involved.
        sizes: Vec<usize>,
    },
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            RuntimeError::Other => "unknown error".to_string(),
            RuntimeError::SemanticError(semantic_error) => format!("{}", semantic_error),
            _ => match lazy_humanize! {
                self,
                RuntimeError::IndexOutOfBounds,
                RuntimeError::RegisterSizeMismatch,
                RuntimeError::SymbolNotFound,
                RuntimeError::TypeMismatch,
                RuntimeError::UndefinedGate,
                RuntimeError::WrongNumberOfParameters
            } {
                Some(message) => message,
                None => unreachable!(),
            },
        };
        write!(f, "{}", message)
    }
}

impl error::Error for RuntimeError {}

pub(crate) type Result<T> = std::result::Result<T, RuntimeError>;

impl From<SemanticError> for RuntimeError {
    fn from(semantic_error: SemanticError) -> Self {
        RuntimeError::SemanticError(semantic_error)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Runtime<'program> {
    macro_stack: VecDeque<BindingMappings>,
    semantics: Semantics,
    statevector: StateVector,
    memory: HashMap<String, u64>,
    location: Option<&'program Location>,
}

impl<'src, 'program> Runtime<'program> {
    pub fn new(semantics: Semantics) -> Self {
        let memory_size = semantics.quantum_memory_size;

        let mut runtime = Runtime {
            macro_stack: VecDeque::new(),
            semantics,
            statevector: StateVector::new(memory_size),
            memory: HashMap::new(),
            location: None,
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

    fn apply_gates(&mut self, statements: &'program [ast::Span<ast::Statement>]) -> Result<()> {
        for span in statements {
            self.location = Some(&span.boundaries.0);
            match &*span.node {
                ast::Statement::QuantumOperation(operation) => {
                    self.apply_quantum_operation(operation)?;
                }
                ast::Statement::Conditional(register, test, operation) => {
                    let actual_register = (register).clone();
                    let register_name = self.register_name(&actual_register);
                    self.assert_is_classical_register(register_name)?;

                    let value = match actual_register {
                        ast::Argument::Id(register_name) => self
                            .memory
                            .get(&register_name)
                            .expect("after `assert_is_classical_register()`, must exist"),
                        _ => unreachable!("cannot index a register inside the condition"),
                    };
                    if value == test {
                        self.apply_quantum_operation(operation)?;
                    }
                }
                _ => (),
            };
        }
        Ok(())
    }

    fn apply_quantum_operation(&mut self, operation: &ast::QuantumOperation) -> Result<()> {
        match operation {
            ast::QuantumOperation::Unitary(unitary) => self.apply_unitary(unitary),
            ast::QuantumOperation::Measure(source, target) => {
                self.apply_measurement(vec![(*source).clone(), (*target).clone()])
            }
            _ => Ok(()),
        }
    }

    fn apply_unitary(&mut self, unitary: &ast::UnitaryOperation) -> Result<()> {
        let name = &unitary.0;
        let real_args = &unitary.1;
        let args = &unitary.2;

        let actual_args = self.resolve_actual_args(args)?;
        self.check_all_are_quantum_registers(&actual_args)?;

        let solved_real_args = self.resolve_real_expressions(real_args)?;

        let expanded_arguments = self.expand_arguments(&actual_args).map_err(|sizes| {
            RuntimeError::RegisterSizeMismatch {
                location: *self
                    .location
                    .expect("after `apply_gates()`, the location of the statement"),
                symbol_name: name.clone(),
                sizes,
            }
        })?;

        for argument_expansion in expanded_arguments {
            self.apply_one_gate(name, &solved_real_args, &argument_expansion)?;
        }

        Ok(())
    }

    fn resolve_actual_args(&self, args: &[ast::Argument]) -> Result<Vec<ast::Argument>> {
        let actual = if !self.is_running_macro() {
            args.iter()
                .map(|argument| Ok(argument.clone()))
                .collect::<Result<Vec<ast::Argument>>>()
        } else {
            let stack_entry = self
                .macro_stack
                .get(0)
                .expect("if `is_running_macro()`, get first entry of the stack");
            let arg_bindings = &stack_entry.1;
            let argument_solver = ArgumentSolver::new(arg_bindings);
            args.iter()
                .map(|argument| {
                    argument_solver.solve(argument).map_err(|symbol_name| {
                        RuntimeError::SymbolNotFound {
                            location: *self
                                .location
                                .expect("after `apply_gates()`, the location of the statement"),
                            symbol_name,
                            expected: QasmType::QuantumRegister,
                        }
                    })
                })
                .collect::<Result<Vec<ast::Argument>>>()
        }?;
        Ok(actual)
    }

    fn resolve_real_expressions(&self, exprs: &[ast::Expression]) -> Result<Vec<f64>> {
        let mut real_bindings = &HashMap::new();
        if self.is_running_macro() {
            let stack_entry = self
                .macro_stack
                .get(0)
                .expect("if `is_running_macro()`, get first stack entry");
            real_bindings = &stack_entry.0;
        };
        let expression_solver = ExpressionSolver::new(real_bindings);
        let mut solved = Vec::new();
        for expression in exprs {
            let value = expression_solver.solve(expression).map_err(|symbol_name| {
                RuntimeError::SymbolNotFound {
                    location: *self
                        .location
                        .expect("after `apply_gates()`, the location of the statement"),
                    symbol_name,
                    expected: QasmType::RealValue,
                }
            })?;
            solved.push(value);
        }
        Ok(solved)
    }

    fn is_running_macro(&self) -> bool {
        !self.macro_stack.is_empty()
    }

    fn apply_measurement(&mut self, args: Vec<ast::Argument>) -> Result<()> {
        self.assert_is_quantum_register(self.register_name(&args[0]))?;
        self.assert_is_classical_register(self.register_name(&args[1]))?;

        let expanded_arguments =
            self.expand_arguments(&args)
                .map_err(|sizes| RuntimeError::RegisterSizeMismatch {
                    location: *self
                        .location
                        .expect("after `apply_gates()`, the location of the statement"),
                    symbol_name: "measure".into(),
                    sizes,
                })?;

        for argument_expansion in expanded_arguments {
            self.apply_one_measurement(argument_expansion)?;
        }

        Ok(())
    }

    fn apply_one_measurement(&mut self, args: Vec<ast::Argument>) -> Result<()> {
        let classical_register_name = self.register_name(&args[1]);
        let source = self.bit_mapping(&args[0])?;
        let measurement = self.statevector.measure(source) as u64;

        let target = self.bit_mapping(&args[1])?;
        let value = measurement * (1 << target);
        let prev_value = *(self
            .memory
            .get(classical_register_name)
            .expect("after `apply_measurement()`, get the entry"));
        self.memory
            .insert(classical_register_name.into(), prev_value + value);

        Ok(())
    }

    fn apply_one_gate(
        &mut self,
        name: &str,
        real_args: &[f64],
        args: &[ast::Argument],
    ) -> Result<()> {
        match name {
            "U" => {
                let theta = real_args[0];
                let phi = real_args[1];
                let lambda = real_args[2];
                let target = self.bit_mapping(&args[0])?;
                self.statevector.u(theta, phi, lambda, target);
            }
            "CX" => {
                let control = self.bit_mapping(&args[0])?;
                let target = self.bit_mapping(&args[1])?;
                self.statevector.cnot(control, target);
            }
            macro_name => {
                let binding_mappings = self.bind(macro_name.to_owned(), real_args, args)?;
                self.call(macro_name.to_owned(), binding_mappings)?;
            }
        };
        Ok(())
    }

    fn check_all_are_quantum_registers(&self, args: &[ast::Argument]) -> Result<()> {
        for argument in args {
            let register_name = self.register_name(argument);
            self.assert_is_quantum_register(register_name)?;
        }
        Ok(())
    }

    fn register_name<'a>(&self, arg: &'a ast::Argument) -> &'a str {
        match arg {
            ast::Argument::Id(name) => name,
            ast::Argument::Item(name, _) => name,
        }
    }

    fn assert_is_quantum_register(&self, name: &str) -> Result<()> {
        if !self.is_register_of_type(RegisterType::Q, name)? {
            Err(RuntimeError::TypeMismatch {
                location: *self
                    .location
                    .expect("after `apply_gates()`, the location of the statement"),
                symbol_name: name.into(),
                expected: QasmType::QuantumRegister,
            })
        } else {
            Ok(())
        }
    }

    fn assert_is_classical_register(&self, name: &str) -> Result<()> {
        if !self.is_register_of_type(RegisterType::C, name)? {
            Err(RuntimeError::TypeMismatch {
                location: *self
                    .location
                    .expect("after `apply_gates()`, the location of the statement"),
                symbol_name: name.into(),
                expected: QasmType::ClassicalRegister,
            })
        } else {
            Ok(())
        }
    }

    fn is_register_of_type(&self, rtype: RegisterType, name: &str) -> Result<bool> {
        match self.semantics.register_table.get(name) {
            Some(entry) => Ok(entry.1 == rtype),
            None => Err(RuntimeError::SymbolNotFound {
                location: *self
                    .location
                    .expect("after `apply_gates()`, the location of the statement"),
                symbol_name: name.into(),
                expected: match rtype {
                    RegisterType::Q => QasmType::QuantumRegister,
                    RegisterType::C => QasmType::ClassicalRegister,
                },
            }),
        }
    }

    fn apply_gate_operations(&mut self, operations: &[ast::GateOperation]) -> Result<()> {
        for one_operation in operations {
            if let ast::GateOperation::Unitary(unitary) = one_operation {
                self.apply_unitary(unitary)?;
            }
        }
        Ok(())
    }

    fn expand_arguments(
        &self,
        args: &[ast::Argument],
    ) -> std::result::Result<Vec<Vec<ast::Argument>>, Vec<usize>> {
        let range = self.range(args)?;
        Ok(range.map(|index| Runtime::specify(args, index)).collect())
    }

    fn bit_mapping(&self, argument: &ast::Argument) -> Result<usize> {
        match argument {
            ast::Argument::Item(name, index) => match self.semantics.memory_map.get(name) {
                None => Err(RuntimeError::SymbolNotFound {
                    location: *self
                        .location
                        .expect("after `apply_gates()`, location of the statement"),
                    symbol_name: name.into(),
                    expected: QasmType::Register,
                }),
                Some(mapping) => {
                    let size = mapping.2 - mapping.1 + 1;
                    if *index >= size {
                        return Err(RuntimeError::IndexOutOfBounds {
                            location: *self
                                .location
                                .expect("after `apply_gates()`, location of the statement"),
                            symbol_name: name.into(),
                            index: *index,
                            size,
                        });
                    }
                    Ok(mapping.1 + *index)
                }
            },
            _ => unreachable!("after `expand_arguments()`, argument should be Argument::Item"),
        }
    }

    fn range(
        &self,
        args: &[ast::Argument],
    ) -> std::result::Result<std::ops::Range<usize>, Vec<usize>> {
        // XXX: This is performed after validating the type of args.

        let whole_registers: Vec<&ast::Argument> = args
            .iter()
            .filter(|arg| matches!(arg, ast::Argument::Id(_)))
            .collect();

        // Return a one-iteration range, `specify()` takes care of ignoring Item arugments.
        if whole_registers.is_empty() {
            return Ok(0..1);
        }

        let all_sizes: Vec<usize> = whole_registers
            .iter()
            .map(|arg| {
                let register_name = self.register_name(arg);
                let register_entry = self
                    .semantics
                    .register_table
                    .get(register_name)
                    .expect("after validation, get register entry");
                register_entry.2
            })
            .collect();

        let reference_size = all_sizes[0];
        let all_the_same_size = all_sizes.iter().all(|size| *size == reference_size);

        if all_the_same_size {
            Ok(0..reference_size)
        } else {
            Err(all_sizes)
        }
    }

    fn specify(args: &[ast::Argument], index: usize) -> Vec<ast::Argument> {
        let mut result = vec![];
        for arg in args {
            match arg {
                ast::Argument::Id(name) => result.push(ast::Argument::Item(name.clone(), index)),
                other => result.push(other.clone()),
            }
        }
        result
    }

    fn bind(
        &mut self,
        macro_name: String,
        real_args: &[f64],
        args: &[ast::Argument],
    ) -> Result<BindingMappings> {
        let definition = match self.semantics.macro_definitions.get(&macro_name) {
            None => {
                return Err(RuntimeError::UndefinedGate {
                    location: *self
                        .location
                        .expect("after `apply_gates()`, the location of the statement"),
                    symbol_name: macro_name,
                });
            }
            Some(definition) => definition,
        };

        if real_args.len() != definition.1.len() {
            return Err(RuntimeError::WrongNumberOfParameters {
                are_registers: false,
                location: *self
                    .location
                    .expect("after `apply_gates()`, the location of the statement"),
                symbol_name: macro_name,
                given: real_args.len(),
                expected: definition.1.len(),
            });
        }
        let real_args_mapping = HashMap::from_iter(
            definition
                .1
                .iter()
                .zip(real_args.iter()) // pair formal arguments with their float values
                .map(|(s, f)| (s.to_owned(), *f)), // convert them into proper copies
        );

        if args.len() != definition.2.len() {
            return Err(RuntimeError::WrongNumberOfParameters {
                are_registers: true,
                location: *self
                    .location
                    .expect("after `apply_gates()`, the location of the statement"),
                symbol_name: macro_name,
                given: args.len(),
                expected: definition.2.len(),
            });
        }
        let args_mapping = HashMap::from_iter(
            definition
                .2
                .iter()
                .zip(args.iter().cloned()) // pair formal arguments with their registers
                .map(|(s, r)| (s.to_owned(), r)), // convert them into proper copies
        );

        Ok((real_args_mapping, args_mapping))
    }

    fn call(&mut self, macro_name: String, bindings: BindingMappings) -> Result<()> {
        // XXX: Why clonning is necessary??
        let definition = (*self.semantics.macro_definitions.get(&macro_name).unwrap()).clone();
        self.macro_stack.push_front(bindings);
        self.apply_gate_operations(&definition.3)?;
        self.macro_stack.pop_front();
        Ok(())
    }
}

/// Perform a simulation of the parsed `program`.
///
/// # Errors
///
/// Simulate can fail during runtime returning an `Err` variant with a value
/// of the [`RuntimeError`] type. `RuntimeError` is a sourceless error. It
/// can be related to a source code and converted into a more useful
/// [`QasmSimError`] value.
///
/// [`QasmSimError`]: ./error/enum.QasmSimError.html
/// [`RuntimeError`]: ./error/enum.RuntimeError.html
///
/// # Examples
///
/// Basic usage requires a valid AST as input. You can use
/// [`parse_and_link()`].
///
/// ```
/// # use qasmsim::QasmSimError;
/// # use qasmsim::grammar::ast::OpenQasmProgram;
/// # use qasmsim::parse_and_link;
/// use qasmsim::simulate;
///
/// # fn get_program_ast() -> OpenQasmProgram {
/// #     let source = r#"
/// #     OPENQASM 2.0;
/// #     include "qelib1.inc";
/// #     qreg q[2];
/// #     h q[0];
/// #     cx q[0], q[1];
/// #     "#;
/// #     parse_and_link(source).unwrap()
/// # }
///
/// let program = get_program_ast();
/// let computation = simulate(&program)?;
/// # use qasmsim::error::RuntimeError;
/// # Ok::<(), RuntimeError>(())
/// ```
///
/// [`parse_and_link()`]: ./fn.parse_and_link.html
pub fn simulate(program: &ast::OpenQasmProgram) -> Result<Computation> {
    let semantics = extract_semantics(program)?;
    let mut runtime = Runtime::new(semantics);
    runtime.apply_gates(&program.program)?;
    Ok(Computation::new(runtime.memory, runtime.statevector, None))
}

/// Perform `shots` number of simulations of the parsed proram `program`.
///
/// # Errors
///
/// Simulate can fail during runtime returning an `Err` variant with a value
/// of the [`RuntimeError`] type. `RuntimeError` is a sourceless error. It
/// can be related to a source code and converted into a more useful
/// [`QasmSimError`] value.
///
/// [`QasmSimError`]: ./error/enum.QasmSimError.html
/// [`RuntimeError`]: ./error/enum.RuntimeError.html
///
/// # Examples
///
/// Basic usage requires a valid AST as input. You can use
/// [`parse_and_link()`].
///
/// ```
/// # use qasmsim::QasmSimError;
/// # use qasmsim::grammar::ast::OpenQasmProgram;
/// # use qasmsim::parse_and_link;
/// use qasmsim::simulate_with_shots;
///
/// # fn get_program_ast() -> OpenQasmProgram {
/// #     let source = r#"
/// #     OPENQASM 2.0;
/// #     include "qelib1.inc";
/// #     qreg q[2];
/// #     h q[0];
/// #     cx q[0], q[1];
/// #     "#;
/// #     parse_and_link(source).unwrap()
/// # }
///
/// let program = get_program_ast();
/// let computation = simulate_with_shots(&program, 1024)?;
/// # use qasmsim::error::RuntimeError;
/// # Ok::<(), RuntimeError>(())
/// ```
///
/// [`parse_and_link()`]: ./fn.parse_and_link.html
pub fn simulate_with_shots(program: &ast::OpenQasmProgram, shots: usize) -> Result<Computation> {
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
        Some(histogram_builder.histogram()),
    ))
}
