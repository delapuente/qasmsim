# qasmsim
> A QASM interpreter and quantum simulator in Rust.

## Prerequisites

Make sure you have installed [`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html).
For compiling the WASM version, make sure you have [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/)
also installed.

## What is missing?

The interpreter [golden path](https://en.wikipedia.org/wiki/Happy_path) is almost complete, although it still remains to be done...

 - [ ] Add trigonometric and exponential functions for the real expressions.
 - [ ] Test all the ugly, error paths.
 - [ ] Semantic comments for documenting the gates.

Although there is still no support for including external gate definitions,
[including `qelib1.inc`](https://github.com/Qiskit/openqasm/blob/master/examples/generic/qelib1.inc) via the `include` instruction will work:

```
include "qelib1.inc";
```

A sample QASM program can be found here:

```qasm
OPENQASM 2.0;
include "qelib1.inc";
qreg q[2];
h q[0];
cx q[0], q[1];
```

The complete specification can be found under the [QASM repository](https://github.com/Qiskit/openqasm/blob/master/spec-human/).

## Building from sources

Current version of qasmsim is very limited. You can test it by doing:

```sh
$ cargo run < samples/bell.qasm
```

And it will print the internal statevector and the time invested in calculating it. Nothing impressive so far.

## Testing the project

You can refer to unit tests (in the files under the `src` folder) and integration tests (under the `tests` folder) to figure out what is implemented. For passing the tests of the project you can do:

```sh
$ cargo test
```

## WASM version

`qasmsim` can be used in the web if you compile it for Web Assembly. Doing it is easy, simply ensure you have `wasm-pack` installed and run:

```sh
$ wasm-pack build -- --features "wasm"
```

It will compile your project and pack it inside the `pkg` folder. Now enter the `www` directory and run:

```sh
# This command is only needed the first time you want to launch the web version
$ npm install
$ npm start
```

Browse the URL provided by the second command and you'll see a blank page. Go to the developer tools of your browser and try running a small test:

```js
var result = qasmsim.run(`
OPENQASM 2.0;
include "qelib1.inc";
qreg q[2];
h q[0];
cx q[0], q[1];
`);
var statevector = result.get_statevector(); // histogram
var memory = result.get_memory(); // classical memory with name and value
```

The answer is a flat [`Float64Array`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Typed_arrays) representing the pairs of complex numbers that form the statevector, and the list of classical memory locations and current values.