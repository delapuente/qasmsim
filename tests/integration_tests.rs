extern crate qasmsim;

use std::f64::consts::FRAC_1_SQRT_2;
use qasmsim::statevector::{ StateVector, assert_approx_eq};
use qasmsim::complex::Complex;

#[test]
fn endianess() {
    let source = "
    OPENQASM 2.0;
    qreg q[1];
    qreg r[1];
    h r[0];
    ";
    assert_approx_eq(
        &qasmsim::run(source).unwrap(),
        &StateVector::from_bases(vec!(
            Complex::from(FRAC_1_SQRT_2),
            Complex::from(0.0),
            Complex::from(FRAC_1_SQRT_2),
            Complex::from(0.0)
        ))
    )
}

#[test]
fn call_custom_gate() {
    let source = "
    OPENQASM 2.0;
    gate h q {
        U(pi/2, 0, pi);
    }
    qreg q[1];
    h q[0];
    ";
    assert_approx_eq(
        &qasmsim::run(source).unwrap(),
        &StateVector::from_bases(vec!(
            Complex::from(FRAC_1_SQRT_2),
            Complex::from(FRAC_1_SQRT_2)
        ))
    )
}

#[test]
fn test_one_register_bell_circuit() {
    let source = "
    OPENQASM 2.0;
    qreg q[2];
    h q[0];
    cx q[0], q[1];
    ";
    assert_approx_eq(
        &qasmsim::run(source).unwrap(),
        &StateVector::from_bases(vec!(
            Complex::from(FRAC_1_SQRT_2),
            Complex::from(0.0),
            Complex::from(0.0),
            Complex::from(FRAC_1_SQRT_2)
        ))
    )
}

#[test]
fn test_two_registers_bell_circuit() {
    let source = "
    OPENQASM 2.0;
    qreg q[1];
    qreg r[1];
    h q[0];
    cx q[0], r[0];
    ";
    assert_approx_eq(
        &qasmsim::run(source).unwrap(),
        &StateVector::from_bases(vec!(
            Complex::from(FRAC_1_SQRT_2),
            Complex::from(0.0),
            Complex::from(0.0),
            Complex::from(FRAC_1_SQRT_2)
        ))
    )
}

#[test]
fn test_no_indices_bell_circuit() {
    let source = "
    OPENQASM 2.0;
    qreg q[1];
    qreg r[1];
    h q;
    cx q, r;
    ";
    assert_approx_eq(
        &qasmsim::run(source).unwrap(),
        &StateVector::from_bases(vec!(
            Complex::from(FRAC_1_SQRT_2),
            Complex::from(0.0),
            Complex::from(0.0),
            Complex::from(FRAC_1_SQRT_2)
        ))
    )
}

#[test]
fn test_no_indices_superposition() {
    let source = "
    OPENQASM 2.0;
    qreg q[4];
    h q;
    ";
    assert_approx_eq(
        &qasmsim::run(source).unwrap(),
        &StateVector::from_bases(vec!(Complex::from(0.25); 16))
    )
}