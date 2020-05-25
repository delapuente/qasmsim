# Release notes

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