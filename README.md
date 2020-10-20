![RustCI](https://github.com/Lutetium-Vanadium/anilang/workflows/Rust/badge.svg)

# anilang

`anilang` is a dynamically type language currently under construction.

## Installation

You will need to install the [`cargo`](https://www.rust-lang.org/learn/get-started) to compile the both the `repl` and the `core`.

To start the `repl`, run
```sh
cargo run
```

You can alternatively build the project, and the binary will be
available in `target/release/anilang`.
```sh
cargo build --release &&
./target/release/anilang
```

To install the binary using `cargo`, run
```sh
cargo install
```

Run tests:
```sh
cargo test -p anilang -p anilang-core
```

## Usage

The current syntax is subject to change.

To declare a variable:
```rust
let a = <val>
```

Variables can be reassigned to any other value
```rust
a = <val>
```

Basic arithmetic and boolean operators exist:
| operator | purpose                           |
| -------- | --------------------------------- |
| `+`      | Addition and String concatenation |
| `-`      | Subtraction                       |
| `*`      | Multiplication                    |
| `/`      | Division                          |
| `%`      | Modulo                            |
| `^`      | Power                             |
| `++`     | Add 1                             |
| `--`     | Subtract 1                        |
| `\|\|`   | Boolean Or                        |
| `&&`     | Boolean And                       |
| `==`     | Equality                          |
| `!=`     | Not equal                         |
| `>`      | Greater than                      |
| `>=`     | Greater than equal to             |
| `<`      | Less than                         |
| `<=`     | Less than equal to                |

There are also conditionals
```rust
if <cond> {
    ...
} else if <cond> {
    ...
} else {
    ...
}
```

Currently there are 2 kind of loops, `loop` provides and infinite loop.
```rust
loop {
    ...
}

while <cond> {
    ...
}
```

Functions can be declared in the following way:
```rust
fn <func_name>(<args>...) {
    ...
}
```
