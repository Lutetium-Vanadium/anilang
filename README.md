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
| operator | purpose                                    |
| -------- | ------------------------------------------ |
| `+`      | Addition and String and List concatenation |
| `-`      | Subtraction                                |
| `*`      | Multiplication                             |
| `/`      | Division                                   |
| `%`      | Modulo                                     |
| `^`      | Power                                      |
| `\|\|`   | Boolean Or                                 |
| `&&`     | Boolean And                                |
| `==`     | Equality                                   |
| `!=`     | Not equal                                  |
| `>`      | Greater than                               |
| `>=`     | Greater than equal to                      |
| `<`      | Less than                                  |
| `<=`     | Less than equal to                         |

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

Currently there are 2 kind of loops, `loop` provides an infinite loop.
```rust
loop {
    ...
}

while <cond> {
    ...
}
```

Functions can be declared in the following ways:
```rust
// Regular function declaration, gets stored in <func_name>
fn <func_name>(<args>...) {
    ...
}

// Anonymous function which is immediately invoked, but it can be used
// like any other value
(fn(a, b) { a + b })(1, 2)
```

Strings can be indexed using `[]`
```rust
"string"[1]
variable[2]
```
They can also be assigned to
```rust
let variable = "----"
variable[2] = "a"  // variable is "--a-"
variable[1] = "ab" // variable is "-aba-"
```
Strings larger than 1 character will remove the character at that index
and insert the characters given

There are also comments -
- Single line: `// comment`
- Multi line: `/* comment */`
