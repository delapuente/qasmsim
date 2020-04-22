macro_rules! measure {
    ($measure_name:expr, $block:block) => {{
        use web_sys;
        let window = web_sys::window().expect("get `window`");
        let performance = window.performance().expect("get `window.performance`");
        performance.clear_measures();
        performance.clear_marks();

        let start_mark = format!("{}_start", $measure_name);
        let end_mark = format!("{}_end", $measure_name);

        performance.mark(&start_mark).expect("set start mark");
        let result = $block;
        performance.mark(&end_mark).expect("set end mark");

        performance
            .measure_with_start_mark_and_end_mark(&$measure_name, &start_mark, &end_mark)
            .expect("set the measure");
        (result, &performance.get_entries_by_type(&"measure").get(0))
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
