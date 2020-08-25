#![cfg(target_arch = "wasm32")]

use std::convert::From;

use js_sys::{self, Object};
use wasm_bindgen::prelude::JsValue;

use crate::error::QasmSimError;
use crate::interpreter::runtime::RuntimeError;
use crate::semantics::SemanticError;

impl From<QasmSimError<'_>> for JsValue {
    fn from(value: QasmSimError) -> Self {
        let message = format!("{}", &value);
        let obj = Object::new();

        set!(&obj,
            "message" => &message,
            "toString" => js_sys::Function::new_no_args("return this.message")
        );

        match value {
            QasmSimError::UnknownError(_) => {
                set!(&obj, "type" => "Unknown");
            }
            QasmSimError::InvalidToken {
                lineno,
                startpos,
                endpos,
                token,
                ..
            } => {
                set!(&obj,
                    "type" => "InvalidToken",
                    "lineNumber" => lineno as f64,
                    "startPosition" => startpos as f64
                );
                if let Some(endpos) = endpos {
                    set!(&obj, "endPosition" => endpos as f64);
                }
                if let Some(token) = token {
                    set!(&obj, "token" => &format!("{}", token));
                }
            }
            QasmSimError::UnexpectedEOF {
                lineno,
                startpos,
                endpos,
                token,
                ..
            } => {
                set!(&obj,
                    "type" => "UnexpectedEOF",
                    "lineNumber" => lineno as f64,
                    "startPosition" => startpos as f64
                );
                if let Some(endpos) = endpos {
                    set!(&obj, "endPosition" => endpos as f64);
                }
                if let Some(token) = token {
                    set!(&obj, "token" => &format!("{}", token));
                }
            }
            QasmSimError::UnexpectedToken {
                lineno,
                startpos,
                endpos,
                token,
                ..
            } => {
                set!(&obj,
                    "type" => "UnexpectedToken",
                    "lineNumber" => lineno as f64,
                    "startPosition" => startpos as f64
                );
                if let Some(endpos) = endpos {
                    set!(&obj, "endPosition" => endpos as f64);
                }
                if let Some(token) = token {
                    set!(&obj, "token" => &format!("{}", token));
                }
            }
            QasmSimError::RedefinitionError {
                symbol_name,
                lineno,
                previous_lineno,
                ..
            } => {
                set!(&obj,
                    "type" => "RedefinitionError",
                    "lineNumber" => lineno as f64,
                    "previousLineNumber" => previous_lineno as f64,
                    "symbolName" => &symbol_name
                );
            }
            QasmSimError::LibraryNotFound {
                libpath, lineno, ..
            } => {
                set!(&obj,
                    "type" => "LibraryNotFound",
                    "lineNumber" => lineno as f64,
                    "libPath" => &libpath
                );
            }
            QasmSimError::IndexOutOfBounds {
                lineno,
                symbol_name,
                index,
                size,
                ..
            } => {
                set!(&obj,
                    "type" => "IndexOutOfBounds",
                    "lineNumber" => lineno as f64,
                    "symbolName" => &symbol_name,
                    "index" => index as f64,
                    "size" => size as f64
                );
            }
            QasmSimError::SymbolNotFound {
                lineno,
                symbol_name,
                expected,
                ..
            } => {
                set!(&obj,
                    "type" => "SymbolNotFound",
                    "lineNumber" => lineno as f64,
                    "symbolName" => &symbol_name,
                    "expected" => &format!("{}", expected)
                );
            }
            QasmSimError::WrongNumberOfParameters {
                lineno,
                symbol_name,
                are_registers,
                given,
                expected,
                ..
            } => {
                set!(&obj,
                    "type" => "WrongNumberOfParameters",
                    "lineNumber" => lineno as f64,
                    "symbolName" => &symbol_name,
                    "kind" => if are_registers { "register" } else { "real" },
                    "given" => given as f64,
                    "expected" => expected as f64
                );
            }
            QasmSimError::UndefinedGate {
                symbol_name,
                lineno,
                ..
            } => {
                set!(&obj,
                    "type" => "UndefinedGate",
                    "lineNumber" => lineno as f64,
                    "symbolName" => &symbol_name
                );
            }
            QasmSimError::TypeMismatch {
                symbol_name,
                lineno,
                expected,
                ..
            } => {
                set!(&obj,
                    "type" => "TypeMismatch",
                    "lineNumber" => lineno as f64,
                    "symbolName" => &symbol_name,
                    "expected" => &format!("{}", expected)
                );
            }
            QasmSimError::RegisterSizeMismatch {
                symbol_name,
                lineno,
                ..
            } => {
                set!(&obj,
                    "type" => "RegisterSizeMismatch",
                    "lineNumber" => lineno as f64,
                    "symbolName" => &symbol_name
                );
            }
        };
        obj.into()
    }
}

impl From<RuntimeError> for JsValue {
    fn from(value: RuntimeError) -> Self {
        let message = format!("{}", &value);
        let obj = Object::new();

        set!(&obj,
            "message" => &message,
            "toString" => js_sys::Function::new_no_args("return this.message")
        );

        match value {
            RuntimeError::SemanticError(semantic_error) => {
                return semantic_error.into();
            }
            RuntimeError::Other => {
                set!(&obj, "type" => "Unknown");
            }
            RuntimeError::IndexOutOfBounds {
                location,
                symbol_name,
                index,
                size,
                ..
            } => {
                set!(&obj,
                    "type" => "IndexOutOfBounds",
                    "location" => location.0 as f64,
                    "symbolName" => &symbol_name,
                    "index" => index as f64,
                    "size" => size as f64
                );
            }
            RuntimeError::SymbolNotFound {
                location,
                symbol_name,
                expected,
                ..
            } => {
                set!(&obj,
                    "type" => "SymbolNotFound",
                    "location" => location.0 as f64,
                    "symbolName" => &symbol_name,
                    "expected" => &format!("{}", expected)
                );
            }
            RuntimeError::WrongNumberOfParameters {
                location,
                symbol_name,
                are_registers,
                given,
                expected,
                ..
            } => {
                set!(&obj,
                    "type" => "WrongNumberOfParameters",
                    "location" => location.0 as f64,
                    "symbolName" => &symbol_name,
                    "kind" => if are_registers { "register" } else { "real" },
                    "given" => given as f64,
                    "expected" => expected as f64
                );
            }
            RuntimeError::UndefinedGate {
                symbol_name,
                location,
                ..
            } => {
                set!(&obj,
                    "type" => "UndefinedGate",
                    "location" => location.0 as f64,
                    "symbolName" => &symbol_name
                );
            }
            RuntimeError::TypeMismatch {
                symbol_name,
                location,
                expected,
                ..
            } => {
                set!(&obj,
                    "type" => "TypeMismatch",
                    "location" => location.0 as f64,
                    "symbolName" => &symbol_name,
                    "expected" => &format!("{}", expected)
                );
            }
            RuntimeError::RegisterSizeMismatch {
                symbol_name,
                location,
                ..
            } => {
                set!(&obj,
                    "type" => "RegisterSizeMismatch",
                    "location" => location.0 as f64,
                    "symbolName" => &symbol_name
                );
            }
        };
        obj.into()
    }
}

impl From<SemanticError> for JsValue {
    fn from(value: SemanticError) -> Self {
        let message = format!("{}", &value);
        let obj = Object::new();

        set!(&obj,
            "message" => &message,
            "toString" => js_sys::Function::new_no_args("return this.message")
        );

        match value {
            SemanticError::RedefinitionError {
                symbol_name,
                location,
                previous_location,
                ..
            } => {
                set!(&obj,
                    "type" => "RedefinitionError",
                    "location" => location.0 as f64,
                    "previousLocation" => previous_location.0 as f64,
                    "symbolName" => &symbol_name
                );
            }
        };
        obj.into()
    }
}
