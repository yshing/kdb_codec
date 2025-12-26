# K Macro - Simplified kdb+ Data Construction in Rust

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

## Temporal Types

The `k!` macro supports all kdb+ temporal types using Rust's `chrono` library.

### Temporal Atoms

```rust
use kdb_codec::*;
use chrono::prelude::*;
use chrono::Duration;

// Timestamp (datetime with nanosecond precision)
let ts = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()
    .and_hms_nano_opt(10, 30, 0, 123456789).unwrap()
    .and_local_timezone(Utc).unwrap();
let timestamp = k!(timestamp: ts);

// Date
let date = NaiveDate::from_ymd_opt(2024, 12, 25).unwrap();
let d = k!(date: date);

// Month
let month_date = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
let m = k!(month: month_date);

// Datetime (datetime with millisecond precision)
let dt = k!(datetime: ts);

// Duration-based types
let timespan = k!(timespan: Duration::hours(5));
let minute = k!(minute: Duration::minutes(30));
let second = k!(second: Duration::seconds(90));
let time = k!(time: Duration::milliseconds(1000));
```

### Temporal Lists

```rust
use kdb_codec::*;
use chrono::prelude::*;
use chrono::Duration;

// Date list with sorted attribute
let dates = vec![
    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
    NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()
];
let date_list = k!(date: vec![dates[0], dates[1], dates[2]]; @sorted);

// Timestamp list
let timestamps = vec![
    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
        .and_hms_opt(9, 30, 0).unwrap()
        .and_local_timezone(Utc).unwrap(),
    NaiveDate::from_ymd_opt(2024, 1, 2).unwrap()
        .and_hms_opt(10, 30, 0).unwrap()
        .and_local_timezone(Utc).unwrap()
];
let ts_list = k!(timestamp: vec![timestamps[0], timestamps[1]]; @sorted);

// Time list
let times = vec![
    Duration::hours(9),
    Duration::hours(12) + Duration::minutes(30),
    Duration::hours(17) + Duration::minutes(45)
];
let time_list = k!(time: vec![times[0], times[1], times[2]]);

// Timespan list
let spans = k!(timespan: vec![
    Duration::hours(1),
    Duration::minutes(30),
    Duration::seconds(90)
]);
```

### Trading Table with Timestamps

```rust
use kdb_codec::*;
use chrono::prelude::*;

let trades = k!(table: {
    "time" => k!(timestamp: vec![
        NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()
            .and_hms_opt(9, 30, 0).unwrap()
            .and_local_timezone(Utc).unwrap(),
        NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()
            .and_hms_milli_opt(9, 30, 1, 500).unwrap()
            .and_local_timezone(Utc).unwrap()
    ]; @sorted),
    "symbol" => k!(sym: vec!["AAPL", "GOOGL"]),
    "price" => k!(float: vec![150.25, 2801.5]),
    "size" => k!(int: vec![100, 50])
});
```

### Supported Temporal Types

| Type | Q Type | Rust Input | Example |
|------|--------|------------|---------|
| `timestamp` | timestamp | `DateTime<Utc>` | `k!(timestamp: dt)` |
| `date` | date | `NaiveDate` | `k!(date: date)` |
| `month` | month | `NaiveDate` | `k!(month: date)` |
| `datetime` | datetime | `DateTime<Utc>` | `k!(datetime: dt)` |
| `timespan` | timespan | `Duration` | `k!(timespan: dur)` |
| `minute` | minute | `Duration` | `k!(minute: dur)` |
| `second` | second | `Duration` | `k!(second: dur)` |
| `time` | time | `Duration` | `k!(time: dur)` |

### Compound Lists

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

### Dictionaries

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

### Tables

Create kdb+ tables using the `table:` syntax:

```rust
use kdb_codec::*;

let table = k!(table: {
    "id" => k!(int: vec![1, 2, 3]),
    "name" => k!(sym: vec!["Alice", "Bob", "Charlie"]),
    "score" => k!(float: vec![95.5, 87.3, 92.1])
});
```

## Complete Example

Here's a comprehensive example showing various features:

```rust
use kdb_codec::*;

fn main() {
    // Create a trading table
    let trades = k!(table: {
        "time" => k!(long: vec![1000, 2000, 3000]),
        "symbol" => k!(sym: vec!["AAPL", "GOOGL", "MSFT"]; @unique),
        "price" => k!(float: vec![150.5, 2800.25, 300.75]),
        "size" => k!(long: vec![100, 50, 200])
    });
    
    println!("Trades table: {}", trades);
    
    // Create a dictionary for metadata
    let metadata = k!(dict:
        k!(sym: vec!["date", "exchange", "currency"]) =>
        k!([
            k!(string: "2024-01-15"),
            k!(sym: "NYSE"),
            k!(sym: "USD")
        ])
    );
    
    println!("Metadata: {}", metadata);
    
    // Create a compound list with various data
    let report = k!([
        trades,
        metadata,
        k!(string: "Trading Report"),
        k!(long: 20240115)
    ]);
    
    println!("Report: {}", report);
}
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

## Running the Example

```bash
cargo run --example macro_demo
```

This will show various examples of the macro in action with their outputs in q format.
