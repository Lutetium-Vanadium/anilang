# Values

This file contains all the data types, and how to declare and use them.
## Index

- [ints](#ints)
- [floats](#floats)
- [bools](#bools)
- [ranges](#ranges)
- [strings](#strings)
- [lists](#lists)
- [objects](#objects)
- [functions](#functions)
- [null](#null)


## Ints

These are 64 bit integers, ranging from -2<sup>63</sup> to 2<sup>63</sup> - 1.

#### Declaration

```rust
<number>
```

#### Operations

All binary and unary operations are possible on these numbers. Any
non-zero integer is truthy.

#### Indexing

Integers cannot be indexed by any type.

## Floats

These are 64 bit floating point numbers.

#### Declaration

```rust
<number>.<number>

<number>.           <--- equivalent of <number>.0

.<number>           <--- equivalent of 0.<number>
```

#### Operations

All binary and unary operations are possible on these numbers. Any
non-zero float is truthy.

#### Indexing

Floats cannot be indexed by any type.

## Bools

A simple boolean value: `true` or `false`

#### Declaration

```rust
true

false
```

#### Operations

Non arithmetic binary and unary operations are possible on booleans.
Obviously `true` is truthy and `false` is not.

#### Indexing

Booleans cannot be indexed by any type.

## Ranges

A range of 2 integers.

#### Declaration

```rust
<stmt>..<stmt>
```
> Since ranges are only between numbers, the statements must result in
> integers or else it will result in a type error.

#### Operations

Non arithmetic binary and unary operations are possible on ranges. Any
non empty range is truthy, i.e. start != end.

#### Indexing

Ranges have 2 properties present on it.
- `'start'` - The start of the range
- `'end'` - The start of the range

## Strings

Strings are a mutable sequence of characters. Strings support unicode
characters as well.

#### Declaration

```typescript
'...'

"..."

'escape\' with backslash'
```

#### Operations

Non arithmetic binary and unary operations are possible on strings. The
plus operator also acts as a concatenation operator joining 2 strings.
Any non empty string is truthy.

#### Indexing

Strings can be indexed using the following types:
- `int` - Gives the character at the nth index
- `range` - Gives the substring in that range

Strings have 1 property present on it.
- `'len'` - The length of the string

> When indexing with `int`s and `range`s, a negative integer means index
> from the back, similar to python

## Lists

Lists are a mutable sequence of values. They elements of the list do not
need to be of a specific type, but can be any type. Any non empty list
is truthy.

#### Declaration

```rust
[<stmt>, ...]
```

#### Operations

Non arithmetic binary and unary operations are possible on strings. The
plus operator also acts as a concatenation operator joining 2 lists.

#### Indexing

Lists can be indexed using the following types:
- `int` - Gives the character at the nth index
- `range` - Gives the sublist in that range

Lists have 3 properties present on it.
- `'len'` - The length of the list
- `'push'` - An inbuilt function to add a element to the back of the
  list
- `'pop'` - Remove he last element from the list. If the list is empty
  this will error.

> When indexing with `int`s and `range`s, a negative integer means index
> from the back, similar to python

## Objects

Objects are key value pairs where keys are strings and the value can be
any type.

#### Declaration

```rust
{}
^^-- Empty object

{
    <ident>: <stmt>,                                            [1]
    ^^^^^^^^^^^^^^^-- Syntactic sugar for `"<ident>": <stmt>`

    <ident>,                                                    [2]
    ^^^^^^^-- Syntactic sugar for `"<ident>": <ident>`

    <ident>(<ident>, ...) { ... },                              [3]
  .-^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |                             
  '-- Syntactic sugar for `"<ident>": fn(<stmt>, ...) { ... }`
      note: there should **no** fn keyword

    <stmt>: <stmt>,                                             [4]
    ^^^^^^-- This statement **must** result in a string.
}
```

All key values pairs must be separated with commas, and can have an
optional comma after the last pair.

> Due to parsing limitations, for the *first key*, the following
> limitations apply:
> - [2] - `<ident>` must be followed with a comma
> - [4] - key `<stmt>` must **not** contain braces, i.e., `{` and `}`.

#### Operations

Non arithmetic binary and unary operations are possible on objects.
Any non empty object is truthy.

#### Indexing

Objects can be indexed only with strings. The resultant value is the
value stored with the string as key. If the corresponding key value pair
does not exist, it throws an error.

## Functions

Functions represent an immutable callable object.

#### Declaration

Refer [here](./syntax.md#function-declaration)

#### Operations

Non arithmetic binary and unary operations are possible on functions.
All functions are truthy, but the value they return when called need not
be.

#### Indexing

Functions have 3 properties present on it.
- `'call'` - This is a function that can be called call the function.

## Null

`null` represents no value. It is what is returned by statements when
there is no other sensible thing to return.

#### Declaration

Currently there is no specific way to declare null values.

#### Operations

Non arithmetic binary and unary operations are possible on nulls. Null
is falsy.

#### Indexing

Null cannot be indexed by any types.
