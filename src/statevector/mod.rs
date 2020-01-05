use std::f64;

use float_cmp::ApproxEq;
use cached::SizedCache;
use num::Float;
use complex::{ self, Complex, ComplexMargin };

pub mod wasm;

#[derive(Clone, Debug, PartialEq)]
pub struct StateVector {
  bases: Vec<Complex>,
  pub bit_width: usize
}

impl StateVector {

  pub fn new(bit_width: usize) -> Self {
    let mut bases = vec![Complex::new(0.0, 0.0); exp2(bit_width)];
    bases[0].re = 1.0;
    StateVector { bases, bit_width }
  }

  pub fn from_bases(bases: Vec<Complex>) -> Self {
    let bit_width = (bases.len() as f64).log2() as usize;
    StateVector { bases, bit_width }
  }

  pub fn len(&self) -> usize {
    self.bases.len()
  }

  /// Apply a controlled not
  pub fn cnot(&mut self, c: usize, t: usize) {
    let exchangable_rows = find_exchangeable_rows(self.bit_width, c, t);
    for (index_a, index_b) in exchangable_rows {
      self.bases.swap(index_a, index_b);
    }
  }

  /// Apply a 3 degree rotation to the target bit.
  pub fn u(&mut self, theta: f64, phi: f64, lambda: f64, target: usize) {
    let target_rows = find_target_rows(self.bit_width, target);
    let u_matrix = build_u(theta, phi, lambda);
    for (index_0, index_1) in target_rows {
      let selected = (self.bases[index_0], self.bases[index_1]);
      self.bases[index_0] = u_matrix.0 * selected.0 + u_matrix.1 * selected.1;
      self.bases[index_1] = u_matrix.2 * selected.0 + u_matrix.3 * selected.1;
    }
  }
}

impl<'a> ApproxEq for &'a StateVector {
  type Margin = ComplexMargin;

  fn approx_eq<T: Into<Self::Margin>>(self, other: Self, margin: T) -> bool {
    let margin = margin.into();
    for (c1, c2) in self.bases.iter().zip(&other.bases) {
      if c1.re.approx_ne(c2.re, margin) || c1.im.approx_ne(c2.im, margin) {
        return false
      }
    }
    true
  }
}

pub fn assert_approx_eq(v1: &StateVector, v2: &StateVector) {
  if !v1.approx_eq(v2, (complex::EPSILON, 0)) {
    assert!(false, "assertion failed `(left ~= right)`\n  left: `{:?}`\n right: `{:?}`", v1, v2);
  }
}

cached! {
  FIND_EXCHANGEABLE_ROWS;
  fn find_exchangeable_rows(bit_width: usize, c: usize, t: usize)
  -> Vec<(usize, usize)> = {
    let context_range = exp2(bit_width - 2);
    let mut out = Vec::with_capacity(context_range);
    for n in 0..context_range {
      let mut mask = 1;
      let mut histogram_index_10 = 0;
      let mut histogram_index_11 = 0;
      for i in 0..bit_width {
        if i == t {
          histogram_index_11 += exp2(t);
        }
        else if i == c {
          histogram_index_10 += exp2(c);
          histogram_index_11 += exp2(c);
        }
        else {
          let bit = ((n & mask) != 0) as usize;
          histogram_index_10 += bit * exp2(i);
          histogram_index_11 += bit * exp2(i);
          mask <<= 1;
        };
      }
      out.push((histogram_index_10, histogram_index_11))
    }
    out
  }
}

#[inline]
fn exp2(power: usize) -> usize {
  1_usize << power
}

cached! {
  FIND_TARGET_ROWS;
  fn find_target_rows(bit_width: usize, t: usize) -> Vec<(usize, usize)> = {
    let context_range = exp2(bit_width - 1);
    let mut out = Vec::with_capacity(context_range);
    for n in 0..context_range {
      let mut mask = 1;
      let mut histogram_index_0 = 0;
      let mut histogram_index_1 = 0;
      for i in 0..bit_width {
        if i == t {
          histogram_index_1 += exp2(t);
        }
        else {
          let bit = ((n & mask) != 0) as usize;
          histogram_index_0 += bit * exp2(i);
          histogram_index_1 += bit * exp2(i);
          mask <<= 1;
        };
      }
      out.push((histogram_index_0, histogram_index_1))
    }
    out
  }
}

type DecodedFloat = (u64, i16, i8);
type BuildUKey = (DecodedFloat, DecodedFloat, DecodedFloat);
type UMatrix = (Complex, Complex, Complex, Complex);

cached_key! {
  BUILD_U: SizedCache<BuildUKey, UMatrix> = SizedCache::with_size(20);
  Key = {
    (
      Float::integer_decode(theta),
      Float::integer_decode(phi),
      Float::integer_decode(lambda)
    )
  };
  fn build_u(theta: f64, phi: f64, lambda: f64) -> UMatrix = {
    (
      Complex::new((theta/2.0).cos(), 0.0),
      -e_power_to(lambda) * (theta/2.0).sin(),
      e_power_to(phi) * (theta/2.0).sin(),
      e_power_to(phi+lambda) * (theta/2.0).cos()
    )
  }
}


fn e_power_to(x: f64) -> Complex {
  Complex::new(0.0, x).exp()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_cnot_c0t1() {
    let p = Default::default();
    let a = Complex::new(1.0, 0.0);
    let b = Complex::new(0.0, 1.0);
    let mut v = StateVector::from_bases(vec!(p, a, p, b));
    v.cnot(0, 1);
    assert_eq!(v, StateVector::from_bases(vec!(p, b, p, a)));
  }

  #[test]
  fn test_cnot_c1t0_of_2_bits() {
    let p = Default::default();
    let a = Complex::new(1.0, 0.0);
    let b = Complex::new(0.0, 1.0);
    let mut v = StateVector::from_bases(vec!(p, p, a, b));
    v.cnot(1, 0);
    assert_eq!(v, StateVector::from_bases(vec!(p, p, b, a)));
  }

  #[test]
  fn test_cnot_c2t0_of_3_bits() {
    let p = Default::default();
    let a = Complex::new(1.0, 0.0);
    let b = Complex::new(0.0, 1.0);
    let mut v = StateVector::from_bases(vec!(p, p, p, p, a, b, a, b));
    v.cnot(2, 0);
    assert_eq!(v, StateVector::from_bases(vec!(p, p, p, p, b, a, b, a)));
  }

  #[test]
  fn test_cnot_c0t2_of_3_bits() {
    let p = Default::default();
    let a = Complex::new(1.0, 0.0);
    let b = Complex::new(0.0, 1.0);
    let mut v = StateVector::from_bases(vec!(p, a, p, a, p, b, p, b));
    v.cnot(0, 2);
    assert_eq!(v, StateVector::from_bases(vec!(p, b, p, b, p, a, p, a)));
  }

  #[test]
  fn test_cnot_is_reversible() {
    let p = Default::default();
    let a = Complex::new(1.0, 0.0);
    let b = Complex::new(0.0, 1.0);
    let mut v = StateVector::from_bases(vec!(p, a, p, b));
    v.cnot(0, 1);
    v.cnot(0, 1);
    assert_eq!(v, StateVector::from_bases(vec!(p, a, p, b)));
  }
}
