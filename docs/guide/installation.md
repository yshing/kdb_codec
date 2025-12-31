# Installation

Add `kdb_codec` to your `Cargo.toml`:

```toml
[dependencies]
kdb_codec = "0.4"
```

The IPC feature is enabled by default.

## Requirements

- Rust 1.70 or later
- Tokio runtime for async operations

## Optional Features

The crate comes with IPC enabled by default. You can control feature flags in your `Cargo.toml`:

```toml
[dependencies]
kdb_codec = { version = "0.4", default-features = false, features = ["ipc"] }
```

## Verify Installation

Create a simple test to verify the installation:

```rust
use kdb_codec::*;

fn main() {
    // Create a simple K object
    let value = K::new_long(42);
    println!("Created K object: {}", value);
    
    // Create a table using the k! macro
    let table = k!(table: {
        "id" => k!(int: vec![1, 2, 3]),
        "name" => k!(sym: vec!["Alice", "Bob", "Charlie"])
    });
    println!("Created table: {}", table);
}
```

If this compiles and runs without errors, your installation is working correctly.
