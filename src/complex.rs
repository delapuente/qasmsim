use float_cmp;
use num;

/// Alias for the float-64-based complex.
pub type Complex = num::Complex<f64>;
/// The margin withing two floats are considered the same is the same for each
/// component of a complex number.
pub type ComplexMargin = float_cmp::F64Margin;
