use statevector::StateVector;
use std::f64::consts::PI;

pub fn u2(phi: f64, lambda: f64, t: usize, v: StateVector) -> StateVector {
  v.u(PI/2.0, phi, lambda, t)
}

pub fn h(t: usize, v: StateVector) -> StateVector {
  u2(0.0, PI, t, v)
}

pub fn x(t: usize, v: StateVector) -> StateVector {
  v.u(PI, 0.0, PI, t)
}

pub fn z(t: usize, v: StateVector) -> StateVector {
  v.u(0.0, 0.0, PI, t)
}

pub fn cx(c: usize, t: usize, v: StateVector) -> StateVector {
  v.cnot(c, t)
}

#[cfg(test)]
mod tests {
  use super::*;
  use complex::Complex;
  use std::f64::consts::SQRT_2;
  use statevector::assert_approx_eq;

  #[test]
  fn test_bit_flip() {
    let zero = Default::default();
    let whole = Complex::new(1.0, 0.0);
    let mut v = StateVector::from_bases(vec!(whole, zero));
    v = x(0, v);
    assert_approx_eq(&v, &StateVector::from_bases(vec!(zero, whole)));
  }

  #[test]
  fn test_bit_flip_is_reversible() {
    let zero = Default::default();
    let whole = Complex::new(1.0, 0.0);
    let mut v = StateVector::from_bases(vec!(whole, zero));
    v = x(0, v);
    v = x(0, v);
    assert_approx_eq(&v, &StateVector::from_bases(vec!(whole, zero)));
  }

  #[test]
  fn test_bit_flip_x1_on_3_bits() {
    let z = Default::default();
    let w = Complex::new(1.0, 0.0);
    let mut v = StateVector::from_bases(vec!(w, z, z, z, z, z, z, z));
    v = x(1, v);
    assert_approx_eq(&v, &StateVector::from_bases(vec!(z, z, w, z, z, z, z, z)));
  }

  #[test]
  fn test_phase_flip_for_bit_set_to_0() {
    let zero = Default::default();
    let whole = Complex::new(1.0, 0.0);
    let mut v = StateVector::from_bases(vec!(whole, zero));
    v = z(0, v);
    assert_approx_eq(&v, &StateVector::from_bases(vec!(whole, zero)));
  }

  #[test]
  fn test_phase_flip_for_bit_set_to_1() {
    let zero = Default::default();
    let whole = Complex::new(1.0, 0.0);
    let mut v = StateVector::from_bases(vec!(whole, zero));
    v = x(0, v);
    v = z(0, v);
    assert_approx_eq(&v, &StateVector::from_bases(vec!(zero, -whole)));
  }

  #[test]
  fn test_hadamard() {
    let zero = Default::default();
    let whole = Complex::new(1.0, 0.0);
    let half = Complex::new(1.0/SQRT_2, 0.0);
    let mut v = StateVector::from_bases(vec!(whole, zero));
    v = h(0, v);
    assert_approx_eq(&v, &StateVector::from_bases(vec!(half, half)));
  }

  #[test]
  fn test_hadamard_on_bit_0() {
    let zero = Default::default();
    let whole = Complex::new(1.0, 0.0);
    let half = Complex::new(1.0/SQRT_2, 0.0);
    let mut v = StateVector::from_bases(vec!(whole, zero, zero, zero));
    v = h(0, v);
    assert_approx_eq(&v, &StateVector::from_bases(vec!(half, half, zero, zero)));
  }

  #[test]
  fn test_hadamard_on_bit_1() {
    let zero = Default::default();
    let whole = Complex::new(1.0, 0.0);
    let half = Complex::new(1.0/SQRT_2, 0.0);
    let mut v = StateVector::from_bases(vec!(whole, zero, zero, zero));
    v = h(1, v);
    assert_approx_eq(&v, &StateVector::from_bases(vec!(half, zero, half, zero)));
  }

  #[test]
  fn test_total_superposition_of_2_bits() {
    let zero = Default::default();
    let whole = Complex::new(1.0, 0.0);
    let quarter = Complex::new(0.5, 0.0);
    let mut v = StateVector::from_bases(vec!(whole, zero, zero, zero));
    v = h(0, v);
    v = h(1, v);
    assert_approx_eq(&v, &StateVector::from_bases(vec!(quarter, quarter, quarter, quarter)));
  }

  #[test]
  fn test_hadamard_is_reversible() {
    let zero = Default::default();
    let whole = Complex::new(1.0, 0.0);
    let mut v = StateVector::from_bases(vec!(whole, zero));
    v = h(0, v);
    v = h(0, v);
    assert_approx_eq(&v, &StateVector::from_bases(vec!(whole, zero)));
  }
}