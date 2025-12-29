//! Dictionary lookup demonstration using Index trait
//!
//! This example demonstrates how to use dictionary key lookup with the Index trait.
//!
//! Run with: cargo run --example dictionary_lookup_demo

use kdb_codec::*;

fn main() {
    println!("=== Dictionary Lookup Demo ===\n");

    // Example 1: Symbol keys with long values
    println!("1. Symbol keys with compound list values:");
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

    // Example 2: Integer keys
    println!("2. Integer keys:");
    let id_to_name = k!(dict:
        k!(int: vec![101, 102, 103]) =>
        k!([k!(sym: "Alice"), k!(sym: "Bob"), k!(sym: "Charlie")])
    );

    let id_102 = k!(int: 102);
    let name = &id_to_name[&id_102];
    println!("  ID 102 => {}\n", name.get_symbol().unwrap());

    // Example 3: Long keys
    println!("3. Long keys:");
    let timestamp_values = k!(dict:
        k!(long: vec![1000, 2000, 3000]) =>
        k!([k!(float: 10.5), k!(float: 20.3), k!(float: 30.1)])
    );

    let ts_2000 = k!(long: 2000);
    let value = &timestamp_values[&ts_2000];
    println!("  timestamp 2000 => {:.1}\n", value.get_float().unwrap());

    // Example 4: Safe lookup with try_find
    println!("4. Safe lookup with try_find:");
    match fruit_prices.try_find(&k!(sym: "banana")) {
        Ok(price) => println!("  banana price: ${:.2}", price.get_float().unwrap()),
        Err(e) => println!("  Error: {}", e),
    }

    match fruit_prices.try_find(&k!(sym: "mango")) {
        Ok(price) => println!("  mango price: ${:.2}", price.get_float().unwrap()),
        Err(_) => println!("  mango not found in dictionary\n"),
    }

    // Example 5: Building a simple cache
    println!("5. Simple cache example:");
    let cache = k!(dict:
        k!(sym: vec!["user:1", "user:2", "user:3"]) =>
        k!([
            k!(table: {
                "name" => k!(sym: vec!["Alice"]),
                "age" => k!(long: vec![30])
            }),
            k!(table: {
                "name" => k!(sym: vec!["Bob"]),
                "age" => k!(long: vec![25])
            }),
            k!(table: {
                "name" => k!(sym: vec!["Charlie"]),
                "age" => k!(long: vec![35])
            })
        ])
    );

    let user_key = k!(sym: "user:2");
    if let Ok(user_data) = cache.try_find(&user_key) {
        let name = &user_data["name"];
        let age = &user_data["age"];
        println!(
            "  Found user: {} (age: {})",
            name.as_vec::<String>().unwrap()[0],
            age.as_vec::<i64>().unwrap()[0]
        );
    }

    println!("\n=== Demo Complete ===");
}
