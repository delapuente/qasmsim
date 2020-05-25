use std::collections::HashMap;
use std::error;
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::grammar::ast;
use crate::grammar::lexer::Location;

/// The different types for OPENQASM values.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum QasmType {
    /// A generic register.
    Register,
    /// A quantum register.
    QuantumRegister,
    /// A classical register.
    ClassicalRegister,
    /// A real value.
    RealValue,
}

impl fmt::Display for QasmType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                QasmType::RealValue => "real value",
                QasmType::Register => "register",
                QasmType::QuantumRegister => "quantum register",
                QasmType::ClassicalRegister => "classical register",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RegisterType {
    Q,
    C,
}

/// Represent the possible semantic errors.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SemanticError {
    /// Attempt to redefine an already defined register.
    RedefinitionError {
        /// Name of the redefined register.
        symbol_name: String,
        /// Location where the redefinition happens.
        location: Location,
        /// Location of the original definition.
        previous_location: Location,
    },
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            _ => match lazy_humanize!{
                self,
                SemanticError::RedefinitionError
            } {
                Some(message) => message,
                None => unreachable!()
            }
        };
        write!(f, "{}", message)
    }
}

impl error::Error for SemanticError {}

type Result<T> = std::result::Result<T, SemanticError>;

/// Register name, type, size and definition location.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegisterEntry(pub String, pub RegisterType, pub usize, pub Location);

/// Register name, start index, end index.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemoryMapEntry(pub String, pub usize, pub usize);

/// Macro name, real arguments, register arguments, list of statements and definition location.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroDefinition(
    pub String,
    pub Vec<String>,
    pub Vec<String>,
    pub Vec<ast::GateOperation>,
    pub Location,
);

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Semantics {
    pub macro_definitions: HashMap<String, MacroDefinition>,
    pub register_table: HashMap<String, RegisterEntry>,
    /// Map quantum registers to a unique unified register while classical
    /// registers map to themselves.
    pub memory_map: HashMap<String, MemoryMapEntry>,
    pub quantum_memory_size: usize,
    pub classical_memory_size: usize,
}

#[derive(Debug, Clone, PartialEq, Default)]
struct SemanticsBuilder {
    semantics: Semantics,
    last_quantum_register: Option<String>,
    last_classical_register: Option<String>,
}

impl SemanticsBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_quantum_register(
        &mut self,
        name: String,
        size: usize,
        location: Location,
    ) -> Result<()> {
        self.new_register(name.clone(), RegisterType::Q, size, location)?;
        self.map_register(name.clone(), RegisterType::Q, size);
        self.semantics.quantum_memory_size += size;
        self.last_quantum_register = Some(name);
        Ok(())
    }

    pub fn new_classical_register(
        &mut self,
        name: String,
        size: usize,
        location: Location,
    ) -> Result<()> {
        self.new_register(name.clone(), RegisterType::C, size, location)?;
        self.map_register(name.clone(), RegisterType::C, size);
        self.semantics.classical_memory_size += size;
        self.last_classical_register = Some(name);
        Ok(())
    }

    pub fn new_gate(
        &mut self,
        name: String,
        real_args: Vec<String>,
        args: Vec<String>,
        body: Vec<ast::GateOperation>,
        location: Location,
    ) -> Result<()> {
        let entry = self.semantics.macro_definitions.get(&name);
        if let Some(MacroDefinition(_, _, _, _, previous_location)) = entry {
            return Err(SemanticError::RedefinitionError {
                symbol_name: name,
                location,
                previous_location: *previous_location,
            });
        }

        self.semantics.macro_definitions.insert(
            name.clone(),
            MacroDefinition(name, real_args, args, body, location),
        );

        Ok(())
    }

    fn new_register(
        &mut self,
        name: String,
        kind: RegisterType,
        size: usize,
        location: Location,
    ) -> Result<()> {
        let entry = self.semantics.register_table.get(&name);
        if let Some(RegisterEntry(_, _, _, previous_location)) = entry {
            return Err(SemanticError::RedefinitionError {
                symbol_name: name,
                location,
                previous_location: *previous_location,
            });
        }

        self.semantics
            .register_table
            .insert(name.clone(), RegisterEntry(name, kind, size, location));

        Ok(())
    }

    fn map_register(&mut self, name: String, kind: RegisterType, size: usize) {
        match &kind {
            RegisterType::Q => self.map_quantum_register(name, size),
            RegisterType::C => self.map_classical_register(name, size),
        }
    }

    pub fn map_quantum_register(&mut self, name: String, size: usize) {
        let new_entry = match &self.last_quantum_register {
            None => MemoryMapEntry(name.clone(), 0, size - 1),
            Some(register_name) => {
                let last_index = self
                    .semantics
                    .memory_map
                    .get(register_name)
                    .expect("get last register")
                    .2;
                MemoryMapEntry(name.clone(), last_index + 1, last_index + size)
            }
        };
        self.semantics.memory_map.insert(name, new_entry);
    }

    pub fn map_classical_register(&mut self, name: String, size: usize) {
        self.semantics
            .memory_map
            .insert(name.clone(), MemoryMapEntry(name, 0, size - 1));
    }
}

pub fn extract_semantics(tree: &ast::OpenQasmProgram) -> Result<Semantics> {
    let mut builder = SemanticsBuilder::new();
    for span in &tree.program {
        let location = span.boundaries.0;
        match &*span.node {
            ast::Statement::QRegDecl(name, size) => {
                builder.new_quantum_register(name.clone(), *size, location)?
            }
            ast::Statement::CRegDecl(name, size) => {
                builder.new_classical_register(name.clone(), *size, location)?
            }
            ast::Statement::GateDecl(name, real_args, args, operations) => builder.new_gate(
                name.clone(),
                real_args.to_vec(),
                args.to_vec(),
                operations.to_vec(),
                location,
            )?,
            _ => (),
        }
    }
    Ok(builder.semantics)
}

#[cfg(test)]
mod test {
    use indoc::indoc;

    use super::*;
    use std::iter::FromIterator;

    use crate::grammar::parse_program;

    #[test]
    fn test_symbol_table_stores_register_info() {
        let source = indoc!(
            "
    OPENQASM 2.0;
    qreg q[2];
    creg c[2];
    qreg r[10];
    creg d[10];
    "
        );
        let tree = parse_program(source).unwrap();
        let semantics_result = extract_semantics(&tree);
        assert!(semantics_result.is_ok());

        let expected_register_table = HashMap::from_iter(vec![
            (
                "q".to_owned(),
                RegisterEntry("q".to_owned(), RegisterType::Q, 2, Location(14)),
            ),
            (
                "r".to_owned(),
                RegisterEntry("r".to_owned(), RegisterType::Q, 10, Location(36)),
            ),
            (
                "c".to_owned(),
                RegisterEntry("c".to_owned(), RegisterType::C, 2, Location(25)),
            ),
            (
                "d".to_owned(),
                RegisterEntry("d".to_owned(), RegisterType::C, 10, Location(48)),
            ),
        ]);
        if let Ok(semantics) = semantics_result {
            assert_eq!(semantics.register_table, expected_register_table);
        }
    }

    #[test]
    fn test_total_quantum_memory_size_is_ok() {
        let source = "
    OPENQASM 2.0;
    qreg q[2];
    creg c[2];
    qreg r[10];
    creg d[10];
    ";
        let tree = parse_program(source).unwrap();
        let semantics_result = extract_semantics(&tree);
        assert!(semantics_result.is_ok());
        if let Ok(semantics) = semantics_result {
            assert_eq!(semantics.quantum_memory_size, 12);
        }
    }

    #[test]
    fn test_total_classical_memory_size_is_ok() {
        let source = "
    OPENQASM 2.0;
    qreg q[2];
    creg c[2];
    qreg r[10];
    creg d[10];
    ";
        let tree = parse_program(source).unwrap();
        let semantics_result = extract_semantics(&tree);
        assert!(semantics_result.is_ok());
        if let Ok(semantics) = semantics_result {
            assert_eq!(semantics.classical_memory_size, 12);
        }
    }

    #[test]
    fn test_cannot_redeclare_a_register() {
        let sources = vec![
            indoc!(
                "
      OPENQASM 2.0;
      qreg r[2];
      qreg r[2];
      "
            ),
            indoc!(
                "
      OPENQASM 2.0;
      qreg r[2];
      creg r[2];
      "
            ),
            indoc!(
                "
      OPENQASM 2.0;
      creg r[2];
      creg r[2];
      "
            ),
            indoc!(
                "
      OPENQASM 2.0;
      creg r[2];
      qreg r[2];
      "
            ),
            indoc!(
                "
      OPENQASM 2.0;
      qreg r[2];
      qreg r[20];
      "
            ),
            indoc!(
                "
      OPENQASM 2.0;
      creg r[2];
      creg r[20];
      "
            ),
        ];
        for (index, source) in sources.iter().enumerate() {
            let tree = parse_program(source).unwrap();
            let error = extract_semantics(&tree).expect_err("should be a redeclaration error");
            println!("Using source sample #{}", index);
            assert_eq!(
                error,
                SemanticError::RedefinitionError {
                    symbol_name: "r".into(),
                    location: Location(25),
                    previous_location: Location(14)
                }
            );
        }
    }

    #[test]
    fn test_memory_map() {
        let source = "
    OPENQASM 2.0;
    qreg q[2];
    creg c[2];
    qreg r[10];
    creg d[10];
    ";
        let tree = parse_program(source).unwrap();
        let semantics_result = extract_semantics(&tree);
        assert!(semantics_result.is_ok());
        let expected_memory_map = HashMap::from_iter(vec![
            ("q".to_owned(), MemoryMapEntry("q".to_owned(), 0, 1)),
            ("r".to_owned(), MemoryMapEntry("r".to_owned(), 2, 11)),
            ("c".to_owned(), MemoryMapEntry("c".to_owned(), 0, 1)),
            ("d".to_owned(), MemoryMapEntry("d".to_owned(), 0, 9)),
        ]);
        if let Ok(semantics) = semantics_result {
            assert_eq!(semantics.memory_map, expected_memory_map);
        }
    }

    #[test]
    fn test_macro_definitions() {
        let source = indoc!(
            "
    OPENQASM 2.0;
    gate only_qubits q {
      h q;
    }
    qreg q[2];
    U(0, 0, 0) q;
    gate reals_and_qubits (a, b) q, r {
      U(a/b, 0, 0) q;
    }
    "
        );
        let tree = parse_program(source).unwrap();
        let semantics_result = extract_semantics(&tree);
        assert!(semantics_result.is_ok());
        let expected_definitions = HashMap::from_iter(vec![
            (
                "only_qubits".to_owned(),
                MacroDefinition(
                    "only_qubits".to_owned(),
                    vec![],
                    vec!["q".to_owned()],
                    vec![ast::GateOperation::Unitary(ast::UnitaryOperation(
                        "h".to_owned(),
                        vec![],
                        vec![ast::Argument::Id("q".to_owned())],
                    ))],
                    Location(14),
                ),
            ),
            (
                "reals_and_qubits".to_owned(),
                MacroDefinition(
                    "reals_and_qubits".to_owned(),
                    vec!["a".to_owned(), "b".to_owned()],
                    vec!["q".to_owned(), "r".to_owned()],
                    vec![ast::GateOperation::Unitary(ast::UnitaryOperation(
                        "U".to_owned(),
                        vec![
                            ast::Expression::Op(
                                ast::OpCode::Div,
                                Box::new(ast::Expression::Id("a".to_owned())),
                                Box::new(ast::Expression::Id("b".to_owned())),
                            ),
                            ast::Expression::Real(0.0),
                            ast::Expression::Real(0.0),
                        ],
                        vec![ast::Argument::Id("q".to_owned())],
                    ))],
                    Location(69),
                ),
            ),
        ]);
        if let Ok(semantics) = semantics_result {
            assert_eq!(semantics.macro_definitions, expected_definitions);
        }
    }
}
