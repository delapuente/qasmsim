# Release notes

## Version 1.3.0

This version adds support for running the WASM version of the simulator inside a
web worker.

### Features
- The WASM version of the simulator can now be run inside a web worker. This
allows to run the simulator in a separate thread, and avoid blocking the main
thread.

## Version 1.2.0

This version adds documenting comments to OpenQASM 2.0. The specification
mentions the comment before a gate definition may document the gate. The
sepecification does not mention if it is just one comment or all the comments
that precede the gate definition. This implementation considers all the comments
immediately before the line of the gate definition as documentation.

```
// This comment is not documentation.

// This is also
// not documentation either.

// This comment, and the next one,
// are the documentation of the following gate.
gate id q { }
```

### Features
- The command line tool has now the `--info` option to extract the gate
documentation from the source code.
- The crate has a new function, [`get_gate_info`] to extract the signature of a
gate, and its documentation.
- The WASM version has a new function, `getGateInfo`, to extract the signature
of a gate, and its documentation.

[`get_gate_info`](https://docs.rs/qasmsim/latest/qasmsim/fn.get_gate_info.html)

## Version 1.1.0

This version makes `qasmsim` dual license: [APACHE] and [MIT] as recommended in
the [Rust API guidelines].

[APACHE]: LICENSE-APACHE.txt
[MIT]: LICENSE-MIT.txt
[Rust API guidelines]: https://rust-lang.github.io/api-guidelines/necessities.html#crate-and-its-dependencies-have-a-permissive-license-c-permissive

### Features
- `RuntimeError` and `SemanticError` now implements error-related traits
`std::error::Error` and `std::fmt::Display`.

## Version 1.0.0

Most of the features of OPENQASM are implemented. Main features missing are
including arbitrary files with the `include` directive, and using the comment
closest to a function as its documentation.

The `grammar` module is **unstable** and can introduce backward-compatibility
breaking-changes from release to release. Use at your own risk.