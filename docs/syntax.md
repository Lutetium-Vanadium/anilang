# Syntax

Anilang currently contains many of the usual language features, but also
provides the ability to compose statements. In anilang, everything is a
statement, and language features are defined on the basis of statements,
and every statement returns a value (which could be null).

However, in some special cases there is a need to define a second base
construct - expression. An expression is a statement, but a statement is
not necessarily an expression. However, a non-expression statement can
be converted to an expression by wrapping it in parenthesis - `(<stmt>)`.

The following notation will be used:
- `<stmt>` - statement
- `<expr>` - expression
- `...` - etc

## Identifiers

Identifiers are nothing but valid variable names. They consist of only
alphabets, numbers and `_`, but cannot start with a number.

For example `0var` is invalid, but `var0` is valid.

In the following explanations, `<ident>` refers to a identifier.

## Blocks and Scoping

Like other languages, there are variables, and a scoping mechanism.
Every scope is tied to a block, and every block has a scope. 

Blocks are a group of statements surrounded by `{}`. Blocks return the
value of the last statement.

```rust
{ <---------- Start of the block
    <stmt>
    ...
    <stmt>
} <---------- End of the block
```

## Variables

Variables can be declared in the following manner.
```rust
let <ident> = <stmt> // Declaration statement
```
The variable is declared in the current scope, and can be reassigned in
the following way.
```rust
<ident> = <stmt> // Assignment statement
```
> Assigning to an index is also possible for certain data types, for more
> information on the indexing capabilities of different types, refer
> [here](./values.md)

The value variables store can be accessed just like any other language
with just its ident.

Both declaration statements and assignment statements return the value
of the variable assigned to it.

## Expressions

[Expressions](./expressions.md) consist of the following types:
- [Binary expressions](./expressions.md#binary-expressions)
- [Unary expressions](./expressions.md#unary-expressions)
- [Literals](./expressions.md#literals)
- [Variable accesses](./expressions.md#variable-accesses)
- [Blocks](#blocks)
- [Parenthesised statements](./expressions.md#parenthesised-statements)
- [Function calls](./expressions.md#function-calls)
- [Index/property accesses](./expressions.md#indexproperty-accesses)

## Control flow

### If conditions

Like other languages, there are if-else clauses. They are in the
following format:
```rust
if <stmt> {
    ...
} else if <stmt> {
    ...
} else {
    ...
}
```

The conditional returns the value of the block executed. So for example
if the first condition is true, the value returned is the value of if
block. In case there is no else block and the condition is false, it
returns it null.

> The if and else blocks are blocks and so create new scopes.

### Loops

Currently there are 2 kind of loops, `loop` provides an infinite loop.
```rust
loop {
    ...
}

while <stmt> {
    ...
}
```
Loops always return null. While in a loop, you can use `break` statements
to exit the loop.

> The loop blocks are blocks and so create new scopes.

## Function declaration

A function declaration creates a function, the return value is a
function object.

Functions by default return the value of the last statement, but early
returns are possible with the `return` keyword. In case no value is
specified after `return`, it returns null.
```rust
fn factorial(n) {
    if n == 2 {
        return n // Exit early and return `n`
    }

    n * factorial(n-1) // If n is not 2, return this
}
```

> See also [function value](./values.md#functions)

### Named functions

Named functions declare a function and store it inside a variable.

```rust
fn <ident>(<ident>, ...) {
    ...
}
```

It is the equivalent of creating a function object and then storing it
in a variable. It also returns the function object.


```rust
let a = fn b() { 123 }
a == b // true
```

### Anonymous functions

These just create the function object.

```rust
fn(<ident>, ...) {
    ...
}
```

```rust
// This creates an anonymous function and assigns it to sum
let sum = fn(a, b) { a + b }

fn sum(a, b) { a + b }
```
Both of the above declarations have the same effect.

## Interfaces

Interfaces can be used to generate objects of the same structure.

They can be declared in the following way:
```typescript
interface <ident> {
    // Regular properties on the object
    <ident> = <stmt>

    // Functions on the object
    fn <ident>(<ident>, ...) {
        ...
    }

    // A special function with the same name as the interface as a
    // constructor
    // 
    // note the `fn` keyword is optional here
    [fn] <ident = interface name>(<ident>, ...) {
    }
}
```

If an interface `I` was declared, the properties and functions declared
on it can be accessed through `I::<ident>`. The constructor can be
called through `I()` itself.

> Interfaces are not real values and is just syntactic sugar for
> declaring properties and functions within a namespace

When the constructor is called, it gets a magic `self` variable, which
is already initialised to an object with the properties and functions*
defined on the interface. The constructor also automatically returns the
generated self variable

\* Only functions with 'self' as the first argument are added

```typescript
interface I {
    // Property v
    v = 123

    // Function get_v has 'self' as first argument so included in the
    // magic self for constructor
    fn get_v(self) {
        self.v
    }

    // Function make doesn't have self as first argument, so not
    // included in the magic self
    // 
    // NOTE: this creates an object with **only** one property `v` and
    // has nothing to do with the properties and functions on the
    // interface
    fn make(v) {
        { v, }
    }

    // Constructor 
    I(v) {
        // --------- magic line ---------
        // self = {
        //     v: 123,
        //     get_v(self) { self.v }
        // }
        // ------------------------------

        self.v = v

        // --------- magic line ---------
        // return self
        // ------------------------------
    }
}
```

> The magic self only is only generated for the constructor and not for
> any other function

> Currently, calling methods on objects doesn't automatically get the
> self arg. This is not supposed to be intended behaviour, and will be
> fixed soon.

## Comments

There are 2 kinds of comments:
- Single line: `// comment`
- Multi line: `/* comment */`

These are ignored and have no effect on the program
