#![cfg(target_arch = "wasm32")]

use std::convert::From;

use wasm_bindgen::prelude::JsValue;
use js_sys::{ self, Object };

use crate::QasmSimError;

impl From<QasmSimError<'_>> for JsValue {
  fn from(value: QasmSimError) -> Self {
    let message = format!("{}", &value);
    let obj = Object::new();
    js_sys::Reflect::set(&obj,
      &"message".into(),
      &JsValue::from_str(&message)
    ).expect("set `message`");
    js_sys::Reflect::set(&obj,
      &"toString".into(),
      &js_sys::Function::new_no_args("return this.message").into()
    ).expect("set `toString`");

    match value {
      QasmSimError::UnknownError (_) => {
        js_sys::Reflect::set(&obj,
          &"type".into(),
          &JsValue::from_str("Unknown")
        ).expect("set `type`");
      },
      QasmSimError::SyntaxError {
        kind,
        lineoffset,
        lineno,
        startpos,
        endpos,
        token,
        ..
      } => {
        js_sys::Reflect::set(&obj,
          &"type".into(),
          &JsValue::from_str(&format!("{:?}", kind))
        ).expect("set `type`");
        js_sys::Reflect::set(&obj,
          &"lineOffset".into(),
          &JsValue::from_f64(lineoffset as f64)
        ).expect("set `lineOffset`");
        js_sys::Reflect::set(&obj,
          &"lineNumber".into(),
          &JsValue::from_f64(lineno as f64)
        ).expect("set `lineNumber`");
        js_sys::Reflect::set(&obj,
          &"startPosition".into(),
          &JsValue::from_f64(startpos as f64)
        ).expect("set `startPosition`");
        if let Some(endpos) = endpos {
          js_sys::Reflect::set(&obj,
            &"endPosition".into(),
            &JsValue::from_f64(endpos as f64)
          ).expect("set `endPosition`");
        }
        if let Some(token) = token {
          js_sys::Reflect::set(&obj,
            &"token".into(),
            &JsValue::from_str(&format!("{}", token))
          ).expect("set `token`");
        }
      }
      QasmSimError::SemanticError { symbol_name, .. } => {
        js_sys::Reflect::set(&obj,
          &"symbolName".into(),
          &JsValue::from_str(&symbol_name)
        ).expect("set `name`");
      }
    };
    obj.into()
  }
}