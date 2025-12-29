# K Object Index Trait Implementation

## Overview

The `Index` and `IndexMut` traits have been implemented for `K` objects to provide intuitive `[]` syntax access to dictionaries and tables.

## Implementation

### Dictionary Access by Position

`Index<usize>` and `IndexMut<usize>` allow accessing dictionary keys (index 0) and values (index 1):

```rust
let dict = k!(dict: k!(sym: vec!["a", "b"]) => k!(long: vec![10, 20]));

// Read access
let keys = &dict[0];    // Get keys K object
let values = &dict[1];  // Get values K object

// Mutable access
dict[1] = k!(long: vec![100, 200]);  // Replace values
```

**Panics if:**

- The K object is not a dictionary
- Index is out of bounds (not 0 or 1)

### Dictionary Lookup by Key

**New in v0.3.0:** `Index<&K>` and `IndexMut<&K>` allow looking up and modifying values in a dictionary using a K object as the key:

```rust
let mut dict = k!(dict:
    k!(sym: vec!["apple", "banana", "cherry"]) =>
    k!([k!(long: 10), k!(long: 20), k!(long: 30)])
);

// Read access: Look up value by key
let key = k!(sym: "banana");
let value = &dict[&key];  // Returns K object with value 20

// Write access: Modify value by key
dict[&key] = k!(long: 99);  // Update banana's value to 99
```

**Supported key types:**

- Symbol (`qtype::SYMBOL_LIST`)
- Long (`qtype::LONG_LIST`)
- Int (`qtype::INT_LIST`)
- Float (`qtype::FLOAT_LIST`)

**Note:** Dictionary values must be compound lists (not typed lists) for this feature to work.

**Panics if:**

- The K object is not a dictionary
- The key is not found in the dictionary
- The key type is not supported
- Dictionary values are not a compound list (for write access)

### Table Column Access by Name

`Index<&str>` and `IndexMut<&str>` allow accessing table columns by name:

```rust
let table = k!(table: {
    "fruit" => k!(sym: vec!["apple", "banana"]),
    "price" => k!(float: vec![1.5, 2.3])
});

// Read access
let fruits = &table["fruit"];
let prices = &table["price"];

// Mutable access
table["price"] = k!(float: vec![1.8, 2.5]);
```

**Panics if:**

- The K object is not a table
- The column name does not exist

### Compound List Access

Compound lists also support `try_index()` for safe access by position:

```rust
let compound = k!([k!(long: 1), k!(float: 2.5), k!(sym: "test")]);

// Safe access
if let Ok(first) = compound.try_index(0) {
    println!("First element: {:?}", first);
}
```

## Safe Access Methods

Non-panicking alternatives are provided for error handling:

### `try_index()` and `try_index_mut()`

Safe access to dictionaries and compound lists by position:

```rust
let dict = k!(dict: k!(sym: vec!["x"]) => k!(long: vec![42]));

// Returns Result<&K, Error>
match dict.try_index(0) {
    Ok(keys) => println!("Keys: {:?}", keys),
    Err(e) => println!("Error: {}", e),
}

// Mutable access
let mut dict = k!(dict: k!(sym: vec!["x"]) => k!(long: vec![42]));
if let Ok(values) = dict.try_index_mut(1) {
    *values = k!(long: vec![100]);
}
```

### `try_find()`

Safe dictionary key lookup for compound list values:

```rust
let dict = k!(dict:
    k!(sym: vec!["x", "y", "z"]) =>
    k!([k!(long: 1), k!(long: 2), k!(long: 3)])
);

// Returns Result<&K, Error> - only works with compound list values
let key = k!(sym: "y");
match dict.try_find(&key) {
    Ok(value) => println!("Value: {:?}", value),
    Err(e) => println!("Key not found: {}", e),
}
```

### `try_find_owned()`

Safe dictionary key lookup that works with both typed and compound list values:

```rust
// Works with typed list values (long list, symbol list, etc.)
let dict = k!(dict:
    k!(sym: vec!["a", "b", "c"]) =>
    k!(long: vec![10, 20, 30])
);

// Returns Result<K, Error> - creates owned K atom for typed lists
let key = k!(sym: "b");
match dict.try_find_owned(&key) {
    Ok(value) => println!("Value: {}", value.get_long().unwrap()),
    Err(e) => println!("Key not found: {}", e),
}

// Also works with compound list values (clones the K object)
let dict2 = k!(dict:
    k!(sym: vec!["x", "y"]) =>
    k!([k!(float: 1.5), k!(float: 2.5)])
);

let key2 = k!(sym: "y");
let value2 = dict2.try_find_owned(&key2).unwrap();
println!("Value: {}", value2.get_float().unwrap());
```

### `try_find_mut()`

Safe mutable dictionary key lookup for compound list values:

```rust
let mut dict = k!(dict:
    k!(sym: vec!["a", "b", "c"]) =>
    k!([k!(int: 10), k!(int: 20), k!(int: 30)])
);

// Returns Result<&mut K, Error> - only works with compound list values
let key = k!(sym: "b");
match dict.try_find_mut(&key) {
    Ok(value) => *value = k!(int: 99),
    Err(e) => println!("Key not found: {}", e),
}
```

### `set_value()`

Set a value in a dictionary by key, working with **both typed and compound list values**:

```rust
// Works with typed list values (float list, long list, symbol list, etc.)
let mut dict = k!(dict:
    k!(sym: vec!["apple", "banana", "cherry"]) =>
    k!(float: vec![1.5, 0.8, 2.3])
);

let key = k!(sym: "banana");
dict.set_value(&key, k!(float: 1.2)).unwrap();  // ✅ Updates individual element!

// Also works with compound list values
let mut dict2 = k!(dict:
    k!(sym: vec!["a", "b"]) =>
    k!([k!(int: 10), k!(int: 20)])
);

dict2.set_value(&k!(sym: "a"), k!(int: 99)).unwrap();
```

**Important behavior:**

- **Type preservation**: `set_value()` preserves the list type structure. It will not convert a typed list to a compound list or vice versa.
- **Type matching required**: The new value's type must match the list's element type. For typed lists, attempting to set a mismatched type returns an error.
- **Typed lists**: Value must be extractable via `get_long()`, `get_float()`, `get_symbol()`, etc., matching the list type.
- **Compound lists**: Any K object can be stored.

```rust
// Type safety example
let mut dict = k!(dict:
    k!(sym: vec!["a", "b"]) =>
    k!(long: vec![10, 20])  // Long list
);

// ✅ Success: type matches
dict.set_value(&k!(sym: "a"), k!(long: 100)).unwrap();

// ❌ Error: type mismatch - will not convert to compound list
dict.set_value(&k!(sym: "a"), k!(float: 1.5)).unwrap_err();

// To store mixed types, replace the entire value list with a compound list:
dict[1] = k!([k!(long: 10), k!(float: 1.5)]);  // Now it's a compound list
```

````

### `try_column()` and `try_column_mut()`

Safe access to table columns by name:

```rust
let table = k!(table: {
    "price" => k!(float: vec![1.5])
});

// Returns Result<&K, Error>
match table.try_column("price") {
    Ok(price) => println!("Price column: {:?}", price),
    Err(e) => println!("Column not found: {}", e),
}

// Returns Result<&mut K, Error>
let mut table = k!(table: {
    "price" => k!(float: vec![1.5])
});
if let Ok(price) = table.try_column_mut("price") {
    *price = k!(float: vec![2.0]);
}
````

## Benefits

✅ **Ergonomic**: Natural `[]` syntax familiar to Rust users  
✅ **Type Safe**: Rust's borrow checker enforces safety  
✅ **Flexible**: Both panicking (`[]`) and safe (`try_*`) APIs available  
✅ **Backward Compatible**: Existing methods remain unchanged  
✅ **Intuitive**: Matches expectations from Python/q users

## Usage Guidelines

**Use `[]` syntax when:**

- You're certain the key/column exists (e.g., in tests)
- Invalid access is a programming error that should panic
- Code readability is prioritized
- Dictionary values are compound lists (for `dict[&key]` syntax)

**Use `try_find_owned()` when:**

- Working with typed list values in dictionaries
  **Use `try_find_owned()` when:**
- Working with typed list values in dictionaries
- You need an owned K object (not just a reference)
- You want to avoid panics and handle missing keys gracefully

**Use `set_value()` when:**

- You need to update values in typed list dictionaries
- You want a single method that works with all dictionary value types
- You want element-wise updates without replacing the entire list

**Use `try_*` methods when:**

- Working with user input or dynamic data
- You need to handle missing keys/columns gracefully
- Building robust production code

## Example: Combining Both Approaches

```rust
use kdb_codec::*;

fn process_table(table: &K) -> Result<(), Error> {
    // Use try_* for dynamic column access
    if let Ok(price_col) = table.try_column("price") {
        println!("Processing prices: {:?}", price_col);
    }

    // Use [] for known columns in structured code
    let required_col = &table["id"];  // Panics if missing - that's a bug
    println!("ID column: {:?}", required_col);

    Ok(())
}

// In tests, [] is more readable
#[test]
fn test_table_structure() {
    let table = k!(table: {
        "id" => k!(long: vec![1, 2, 3]),
        "name" => k!(sym: vec!["a", "b", "c"])
    });

    assert_eq!(table["id"].len(), 3);
    assert_eq!(table["name"].get_type(), qtype::SYMBOL_LIST);
}
```

## See Also

- [index.rs](kdb_codec/src/index.rs) - Full implementation with documentation
- [README.md](README.md) - Usage examples in the main documentation
- [index_trait_demo.rs](ipc_examples/examples/index_trait_demo.rs) - Comprehensive examples
