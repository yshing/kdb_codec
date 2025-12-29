//! Example demonstrating Index trait usage for K objects
//!
//! This example shows how to use the convenient `[]` syntax to access
//! dictionary and table data, as well as safe access methods.

use kdb_codec::*;

fn main() -> Result<()> {
    println!("=== K Object Index Trait Examples ===\n");

    // Example 1: Dictionary Access
    println!("1. Dictionary Access with [] syntax:");
    dictionary_example()?;

    // Example 2: Table Column Access
    println!("\n2. Table Column Access with [] syntax:");
    table_example()?;

    // Example 3: Safe Access Methods
    println!("\n3. Safe Access Methods (non-panicking):");
    safe_access_example()?;

    // Example 4: Mutable Access
    println!("\n4. Mutable Access with []:");
    mutable_access_example()?;

    // Example 5: Compound List Access
    println!("\n5. Compound List Access:");
    compound_list_example()?;

    Ok(())
}

fn dictionary_example() -> Result<()> {
    // Create a dictionary with symbol keys and long values
    let dict = k!(dict:
        k!(sym: vec!["apple", "banana", "cherry"]) =>
        k!(long: vec![100, 200, 300])
    );

    // Access keys and values using [] syntax
    let dict_keys = &dict[0]; // Get keys
    let dict_values = &dict[1]; // Get values

    println!("  Dictionary keys: {}", dict_keys);
    println!("  Dictionary values: {}", dict_values);

    // Access specific values from the vector
    let key_vec = dict_keys.as_vec::<String>()?;
    let value_vec = dict_values.as_vec::<i64>()?;

    for (k, v) in key_vec.iter().zip(value_vec.iter()) {
        println!("  {} -> {}", k, v);
    }

    Ok(())
}

fn table_example() -> Result<()> {
    // Create a table with fruit names and prices
    let table = k!(table: {
        "fruit" => k!(sym: vec!["apple", "banana", "cherry"]),
        "price" => k!(float: vec![1.5, 2.3, 3.8]),
        "quantity" => k!(long: vec![100, 150, 75])
    });

    // Access columns by name using [] syntax
    let fruits = &table["fruit"];
    let prices = &table["price"];
    let quantities = &table["quantity"];

    println!("  Fruits: {}", fruits);
    println!("  Prices: {}", prices);
    println!("  Quantities: {}", quantities);

    // Access individual values
    let fruit_vec = fruits.as_vec::<String>()?;
    let price_vec = prices.as_vec::<f64>()?;
    let qty_vec = quantities.as_vec::<i64>()?;

    println!("\n  Table data:");
    println!("  {:12} {:8} {:8}", "Fruit", "Price", "Qty");
    println!("  {}", "-".repeat(30));
    for i in 0..fruit_vec.len() {
        println!(
            "  {:12} ${:<7.2} {:<8}",
            fruit_vec[i], price_vec[i], qty_vec[i]
        );
    }

    Ok(())
}

fn safe_access_example() -> Result<()> {
    let dict = k!(dict: k!(sym: vec!["x", "y"]) => k!(long: vec![10, 20]));

    // Safe access with try_index - returns Result
    println!("  Safe dictionary access:");
    match dict.try_index(0) {
        Ok(keys) => println!("    Keys (safe): {}", keys),
        Err(e) => println!("    Error: {:?}", e),
    }

    match dict.try_index(1) {
        Ok(values) => println!("    Values (safe): {}", values),
        Err(e) => println!("    Error: {:?}", e),
    }

    // Try to access out of bounds - this won't panic
    match dict.try_index(2) {
        Ok(_) => println!("    Index 2: Success (unexpected!)"),
        Err(_) => println!("    Index 2: Out of bounds (expected)"),
    }

    // Safe table column access
    let table = k!(table: {
        "name" => k!(sym: vec!["Alice"])
    });

    println!("\n  Safe table column access:");
    match table.try_column("name") {
        Ok(col) => println!("    Column 'name': {}", col),
        Err(e) => println!("    Error: {:?}", e),
    }

    match table.try_column("nonexistent") {
        Ok(_) => println!("    Column 'nonexistent': Found (unexpected!)"),
        Err(_) => println!("    Column 'nonexistent': Not found (expected)"),
    }

    Ok(())
}

fn mutable_access_example() -> Result<()> {
    // Create a mutable dictionary
    let mut dict = k!(dict: k!(sym: vec!["count"]) => k!(long: vec![42]));

    println!("  Original dictionary: {}", dict);

    // Modify values using [] syntax
    dict[1] = k!(long: vec![999]);

    println!("  Modified dictionary: {}", dict);

    // Create a mutable table
    let mut table = k!(table: {
        "price" => k!(float: vec![10.5])
    });

    println!("\n  Original table: {}", table);

    // Modify column using [] syntax
    table["price"] = k!(float: vec![25.0]);

    println!("  Modified table: {}", table);

    Ok(())
}

fn compound_list_example() -> Result<()> {
    // Create a compound list (heterogeneous list)
    let list = k!([
        k!(long: 100),
        k!(float: 3.14),
        k!(sym: "hello"),
        k!(bool: vec![true, false, true])
    ]);

    println!("  Compound list: {}", list);

    // Access elements safely
    if let Ok(first) = list.try_index(0) {
        println!("  Element 0: {} (type: {})", first, first.get_type());
    }

    if let Ok(second) = list.try_index(1) {
        println!("  Element 1: {} (type: {})", second, second.get_type());
    }

    if let Ok(third) = list.try_index(2) {
        println!("  Element 2: {} (type: {})", third, third.get_type());
    }

    if let Ok(fourth) = list.try_index(3) {
        println!("  Element 3: {} (type: {})", fourth, fourth.get_type());
    }

    // Try accessing out of bounds
    match list.try_index(4) {
        Ok(_) => println!("  Element 4: Found (unexpected)"),
        Err(_) => println!("  Element 4: Out of bounds (expected)"),
    }

    Ok(())
}
