const worker = new Worker(new URL('./worker.js', import.meta.url));
worker.onmessage = (evt) => {
  if (evt.data === 'ready') {
    worker.postMessage(`
    OPENQASM 2.0;
    include "qelib1.inc";
    qreg q[2];
    h q[0];
    cx q[0], q[1];
    `);
  }
};