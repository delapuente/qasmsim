#![cfg(target_arch = "wasm32")]

use std::convert::From;

use wasm_bindgen::prelude::JsValue;
use js_sys::{ self, Object };

use crate::QasmSimError;

impl From<QasmSimError<'_>> for JsValue {
  fn from(value: QasmSimError) -> Self {
    let message = format!("{}", &value);
    let obj = Object::new();

    set!(&obj,
      "message" => &message,
      "toString" => js_sys::Function::new_no_args("return this.message")
    );

    match value {
      QasmSimError::UnknownError (_) => {
        set!(&obj, "type" => "Unknown");
      },
      QasmSimError::SyntaxError {
        kind,
        lineno,
        startpos,
        endpos,
        token,
        ..
      } => {
        set!(&obj,
          "type" => &format!("{:?}", kind),
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
      QasmSimError::SemanticError { symbol_name } => {
        set!(&obj, "symbolName" => &symbol_name);
      }
      QasmSimError::LinkerError { libpath } => {
        set!(&obj, "libPath" => &libpath);
      }
      QasmSimError::RuntimeError { kind, symbol_name } => {
        set!(&obj,
          "type" => &format!("{:?}", kind),
          "symbolName" => &symbol_name
        );
      }
    };
    obj.into()
  }
}