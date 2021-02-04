# Expressions

This file contains the various expressions that exist, and explanations
about what they are and their format.

## Index
- [Binary expressions](#binary-expressions)
- [Unary expressions](#unary-expressions)
- [Literals](#literals)
- [Variable accesses](#variable-accesses)
- [Blocks](#blocks)
- [Parenthesised statements](#parenthesised-statements)
- [Function calls](#function-calls)
- [Index/property accesses](#indexproperty-accesses)

## Binary Expressions

A binary expression is a combination of 2 expressions to give a single
expression. It is in the following format:
```rust
<expr> <op> <expr>
```

The operators are of the following types:
| operator | purpose                                    | type       |
| -------- | ------------------------------------------ | ---------- |
| `+`      | Addition and String and List concatenation | arithmetic |
| `-`      | Subtraction                                | arithmetic |
| `*`      | Multiplication                             | arithmetic |
| `/`      | Division                                   | arithmetic |
| `%`      | Modulo                                     | arithmetic |
| `^`      | Power                                      | arithmetic |
| `\|\|`   | Boolean Or                                 | boolean    |
| `&&`     | Boolean And                                | boolean    |
| `==`     | Equality                                   | boolean    |
| `!=`     | Not equal                                  | boolean    |
| `>`      | Greater than                               | boolean    |
| `>=`     | Greater than equal to                      | boolean    |
| `<`      | Less than                                  | boolean    |
| `<=`     | Less than equal to                         | boolean    |

A binary expression returns the result of the operation.

> It follows regular operator precedence rules.
> See [https://en.wikipedia.org/wiki/Order_of_operations](https://en.wikipedia.org/wiki/Order_of_operations)

Arithmetic binary operators only work between the same types, but an int
can be casted to a float, so operations between ints and floats exist.

Boolean operators work on whether the value is truthy or falsy.

## Unary Expressions

A unary expressions performs some operation on a expression. It is in
the following format.
```rust
<op> <expr>
```

The operators are of the following types:
| operator | purpose        | type       |
| -------- | -------------- | ---------- |
| `+`      | Unary plus     | arithmetic |
| `-`      | Unary negative | arithmetic |
| `!`      | Boolean not    | boolean    |

A unary expression returns the result of the operation.

> It follows regular operator precedence rules.
> See [https://en.wikipedia.org/wiki/Order_of_operations](https://en.wikipedia.org/wiki/Order_of_operations)

## Literals

A literal is a declaration of any of the base [values](./values.md).
Refer to specific value docs for how to declare each one.


## Variable accesses

It returns the value of the given variable in the scope hierarchy. If it
can't find the value in the current scope, it checks the current scope's
parent scope, and so on until the global scope after which it will give
an error.

## Blocks

Refer to the part about blocks [here](./syntax.md#blocks-and-scoping)

## Parenthesised statements

This converts a statement to an expression. It is in following format:
```rust
(<stmt>)
```

This returns the value that the statement it encloses returns.

## Function calls

Function calls are of the following format:
```rust
<expr>(<stmt>, ...)
```
The `<expr>` represents the function object to be called, and the comma
separated statements in the parenthesis represents the arguments
supplied to the function.

Since functions are objects (see [here](./values.md#functions)), the
`<expr>` need not be an ident, but can be anything that results in a 
function.

```rust
(fn(a, b) {
    print(a + b == 3)
})(1, 2)
```
See [anonymous functions](./syntax.md#anonymous-functions).

## Index/property accesses

Values can be indexed in the following way:
```rust
// Index
<expr>[<stmt>]

// Property
<expr>.<ident>
```

Note property access is syntactic sugar for indexing through strings.
```rust
<expr>.<ident> == <expr>["<ident>"]
```

For more information on the indexing capabilities of different types,
refer [here](./values.md)
