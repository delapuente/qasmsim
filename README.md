# qasmsim
A QASM interpreter and quantum simulator in Rust

## Prerequisites

Make sure you have installed [`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html).
For compiling the WASM version, make sure you have [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/)
also installed.

## What is currently done?

Except for including external gate definition libraries, the interpreter has basic support of the quantum features of QASM. However, the interpreter does not support measurements yet, or conditions on classical registers. Fortunately, this is a simulator and you can do inspect the state of the system ;)

There is still a lot of corner cases that need testing, but basic functionality is there.

A sample QASM program can be found here:

```qasm
OPENQASM 2.0;
qreg q[2];
gate h q {
  U (pi/2, 0, pi) q;
}
h q[0];
CX q[0], q[1];
```

The complete specification can be found under the [QASM repository](https://github.com/Qiskit/openqasm/blob/master/spec-human/).

## Building from sources

Current version of qasmsim is very limited. You can test it by doing:

```sh
$ cargo run < samples/bell.qasm
```

And it will print the internal statevector and the time invested in calculating it. Nothing impressive so far.

## Testing the project

You can refer to unit tests (in the files under the `src` folder) and integration tests (under the `tests` folder)
to figure out what is implemented. For passing the tests of the project you can do:

```sh
$ cargo test
```

## WASM version

`qasmsim` can be used in the web if you compile it for Web Assembly. Doing it is easy, simply ensure you have
`wasm-pack` installed and run:

```sh
$ wasm-pack build -- --features "wasm"
```

It will compile your project and pack it inside the `pkg` folder. Now enter the `www` directory and run:

```sh
# This command is only needed the first time you want to launch the web version
$ npm install
$ npm start
```

Browse the URL provided by the second command and you'll see a blank page. Go to the developer tools of your browser
and try running a small test:

```js
qasmsim.run(`
OPENQASM 2.0;
qreg q[2];
gate h q {
  U (pi/2, 0, pi) q;
}
h q;
CX q[0], q[1];
`);
```

The answer is a flat [`Float64Array`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Typed_arrays) representing
the pairs of complex numbers that form the statevector.
