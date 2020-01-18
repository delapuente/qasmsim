OPENQASM 2.0;
qreg q[2];
gate h q {
  U (pi/2, 0, pi) q;
}
h q[0];
CX q[0], q[1];
