mod grammar;

use grammar::open_qasm2::open_qasm2;

fn main() {
}

#[test]
fn test_parse_open_qasm() {
  let source = "
OPENQASM 2.0;
qreg q[2];
creg c[2];
";
  let parser = open_qasm2::OpenQasmProgramParser::new();
  assert!(parser.parse(source).is_ok());
}

mod gates {
  use std::{f64};
  use std::ops::{Mul, Add, Neg};

  #[derive(Default, Debug, Clone, Copy)]
  pub struct Cx(f64, f64);

  impl PartialEq for Cx {
    fn eq(&self, other: &Cx) -> bool {
      let epsilon = 1e-10_f64;
      (self.0 - other.0).abs() < epsilon &&
      (self.1 - other.1).abs() < epsilon
    }
  }

  impl Mul for Cx {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
      let real = self.0 * rhs.0 - self.1 * rhs.1;
      let imaginary = self.0 * rhs.1 + self.1 * rhs.0;
      Cx(real, imaginary)
    }
  }

  impl <T> Mul<T> for Cx where T: Into<f64> + Copy {
    type Output = Self;

    fn mul(self, rhs: T) -> Self {
      self * Cx(rhs.into(), 0.0)
    }
  }

  impl Neg for Cx {
    type Output = Self;

    fn neg(self) -> Self {
      Cx(-self.0, -self.1)
    }
  }

  impl Add for Cx {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
      let real = self.0 + rhs.0;
      let imaginary = self.1 + rhs.1;
      Cx(real, imaginary)
    }
  }

  impl <T> Add<T> for Cx where T: Into<f64> + Copy {
    type Output = Self;

    fn add(self, rhs: T) -> Cx {
      self + Cx(rhs.into(), 0.0)
    }
  }

  pub type StateVector = Vec<Cx>;

  /// Apply a controlled not
  pub fn cnot(c: usize, t: usize, mut v: StateVector) -> StateVector {
    let bit_width = (v.len() as f64).log2() as usize;
    let exchangable_rows = find_exchangeable_rows(bit_width, c, t);
    for (index_a, index_b) in exchangable_rows {
      v.swap(index_a, index_b);
    }
    v
  }

  /// Apply a 3 degree rotation to the target bit.
  pub fn u(theta: f64, phi: f64, lambda: f64, t: usize, mut v: StateVector)
  -> StateVector {
    let bit_width = (v.len() as f64).log2() as usize;
    let target_rows = find_target_rows(bit_width, t);
    let u_matrix = build_u(theta, phi, lambda);
    for (index_0, index_1) in target_rows {
      let result_0 = u_matrix.0 * v[index_0] + u_matrix.1 * v[index_1];
      let result_1 = u_matrix.2 * v[index_0] + u_matrix.3 * v[index_1];
      v[index_0] = result_0;
      v[index_1] = result_1;
    }
    v
  }

  fn find_exchangeable_rows(bit_width: usize, c: usize, t: usize)
  -> Vec<(usize, usize)>
  {
    let max = 2_usize.pow(bit_width as u32 - 2);
    let mut out = Vec::with_capacity(max);
    for n in 0..max {
      let mut mask = 1;
      let mut histogram_index_10 = 0;
      let mut histogram_index_11 = 0;
      for i in 0..bit_width {
        if i == t {
          histogram_index_11 += 2_usize.pow(t as u32);
        }
        else if i == c {
          histogram_index_10 += 2_usize.pow(c as u32);
          histogram_index_11 += 2_usize.pow(c as u32);
        }
        else {
          let bit = ((n & mask) != 0) as usize;
          histogram_index_10 += bit * 2_usize.pow(i as u32);
          histogram_index_11 += bit * 2_usize.pow(i as u32);
          mask <<= mask;
        };
      }
      out.push((histogram_index_10, histogram_index_11))
    }
    out
  }

  fn find_target_rows(bit_width: usize, t: usize) -> Vec<(usize, usize)> {
    let max = 2_usize.pow(bit_width as u32 - 1);
    let mut out = Vec::with_capacity(max);
    for n in 0..max {
      let mut mask = 1;
      let mut histogram_index_0 = 0;
      let mut histogram_index_1 = 0;
      for i in 0..bit_width {
        if i == t {
          histogram_index_1 += 2_usize.pow(t as u32);
        }
        else {
          let bit = ((n & mask) != 0) as usize;
          histogram_index_0 += bit * 2_usize.pow(i as u32);
          histogram_index_1 += bit * 2_usize.pow(i as u32);
          mask <<= mask;
        };
      }
      out.push((histogram_index_0, histogram_index_1))
    }
    out
  }

  fn build_u(theta: f64, phi: f64, lambda: f64) -> (Cx, Cx, Cx, Cx) {
    (
      Cx((theta/2.0).cos(), 0.0),
      -e_power_to(lambda) * (theta/2.0).sin(),
      e_power_to(phi) * (theta/2.0).sin(),
      e_power_to(phi+lambda) * (theta/2.0).cos()
    )
  }

  fn e_power_to(x: f64) -> Cx {
    Cx(x.cos(), x.sin())
  }

  mod qelib1 {
    use super::*;
    use std::f64::consts::PI;

    pub fn u2(phi: f64, lambda: f64, t: usize, v: StateVector) -> StateVector {
      u(PI/2.0, phi, lambda, t, v)
    }

    pub fn h(t: usize, v: StateVector) -> StateVector {
      u2(0.0, PI, t, v)
    }

    pub fn x(t: usize, v: StateVector) -> StateVector {
      u(PI, 0.0, PI, t, v)
    }

    pub fn z(t: usize, v: StateVector) -> StateVector {
      u(0.0, 0.0, PI, t, v)
    }

    #[cfg(test)]
    mod tests {
      use super::*;

      #[test]
      fn test_bit_flip() {
        let zero = Default::default();
        let whole = Cx(1.0, 0.0);
        let mut v = vec!(whole, zero);
        v = x(0, v);
        assert_eq!(v, vec!(zero, whole));
      }

      #[test]
      fn test_bit_flip_is_reversible() {
        let zero = Default::default();
        let whole = Cx(1.0, 0.0);
        let mut v = vec!(whole, zero);
        v = x(0, v);
        v = x(0, v);
        assert_eq!(v, vec!(whole, zero));
      }

      #[test]
      fn test_bit_flip_x1_on_3_bits() {
        let z = Default::default();
        let w = Cx(1.0, 0.0);
        let mut v = vec!(w, z, z, z, z, z, z, z);
        v = x(1, v);
        assert_eq!(v, vec!(z, z, w, z, z, z, z, z));
      }

      #[test]
      fn test_phase_flip_for_bit_set_to_0() {
        let zero = Default::default();
        let whole = Cx(1.0, 0.0);
        let mut v = vec!(whole, zero);
        v = z(0, v);
        assert_eq!(v, vec!(whole, zero));
      }

      #[test]
      fn test_phase_flip_for_bit_set_to_1() {
        let zero = Default::default();
        let whole = Cx(1.0, 0.0);
        let mut v = vec!(whole, zero);
        v = x(0, v);
        v = z(0, v);
        assert_eq!(v, vec!(zero, -whole));
      }

      #[test]
      fn test_hadamard() {
        let zero = Default::default();
        let whole = Cx(1.0, 0.0);
        let half = Cx(1.0/f64::consts::SQRT_2, 0.0);
        let mut v = vec!(whole, zero);
        v = h(0, v);
        assert_eq!(v, vec!(half, half));
      }

      #[test]
      fn test_hadamard_on_bit_0() {
        let zero = Default::default();
        let whole = Cx(1.0, 0.0);
        let half = Cx(1.0/f64::consts::SQRT_2, 0.0);
        let mut v = vec!(whole, zero, zero, zero);
        v = h(0, v);
        assert_eq!(v, vec!(half, half, zero, zero));
      }

      #[test]
      fn test_hadamard_on_bit_1() {
        let zero = Default::default();
        let whole = Cx(1.0, 0.0);
        let half = Cx(1.0/f64::consts::SQRT_2, 0.0);
        let mut v = vec!(whole, zero, zero, zero);
        v = h(1, v);
        assert_eq!(v, vec!(half, zero, half, zero));
      }

      #[test]
      fn test_total_superposition_of_2_bits() {
        let zero = Default::default();
        let whole = Cx(1.0, 0.0);
        let quarter = Cx(0.5, 0.0);
        let mut v = vec!(whole, zero, zero, zero);
        v = h(0, v);
        v = h(1, v);
        assert_eq!(v, vec!(quarter, quarter, quarter, quarter));
      }

      #[test]
      fn test_hadamard_is_reversible() {
        let zero = Default::default();
        let whole = Cx(1.0, 0.0);
        let mut v = vec!(whole, zero);
        v = h(0, v);
        v = h(0, v);
        assert_eq!(v, vec!(whole, zero));
      }
    }

  }

  #[cfg(test)]
  mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_cnot_c0t1() {
      let p = Default::default();
      let a = Cx(1.0, 0.0);
      let b = Cx(0.0, 1.0);
      let mut v = vec!(p, a, p, b);
      v = cnot(0, 1, v);
      assert_eq!(v, vec!(p, b, p, a));
    }

    #[test]
    fn test_cnot_c1t0_of_2_bits() {
      let p = Default::default();
      let a = Cx(1.0, 0.0);
      let b = Cx(0.0, 1.0);
      let mut v = vec!(p, p, a, b);
      v = cnot(1, 0, v);
      assert_eq!(v, vec!(p, p, b, a));
    }

    #[test]
    fn test_cnot_c2t0_of_3_bits() {
      let p = Default::default();
      let a = Cx(1.0, 0.0);
      let b = Cx(0.0, 1.0);
      let mut v = vec!(p, p, p, p, a, b, a, b);
      v = cnot(2, 0, v);
      assert_eq!(v, vec!(p, p, p, p, b, a, b, a));
    }

    #[test]
    fn test_cnot_c0t2_of_3_bits() {
      let p = Default::default();
      let a = Cx(1.0, 0.0);
      let b = Cx(0.0, 1.0);
      let mut v = vec!(p, a, p, a, p, b, p, b);
      v = cnot(0, 2, v);
      assert_eq!(v, vec!(p, b, p, b, p, a, p, a));
    }

    #[test]
    fn test_cnot_is_reversible() {
      let p = Default::default();
      let a = Cx(1.0, 0.0);
      let b = Cx(0.0, 1.0);
      let mut v = vec!(p, a, p, b);
      v = cnot(0, 1, v);
      v = cnot(0, 1, v);
      assert_eq!(v, vec!(p, a, p, b));
    }

    #[test]
    fn test_e_power_to() {
      assert_eq!(e_power_to(0.0), Cx(1.0, 0.0));
      assert_eq!(e_power_to(PI/2.0), Cx(0.0, 1.0));
      assert_eq!(e_power_to(PI/4.0), Cx((PI/4.0).cos(), (PI/4.0).sin()));
    }

    #[test]
    fn test_euler_identity() {
      assert_eq!(e_power_to(PI) + 1, Default::default())
    }

  }
}