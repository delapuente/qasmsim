# qasmsim
> A QASM interpreter and quantum simulator in Rust.

## Prerequisites

Make sure you have installed [`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html).
For compiling the WASM version, make sure you have [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/)
also installed.

## What is missing?

The interpreter [golden path](https://en.wikipedia.org/wiki/Happy_path) is almost complete, although the following is still lacking before version `1.0.0`:

 - [X] Add trigonometric and exponential functions in real expressions.
 - [X] Create CLI interface.
   - [X] Better formatting of results.
   - [X] Add --shots option + histogram output.
 - [X] Handle error paths.
   - [X] Improve error hierarchy.
   - [X] Add source references to errors.

And planned for `1.1.0` is:

 - [ ] Allow including external source.
   - [ ] In the native lib.
   - [ ] In the WASM version.
 - [ ] Add a semantic checker for checking the correctness of the program before runtime.
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
qasmsim 0.1.0
A QASM interpreter and quantum simulator in Rust.

USAGE:
    qasmsim [FLAGS] [OPTIONS] [source]

FLAGS:
    -b, --binary           Prints the binary representation of the values
    -h, --help             Prints help information
    -x, --hexadecimal      Prints the hexadecimal representation of the values
    -i, --integer          Prints the interger representation of the values. Default option
        --probabilities    Prints the probabilities vector of the simulation. Ignored if shots is set
        --statevector      Prints the state vector of the simulation. Ignored if shots is set
    -t, --times            Prints times measured for parsing and simulating
    -V, --version          Prints version information
    -v                     Verbosity of the output

OPTIONS:
        --out <out>        Output files prefix, print in the stdout if not present. The output format of each file is
                           CSV. At most, three files are created with the names out.memory.csv, out.state.csv and
                           out.times.csv
        --shots <shots>    Specify the number of simulations

ARGS:
    <source>    QASM program file, read from stdin if not present
```

## qasmsim library

`qasmsim` is also a library including a QASM parser which generates a QASM AST,
an interpreter, and a quantum state-vector simulator. The command-line tool is
just one of the multiple consumers the library can have. If you want to install
the library functionality only, remove the `default` features when installing:

```sh
$ cargo install --no-default-features
```

## Testing the project

You can refer to unit tests (in the files under the `src` folder) and integration tests (under the `tests` folder) to figure out what is implemented. For passing the tests of the project you can do:

```sh
$ cargo test
```

## WASM version

`qasmsim` can be used in the web if you compile it for Web Assembly. Doing it is easy, simply download the sources, ensure you have `wasm-pack` installed and run:

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
```

The module is exported by default as the `qasmsim` object in `window` and implments the following interface:

```ts
interface qasmsim {
  run: (input: string, shots?: number) => Run
}

interface Run {
  histogram?: Histogram,
  probabilities: Float64Array,
  statevector: Float64Array,
  memory: Memory,
  times: RunTimes
}

type Memory = { [key: string]: Array[number] }
type Histogram = { [key: string]: Array[[number, number]] }
type RunTimes = {
  parsing_time: number,
  simulation_time: number,
  serialization_time: number
}
```