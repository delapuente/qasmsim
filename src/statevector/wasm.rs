use statevector::StateVector;

pub fn as_float_array(statevector: &StateVector) -> Vec<f64> {
  statevector.bases.iter().flat_map(|a| vec!(a.re, a.im)).collect()
}