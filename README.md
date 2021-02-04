![RustCI](https://github.com/Lutetium-Vanadium/anilang/workflows/Rust/badge.svg)

# anilang

`anilang` is a dynamically type language currently under construction.

## Installation

You will need to install the [`cargo`](https://www.rust-lang.org/learn/get-started) to compile the both the `repl` and the `core`.

To start the `repl`, run
```sh
cargo run -- --help
```

You can alternatively build the project, and the binary will be
available in `target/release/anilang`.
```sh
cargo build --release
./target/release/anilang --help
```

To install the binary using `cargo`, run
```sh
cargo install --path .
```

Run tests:
```sh
cargo test -p anilang -p anilang-core
```

## Usage

The documentation of the syntax of the language can be found
[here](./docs/syntax.md).

The current syntax is subject to change.
