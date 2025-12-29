//! Dictionary lookup demonstration using Index trait
//!
//! This example demonstrates two ways to look up dictionary values:
//! 1. Index trait (dict[&key]) - for compound list values only
//! 2. try_find_owned() - for both typed lists and compound lists
//!
//! Run with: cargo run --example dictionary_lookup_demo

use kdb_codec::*;

fn main() {
    println!("=== Dictionary Lookup Demo ===\n");

    // Example 1: Compound list values with Index trait
    println!("1. Symbol keys with compound list values (using [] syntax):");
    let fruit_prices = k!(dict:
        k!(sym: vec!["apple", "banana", "cherry", "date"]) =>
        k!([k!(float: 1.5), k!(float: 0.8), k!(float: 2.3), k!(float: 3.0)])
    );

    let apple_key = k!(sym: "apple");
    let apple_price = &fruit_prices[&apple_key];
    println!("  apple price: ${:.2}", apple_price.get_float().unwrap());

    let cherry_key = k!(sym: "cherry");
    let cherry_price = &fruit_prices[&cherry_key];
    println!(
        "  cherry price: ${:.2}\n",
        cherry_price.get_float().unwrap()
    );

    // Example 2: Typed list values with try_find_owned
    println!("2. Typed list values (long list) with try_find_owned:");
    let product_quantities = k!(dict:
        k!(sym: vec!["apple", "banana", "cherry"]) =>
        k!(long: vec![100, 250, 80])
    );

    let banana_key = k!(sym: "banana");
    match product_quantities.try_find_owned(&banana_key) {
        Ok(quantity) => println!("  banana quantity: {}", quantity.get_long().unwrap()),
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // Example 3: Symbol list values
    println!("3. Symbol list values with try_find_owned:");
    let id_to_name = k!(dict:
        k!(int: vec![101, 102, 103]) =>
        k!(sym: vec!["Alice", "Bob", "Charlie"])
    );

    let id_102 = k!(int: 102);
    match id_to_name.try_find_owned(&id_102) {
        Ok(name) => println!("  ID 102 => {}", name.get_symbol().unwrap()),
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // Example 4: Safe lookup with try_find (compound list)
    println!("4. Safe lookup with try_find (compound list values):");
    match fruit_prices.try_find(&k!(sym: "banana")) {
        Ok(price) => println!("  banana price: ${:.2}", price.get_float().unwrap()),
        Err(e) => println!("  Error: {}", e),
    }

    match fruit_prices.try_find(&k!(sym: "mango")) {
        Ok(price) => println!("  mango price: ${:.2}", price.get_float().unwrap()),
        Err(_) => println!("  mango not found in dictionary\n"),
    }

    // Example 5: Mutating dictionaries - compound list vs typed list
    println!("5. Mutating dictionary values:\n");

    // 5a. Dictionary with compound list values - can mutate individual elements
    println!("  a) Compound list values (mutable via Index trait):");
    let mut scores = k!(dict:
        k!(sym: vec!["alice", "bob", "charlie"]) =>
        k!([k!(int: 85), k!(int: 90), k!(int: 78)])
    );

    println!(
        "     Before: alice={}",
        scores[&k!(sym: "alice")].get_int().unwrap()
    );

    // Mutate via position index (values are at index 1)
    scores[1].as_mut_vec::<K>().unwrap()[0] = k!(int: 95);

    println!(
        "     After: alice={}",
        scores[&k!(sym: "alice")].get_int().unwrap()
    );

    let key = k!(sym: "alice");
    scores[&key] = k!(int: 88); // Another way to update alice's score

    println!(
        "     After: alice={}",
        scores[&k!(sym: "alice")].get_int().unwrap()
    );

    // 5b. Dictionary with typed list values - replace entire values list
    println!("\n  b) Typed list values (can now mutate individual elements!):");
    let mut prices = k!(dict:
        k!(sym: vec!["apple", "banana", "cherry"]) =>
        k!(float: vec![1.5, 0.8, 2.3])
    );

    println!(
        "     Before: banana=${:.2}",
        prices
            .try_find_owned(&k!(sym: "banana"))
            .unwrap()
            .get_float()
            .unwrap()
    );

    // Method 1: Use set_value() to update individual element
    let banana_key = k!(sym: "banana");
    prices.set_value(&banana_key, k!(float: 1.2)).unwrap();

    println!(
        "     After (set_value): banana=${:.2}",
        prices
            .try_find_owned(&k!(sym: "banana"))
            .unwrap()
            .get_float()
            .unwrap()
    );

    // Method 2: Replace entire values list (old way)
    prices[1] = k!(float: vec![1.5, 0.9, 2.3]); // Replace all values

    println!(
        "     After (replace list): banana=${:.2}",
        prices
            .try_find_owned(&k!(sym: "banana"))
            .unwrap()
            .get_float()
            .unwrap()
    );

    println!("\n  Note: Compound lists allow element-wise mutation via IndexMut,");
    println!("        Typed lists can use set_value() for element updates or replace entire list.");

    // Example 6: Method comparison
    println!("\n6. Method comparison:");
    println!("  [] syntax: Fast, panics if key missing, compound list values only");
    println!("  try_find(): Safe, returns &K, compound list values only");
    println!("  try_find_owned(): Safe, works with all list types, returns owned K");

    println!("\n=== Demo Complete ===");
}
