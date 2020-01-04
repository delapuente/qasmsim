use std::collections::HashMap;
use grammar::ast;

#[derive(Debug, PartialEq)]
pub enum RegisterType {
  Q,
  C
}

/// Register name, type and size.
#[derive(Debug, PartialEq)]
pub struct RegisterEntry(String, RegisterType, usize);

/// Register name, start index, end index.
#[derive(Debug, PartialEq)]
pub struct MemoryMapEntry(String, usize, usize);

#[derive(Debug, PartialEq)]
pub struct Semantics {
  pub register_table: HashMap<String, RegisterEntry>,
  pub quantum_memory_map: HashMap<String, MemoryMapEntry>,
  pub classical_memory_map: HashMap<String, MemoryMapEntry>,
  pub quantum_memory_size: usize,
  pub classical_memory_size: usize
}

impl Semantics {
  pub fn new() -> Self {
    Semantics {
      register_table: HashMap::new(),
      quantum_memory_map: HashMap::new(),
      classical_memory_map: HashMap::new(),
      quantum_memory_size: 0,
      classical_memory_size: 0
    }
  }
}

struct SemanticsBuilder {
  semantics: Semantics,
  last_quantum_register: Option<String>,
  last_classical_register: Option<String>
}

impl SemanticsBuilder {
  pub fn new() -> Self {
    SemanticsBuilder {
      semantics: Semantics::new(),
      last_quantum_register: None,
      last_classical_register: None
    }
  }

  pub fn new_quantum_register(&mut self, name: String, size: usize)
  -> Result<(), String> {
    let result = self.new_register(name.clone(), RegisterType::Q, size);
    self.semantics.quantum_memory_size += size;
    self.last_quantum_register = Some(name);
    result
  }

  pub fn new_classical_register(&mut self, name: String, size: usize)
  -> Result<(), String> {
    let result = self.new_register(name.clone(), RegisterType::C, size);
    self.semantics.classical_memory_size += size;
    self.last_classical_register = Some(name);
    result
  }

  fn new_register(&mut self, name: String, kind: RegisterType, size: usize)
  -> Result<(), String> {
    if self.semantics.register_table.contains_key(&name) {
      return Err(format!("Register '{}' is already declared.", name))
    }
    self.semantics.register_table.insert(name.clone(), RegisterEntry(name, kind, size));
    Ok(())
  }
}

pub fn extract_semantics(tree: &ast::OpenQasmProgram)
-> Result<Semantics, String> {
  let mut builder = SemanticsBuilder::new();
  for statement in &tree.program {
    match statement {
      ast::Statement::QRegDecl(name, size) =>
        builder.new_quantum_register(name.clone(), *size)?,
      ast::Statement::CRegDecl(name, size) =>
        builder.new_classical_register(name.clone(), *size)?,
      _ => ()
    }
  }
  Ok(builder.semantics)
}

#[cfg(test)]
mod test {
  use super::*;
  use std::iter::FromIterator;
  use open_qasm2;

  #[test]
  fn test_symbol_table_stores_register_info() {
    let source = "
    OPENQASM 2.0;
    qreg q[2];
    creg c[2];
    qreg r[10];
    creg d[10];
    ";
    let tree = open_qasm2::OpenQasmProgramParser::new().parse(source).unwrap();
    let semantics_result = extract_semantics(&tree);
    assert!(semantics_result.is_ok());

    let expected_register_table = HashMap::from_iter(vec![
      ("q".to_owned(), RegisterEntry("q".to_owned(), RegisterType::Q, 2)),
      ("r".to_owned(), RegisterEntry("r".to_owned(), RegisterType::Q, 10)),
      ("c".to_owned(), RegisterEntry("c".to_owned(), RegisterType::C, 2)),
      ("d".to_owned(), RegisterEntry("d".to_owned(), RegisterType::C, 10))
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
    let tree = open_qasm2::OpenQasmProgramParser::new().parse(source).unwrap();
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
    let tree = open_qasm2::OpenQasmProgramParser::new().parse(source).unwrap();
    let semantics_result = extract_semantics(&tree);
    assert!(semantics_result.is_ok());
    if let Ok(semantics) = semantics_result {
      assert_eq!(semantics.classical_memory_size, 12);
    }
  }

  #[test]
  fn test_cannot_redeclare_a_register() {
    let sources = vec![
      "
      OPENQASM 2.0;
      qreg r[2];
      qreg r[2];
      ",
      "
      OPENQASM 2.0;
      qreg r[2];
      creg r[2];
      ",
      "
      OPENQASM 2.0;
      creg r[2];
      creg r[2];
      ",
      "
      OPENQASM 2.0;
      creg r[2];
      qreg r[2];
      ",
      "
      OPENQASM 2.0;
      qreg r[2];
      qreg r[20];
      ",
      "
      OPENQASM 2.0;
      creg r[2];
      creg r[20];
      "
    ];
    for (index, source) in sources.iter().enumerate() {
      let tree = open_qasm2::OpenQasmProgramParser::new().parse(source).unwrap();
      let semantics_result = extract_semantics(&tree);
      assert!(semantics_result.is_err());
      if let Err(error) = semantics_result {
        println!("Using source sample #{}", index);
        assert_eq!(error, "Register 'r' is already declared.");
      }
    }
  }

  #[test]
  fn test_quantum_memory_map() {
    let source = "
    OPENQASM 2.0;
    qreg q[2];
    creg c[2];
    qreg r[10];
    creg d[10];
    ";
    let tree = open_qasm2::OpenQasmProgramParser::new().parse(source).unwrap();
    let semantics_result = extract_semantics(&tree);
    assert!(semantics_result.is_ok());
    let expected_quantum_memory_map = HashMap::from_iter(vec![
      ("q".to_owned(), MemoryMapEntry("q".to_owned(), 0, 1)),
      ("r".to_owned(), MemoryMapEntry("r".to_owned(), 2, 11))
    ]);
    if let Ok(semantics) = semantics_result {
      assert_eq!(semantics.quantum_memory_map, expected_quantum_memory_map);
    }
  }

  #[test]
  fn test_classical_memory_map() {
    let source = "
    OPENQASM 2.0;
    qreg q[2];
    creg c[2];
    qreg r[10];
    creg d[10];
    ";
    let tree = open_qasm2::OpenQasmProgramParser::new().parse(source).unwrap();
    let semantics_result = extract_semantics(&tree);
    assert!(semantics_result.is_ok());
    let expected_classical_memory_map = HashMap::from_iter(vec![
      ("c".to_owned(), MemoryMapEntry("c".to_owned(), 0, 1)),
      ("d".to_owned(), MemoryMapEntry("d".to_owned(), 2, 11))
    ]);
    if let Ok(semantics) = semantics_result {
      assert_eq!(semantics.classical_memory_map, expected_classical_memory_map);
    }
  }
}