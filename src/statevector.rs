use std::f64;

use cached::{cached, cached_key, SizedCache};
use float_cmp::ApproxEq;
use num::Float;
use rand::random;

use crate::complex::{self, Complex, ComplexMargin};

#[derive(Debug, Clone, PartialEq)]
pub struct StateVector {
    pub bases: Vec<Complex>,
    pub bit_width: usize,
}

#[derive(Debug, PartialEq)]
struct Measurement<'a> {
    bases: &'a mut Vec<Complex>,
    chances: [f64; 2],
    target: usize,
}

impl<'a> Measurement<'a> {
    pub fn new(bases: &'a mut Vec<Complex>, target: usize) -> Self {
        let mut chance_universe_0 = 0.0;
        for (index, amplitude) in bases.iter().enumerate() {
            if check_bit(index, target) == 0 {
                chance_universe_0 += amplitude.norm_sqr();
            }
        }
        let chances = [chance_universe_0, 1.0 - chance_universe_0];
        Measurement {
            bases,
            chances,
            target,
        }
    }

    pub fn collapse(&mut self, fate: f64) -> bool {
        assert!(
            0.0 <= fate && fate < 1.0,
            "Fate must be a f64 value in [0.0, 1.0)"
        );
        let value = (fate >= self.chances[0]) as usize;
        let normalization_factor = self.chances[value].sqrt();
        for index in 0..self.bases.len() {
            if check_bit(index, self.target) == value {
                self.bases[index] /= normalization_factor;
            } else {
                self.bases[index] = Complex::from(0.0);
            }
        }
        value != 0
    }
}

impl StateVector {
    pub fn new(bit_width: usize) -> Self {
        let bases = vec![Complex::new(0.0, 0.0); exp2(bit_width)];
        let mut statevector = StateVector { bases, bit_width };
        statevector.reset();
        statevector
    }

    pub fn from_bases(bases: Vec<Complex>) -> Self {
        let bit_width = (bases.len() as f64).log2() as usize;
        StateVector { bases, bit_width }
    }

    pub fn len(&self) -> usize {
        self.bases.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bases.is_empty()
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

    pub fn measure(&mut self, target: usize) -> bool {
        let mut measurement = Measurement::new(&mut self.bases, target);
        measurement.collapse(random::<f64>())
    }

    pub fn probabilities(&self) -> Vec<f64> {
        self.bases.iter().map(|c| c.norm_sqr()).collect()
    }

    pub fn reset(&mut self) {
        for amplitude in self.bases.iter_mut() {
            amplitude.re = 0.0;
            amplitude.im = 0.0;
        }
        self.bases[0].re = 1.0;
    }
}

impl<'a> ApproxEq for &'a StateVector {
    type Margin = ComplexMargin;

    fn approx_eq<T: Into<Self::Margin>>(self, other: Self, margin: T) -> bool {
        let margin = margin.into();
        for (c1, c2) in self.bases.iter().zip(&other.bases) {
            if c1.re.approx_ne(c2.re, margin) || c1.im.approx_ne(c2.im, margin) {
                return false;
            }
        }
        true
    }
}

pub fn assert_approx_eq(v1: &StateVector, v2: &StateVector) {
    if !v1.approx_eq(v2, (complex::EPSILON, 0)) {
        assert!(
            false,
            "assertion failed `(left ~= right)`\n  left: `{:?}`\n right: `{:?}`",
            v1, v2
        );
    }
}

#[inline]
fn check_bit(value: usize, index: usize) -> usize {
    (value & (1 << index)) >> index
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
                } else if i == c {
                    histogram_index_10 += exp2(c);
                    histogram_index_11 += exp2(c);
                } else {
                    let bit = ((n & mask) != 0) as usize;
                    histogram_index_10 += bit * exp2(i);
                    histogram_index_11 += bit * exp2(i);
                    mask <<= 1;
                }
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
                } else {
                    let bit = ((n & mask) != 0) as usize;
                    histogram_index_0 += bit * exp2(i);
                    histogram_index_1 += bit * exp2(i);
                    mask <<= 1;
                }
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
    Key = {(
        Float::integer_decode(theta),
        Float::integer_decode(phi),
        Float::integer_decode(lambda)
    )};
    fn build_u(theta: f64, phi: f64, lambda: f64) -> UMatrix = {
        (
            Complex::new((theta/2.0).cos(), 0.0),
            -e_power_to(lambda) * (theta/2.0).sin(),
            e_power_to(phi) * (theta/2.0).sin(),
            e_power_to(phi+lambda) * (theta/2.0).cos()
        )
    }
}

#[inline]
fn e_power_to(x: f64) -> Complex {
    Complex::new(0.0, x).exp()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_1_SQRT_2, PI};

    use float_cmp::approx_eq;

    #[test]
    fn test_cnot_c0t1() {
        let p = Default::default();
        let a = Complex::new(1.0, 0.0);
        let b = Complex::new(0.0, 1.0);
        let mut v = StateVector::from_bases(vec![p, a, p, b]);
        v.cnot(0, 1);
        assert_eq!(v, StateVector::from_bases(vec!(p, b, p, a)));
    }

    #[test]
    fn test_cnot_c1t0_of_2_bits() {
        let p = Default::default();
        let a = Complex::new(1.0, 0.0);
        let b = Complex::new(0.0, 1.0);
        let mut v = StateVector::from_bases(vec![p, p, a, b]);
        v.cnot(1, 0);
        assert_eq!(v, StateVector::from_bases(vec!(p, p, b, a)));
    }

    #[test]
    fn test_cnot_c2t0_of_3_bits() {
        let p = Default::default();
        let a = Complex::new(1.0, 0.0);
        let b = Complex::new(0.0, 1.0);
        let mut v = StateVector::from_bases(vec![p, p, p, p, a, b, a, b]);
        v.cnot(2, 0);
        assert_eq!(v, StateVector::from_bases(vec!(p, p, p, p, b, a, b, a)));
    }

    #[test]
    fn test_cnot_c0t2_of_3_bits() {
        let p = Default::default();
        let a = Complex::new(1.0, 0.0);
        let b = Complex::new(0.0, 1.0);
        let mut v = StateVector::from_bases(vec![p, a, p, a, p, b, p, b]);
        v.cnot(0, 2);
        assert_eq!(v, StateVector::from_bases(vec!(p, b, p, b, p, a, p, a)));
    }

    #[test]
    fn test_cnot_is_reversible() {
        let p = Default::default();
        let a = Complex::new(1.0, 0.0);
        let b = Complex::new(0.0, 1.0);
        let mut v = StateVector::from_bases(vec![p, a, p, b]);
        v.cnot(0, 1);
        v.cnot(0, 1);
        assert_eq!(v, StateVector::from_bases(vec!(p, a, p, b)));
    }

    #[test]
    fn test_measurement() {
        let size = 1000;
        let mut accum = 0;
        for _ in 0..size {
            let mut v = StateVector::from_bases(vec![
                Complex::from(FRAC_1_SQRT_2),
                Complex::from(FRAC_1_SQRT_2),
            ]);
            v.u(PI / 2.0, 0.0, PI, 0);
            accum += if v.measure(0) { 1 } else { 0 };
        }
        approx_eq!(
            f64,
            (accum as f64) / (size as f64),
            0.5,
            epsilon = std::f64::EPSILON
        );
    }

    #[test]
    fn test_state_vector_measurement_superposition() {
        let mut v = StateVector::from_bases(vec![
            Complex::from(FRAC_1_SQRT_2),
            Complex::from(FRAC_1_SQRT_2),
        ]);
        let mut measurement = Measurement::new(&mut v.bases, 0);
        let faked_random_value = 0.0;
        measurement.collapse(faked_random_value);
        assert_approx_eq(
            &v,
            &StateVector::from_bases(vec![Complex::from(1.0), Complex::from(0.0)]),
        );
    }

    #[test]
    fn test_state_vector_measurement_0() {
        let mut v = StateVector::from_bases(vec![Complex::from(1.0), Complex::from(0.0)]);
        let mut measurement = Measurement::new(&mut v.bases, 0);
        let faked_random_value = 0.0;
        measurement.collapse(faked_random_value);
        assert_approx_eq(
            &v,
            &StateVector::from_bases(vec![Complex::from(1.0), Complex::from(0.0)]),
        );
    }

    #[test]
    fn test_state_vector_measurement_1() {
        let mut v = StateVector::from_bases(vec![Complex::from(0.0), Complex::from(1.0)]);
        let mut measurement = Measurement::new(&mut v.bases, 0);
        let faked_random_value = 0.0;
        measurement.collapse(faked_random_value);
        assert_approx_eq(
            &v,
            &StateVector::from_bases(vec![Complex::from(0.0), Complex::from(1.0)]),
        );
    }

    #[test]
    fn test_state_vector_measurement_2_qubit_superposition() {
        let mut v = StateVector::from_bases(vec![
            Complex::from(0.5),
            Complex::from(0.5),
            Complex::from(0.5),
            Complex::from(0.5),
        ]);
        let mut measurement = Measurement::new(&mut v.bases, 0);
        let faked_random_value = 0.0;
        measurement.collapse(faked_random_value);
        assert_approx_eq(
            &v,
            &StateVector::from_bases(vec![
                Complex::from(FRAC_1_SQRT_2),
                Complex::from(0.0),
                Complex::from(FRAC_1_SQRT_2),
                Complex::from(0.0),
            ]),
        );
    }
}
