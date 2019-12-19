use std::ops::{Mul, Add, Neg};

#[derive(Default, Debug, Clone, Copy)]
pub struct Complex(pub f64, pub f64);

impl PartialEq for Complex {
  fn eq(&self, other: &Complex) -> bool {
    let epsilon = 1e-10_f64;
    (self.0 - other.0).abs() < epsilon &&
    (self.1 - other.1).abs() < epsilon
  }
}

impl Mul for Complex {
  type Output = Self;

  fn mul(self, rhs: Self) -> Self {
    let real = self.0 * rhs.0 - self.1 * rhs.1;
    let imaginary = self.0 * rhs.1 + self.1 * rhs.0;
    Complex(real, imaginary)
  }
}

impl <T> Mul<T> for Complex where T: Into<f64> + Copy {
  type Output = Self;

  fn mul(self, rhs: T) -> Self {
    self * Complex(rhs.into(), 0.0)
  }
}

impl Neg for Complex {
  type Output = Self;

  fn neg(self) -> Self {
    Complex(-self.0, -self.1)
  }
}

impl Add for Complex {
  type Output = Self;

  fn add(self, rhs: Self) -> Self {
    let real = self.0 + rhs.0;
    let imaginary = self.1 + rhs.1;
    Complex(real, imaginary)
  }
}

impl <T> Add<T> for Complex where T: Into<f64> + Copy {
  type Output = Self;

  fn add(self, rhs: T) -> Complex {
    self + Complex(rhs.into(), 0.0)
  }
}
