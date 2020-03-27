# qasmsim
> A QASM interpreter and quantum simulator in Rust.

## Prerequisites

Make sure you have installed [`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html).
For compiling the WASM version, make sure you have [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/)
also installed.

## What is missing?

The interpreter [golden path](https://en.wikipedia.org/wiki/Happy_path) is almost complete, although the following is still lacking before version `1.0.0`:

 - [X] Create CLI interface.
   - [ ] Better formatting of results.
   - [ ] Add --shots option + histogram output.
 - [ ] Add trigonometric and exponential functions in real expressions.
 - [ ] Add a semantic checker for checking the correctness of the program before runtime.
 - [X] Handle error paths.
   - [ ] Improve error hierarchy.
   - [ ] Add source references to errors.
 - [ ] Allow including external source.
   - [ ] In the native lib.
   - [ ] In the WASM version.
 - [ ] Semantic comments for documenting the gates.

Although there is still no support for including external gate definitions,
[including `qelib1.inc`](https://github.com/Qiskit/openqasm/blob/master/examples/generic/qelib1.inc) via the `include` instruction will work, for the `qelib1.inc` lib is embedded in the simulator:

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

## qasmsim CLI

Install `qasmsim` with:

```sh
$ cargo install --git https://github.com/delapuente/qasmsim
```

And simulate a QASM program with:

```sh
$ qasmsim source.qasm
```

See more options with:

```
$ qasmsim --help
qasmsim 0.1.0
A QASM interpreter and quantum simulator in Rust.

USAGE:
    qasmsim [FLAGS] [OPTIONS] [source]

FLAGS:
    -h, --help             Prints help information
        --probabilities    Prints the probabilities vector of the simulation
        --statevector      Prints the state vector of the simulation
    -t, --times            Prints times measured for parsing and simulating
    -V, --version          Prints version information

OPTIONS:
        --output <output>    Output file, stdout if not present

ARGS:
    <source>    QASM program file, read from stdin if not present
```

## Testing the project

You can refer to unit tests (in the files under the `src` folder) and integration tests (under the `tests` folder) to figure out what is implemented. For passing the tests of the project you can do:

```sh
$ cargo test
```

## WASM version

`qasmsim` can be used in the web if you compile it for Web Assembly. Doing it is easy, simply ensure you have `wasm-pack` installed and run:

```sh
$ wasm-pack build
```

It will compile your project and pack it inside the `pkg` folder. Now enter the `www` directory, install the dependencies with (you only need run this once):

```sh
$ npm install
```

And start the web server with:

```sh
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
var statevector = result.statevector.bases; // amplitude histogram; pairs represent complex numbers.
var probabilities = result.probabilities; // vector of probabilities
var memory = result.memory; // JavaScript Map with classical results by name of the registry.
```

The answer is a flat [`Float64Array`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Typed_arrays) representing the pairs of complex numbers that form the statevector, and the list of classical memory locations and current values.