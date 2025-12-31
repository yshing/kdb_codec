# K Macro

The `k!` macro provides a clean and intuitive way to create kdb+/q data structures in Rust, significantly reducing boilerplate code.

## Overview

Instead of writing verbose constructor calls like:
```rust
K::new_long_list(vec![1, 2, 3], qattribute::NONE)
```

You can now write:
```rust
k!(long: vec![1, 2, 3])
```

## Basic Usage

### Atoms

Create atomic (scalar) values:

```rust
use kdb_codec::*;

// Numeric types
let bool_val = k!(bool: true);
let byte_val = k!(byte: 0x2a);
let short_val = k!(short: 42);
let int_val = k!(int: 100);
let long_val = k!(long: 1000);
let real_val = k!(real: 3.14);
let float_val = k!(float: 2.718);

// Text types
let char_val = k!(char: 'a');
let symbol_val = k!(sym: "hello");
let string_val = k!(string: "world");
```

### Lists

Create typed lists using `vec![]`:

```rust
use kdb_codec::*;

// Boolean list
let bools = k!(bool: vec![true, false, true]);

// Numeric lists
let bytes = k!(byte: vec![1, 2, 3]);
let shorts = k!(short: vec![10, 20, 30]);
let ints = k!(int: vec![100, 200, 300]);
let longs = k!(long: vec![1, 2, 3, 4, 5]);
let reals = k!(real: vec![1.1, 2.2, 3.3]);
let floats = k!(float: vec![1.1, 2.2, 3.3]);

// Symbol list
let symbols = k!(sym: vec!["apple", "banana", "cherry"]);
```

### Lists with Repetition Syntax

Create large lists efficiently using `vec![value; count]` syntax:

```rust
use kdb_codec::*;

// Create a list with 3000 copies of 42
let large_list = k!(long: vec![42; 3000]);

// Works with all types
let int_list = k!(int: vec![10; 100]);
let float_list = k!(float: vec![3.14; 50]);
let sym_list = k!(sym: vec!["test"; 10]);

// Also works with attributes
let sorted = k!(long: vec![1; 2500]; @sorted);
```

### Lists with Attributes

Add q attributes using the `@attribute` syntax:

```rust
use kdb_codec::*;

let sorted = k!(long: vec![1, 2, 3, 4, 5]; @sorted);
let unique = k!(sym: vec!["a", "b", "c"]; @unique);
let parted = k!(int: vec![1, 2, 3]; @parted);
let grouped = k!(long: vec![1, 1, 2, 2]; @grouped);
```

Available attributes:
- `@sorted` - sorted attribute (`s#`)
- `@unique` - unique attribute (`u#`)
- `@parted` - parted attribute (`p#`)
- `@grouped` - grouped attribute (`g#`)

## Compound Lists

Create mixed-type lists:

```rust
use kdb_codec::*;

let compound = k!([
    k!(long: 42),
    k!(float: 3.14),
    k!(sym: "hello"),
    k!(long: vec![1, 2, 3])
]);
```

## Dictionaries

Create key-value dictionaries:

```rust
use kdb_codec::*;

// Simple dictionary
let dict = k!(dict:
    k!(int: vec![1, 2, 3]) => k!(sym: vec!["a", "b", "c"])
);

// Complex dictionary with mixed value types
let complex = k!(dict:
    k!(sym: vec!["name", "age", "score"]) =>
    k!([
        k!(string: "Alice"),
        k!(int: 30),
        k!(float: 95.5)
    ])
);
```

## Tables

Create kdb+ tables using the `table:` syntax:

```rust
use kdb_codec::*;

let table = k!(table: {
    "id" => k!(int: vec![1, 2, 3]),
    "name" => k!(sym: vec!["Alice", "Bob", "Charlie"]),
    "score" => k!(float: vec![95.5, 87.3, 92.1])
});
```

## Comparison: Before and After

### Before (verbose):
```rust
let table = K::new_dictionary(
    K::new_symbol_list(
        vec![
            String::from("id"),
            String::from("price"),
            String::from("qty")
        ],
        qattribute::NONE
    ),
    K::new_compound_list(vec![
        K::new_int_list(vec![1, 2, 3], qattribute::NONE),
        K::new_float_list(vec![10.5, 20.3, 15.8], qattribute::NONE),
        K::new_long_list(vec![100, 200, 150], qattribute::NONE),
    ])
).unwrap().flip().unwrap();
```

### After (with k! macro):
```rust
let table = k!(table: {
    "id" => k!(int: vec![1, 2, 3]),
    "price" => k!(float: vec![10.5, 20.3, 15.8]),
    "qty" => k!(long: vec![100, 200, 150])
});
```

## Type Reference

| q Type      | Macro Syntax                              | Example                                    |
|-------------|-------------------------------------------|--------------------------------------------|
| bool        | `k!(bool: value)`                         | `k!(bool: true)`                           |
| byte        | `k!(byte: value)`                         | `k!(byte: 0xff)`                           |
| short       | `k!(short: value)`                        | `k!(short: 42)`                            |
| int         | `k!(int: value)`                          | `k!(int: 100)`                             |
| long        | `k!(long: value)`                         | `k!(long: 1000)`                           |
| real        | `k!(real: value)`                         | `k!(real: 3.14)`                           |
| float       | `k!(float: value)`                        | `k!(float: 2.718)`                         |
| char        | `k!(char: value)`                         | `k!(char: 'a')`                            |
| symbol      | `k!(sym: "text")`                         | `k!(sym: "hello")`                         |
| string      | `k!(string: "text")`                      | `k!(string: "hello world")`                |
| bool list   | `k!(bool: vec![...])`                     | `k!(bool: vec![true, false])`              |
| int list    | `k!(int: vec![...])`                      | `k!(int: vec![1, 2, 3])`                   |
| symbol list | `k!(sym: vec![...])`                      | `k!(sym: vec!["a", "b"])`                  |
| compound    | `k!([item1, item2, ...])`                 | `k!([k!(int: 1), k!(sym: "a")])`           |
| dictionary  | `k!(dict: keys => values)`                | `k!(dict: k!(int: vec![1]) => k!(sym: vec!["a"]))` |
| table       | `k!(table: { "col" => values })`          | `k!(table: {"id" => k!(int: vec![1, 2])})`  |

## Tips

1. **Always use `vec![]`** for lists to distinguish them from atoms
2. **Attributes** are added with `; @attribute` after the list
3. **Tables** use braces `{}` syntax with column names as strings
4. **Compound lists** use square brackets `[]` with comma separation
5. **Type consistency**: All elements in a typed list must be of the same type
