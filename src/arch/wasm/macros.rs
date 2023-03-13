use wasm_bindgen::prelude::wasm_bindgen;

// XXX: there is no way to access the `performance` object from `web_sys`
// inside web workers because `web_sys` only exposes the `Window` object, not
// available in a worker context. Instead, as workaround, we use `wasm_bindgen`
// with `extern "C"` to access the `performance` object directly.
// https://github.com/rustwasm/wasm-bindgen/issues/1752
// https://rustwasm.github.io/wasm-bindgen/reference/attributes/on-js-imports/js_name.html
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = self, js_name = performance)]
    pub static JS_PERFORMANCE: web_sys::Performance;
}

macro_rules! measure {
    ($measure_name:expr, $block:block) => {{
        use crate::arch::wasm::macros::JS_PERFORMANCE;

        JS_PERFORMANCE.clear_measures();
        JS_PERFORMANCE.clear_marks();

        let start_mark = format!("{}_start", $measure_name);
        let end_mark = format!("{}_end", $measure_name);

        JS_PERFORMANCE.mark(&start_mark).expect("set start mark");
        let result = $block;
        JS_PERFORMANCE.mark(&end_mark).expect("set end mark");

        JS_PERFORMANCE
            .measure_with_start_mark_and_end_mark(&$measure_name, &start_mark, &end_mark)
            .expect("set the measure");
        (
            result,
            &JS_PERFORMANCE.get_entries_by_type(&"measure").get(0),
        )
    }};
}

macro_rules! set {
  ($obj:expr, $( $key:expr => $value:expr ),*) => {
    {
      use js_sys;
      $(
        js_sys::Reflect::set($obj, &$key.into(), &$value.into()).expect(&format!("set `{}`", $key));
      )*
    }
  };
}
