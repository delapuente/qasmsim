use num;
use float_cmp;

pub type Complex = num::Complex<f64>;
pub type ComplexMargin = float_cmp::F64Margin;
pub const EPSILON: f64 = std::f64::EPSILON;