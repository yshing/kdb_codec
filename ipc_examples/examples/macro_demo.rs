//! Example demonstrating the k! macro for creating K objects with simplified syntax.
//!
//! This example shows how the k! macro reduces boilerplate and makes
//! creating kdb+ data structures in Rust more intuitive.

use kdb_codec::*;

fn main() {
    println!("=== K Macro Examples ===\n");

    // ========== Atoms ==========
    println!("--- Atoms ---");
    
    let bool_true = k!(bool: true);
    println!("Boolean true: {}", bool_true);
    
    let bool_false = k!(bool: false);
    println!("Boolean false: {}", bool_false);
    
    let byte_val = k!(byte: 0x2a);
    println!("Byte: {}", byte_val);
    
    let short_val = k!(short: 42);
    println!("Short: {}", short_val);
    
    let int_val = k!(int: 42);
    println!("Int: {}", int_val);
    
    let long_val = k!(long: 42);
    println!("Long: {}", long_val);
    
    let real_val = k!(real: 42.5);
    println!("Real: {}", real_val);
    
    let float_val = k!(float: 42.5);
    println!("Float: {}", float_val);
    
    let char_val = k!(char: 'a');
    println!("Char: {}", char_val);
    
    let symbol_val = k!(sym: "hello");
    println!("Symbol: {}", symbol_val);
    
    let string_val = k!(string: "hello world");
    println!("String: {}", string_val);

    // ========== Lists ==========
    println!("\n--- Lists ---");
    
    let bool_list = k!(bool: vec![true, false, true, false]);
    println!("Bool list: {}", bool_list);
    
    let byte_list = k!(byte: vec![0x01, 0x02, 0xff]);
    println!("Byte list: {}", byte_list);
    
    let short_list = k!(short: vec![10, 20, 30]);
    println!("Short list: {}", short_list);
    
    let int_list = k!(int: vec![100, 200, 300]);
    println!("Int list: {}", int_list);
    
    let long_list = k!(long: vec![1, 2, 3, 4, 5]);
    println!("Long list: {}", long_list);
    
    let real_list = k!(real: vec![1.1, 2.2, 3.3]);
    println!("Real list: {}", real_list);
    
    let float_list = k!(float: vec![1.1, 2.2, 3.3]);
    println!("Float list: {}", float_list);
    
    let symbol_list = k!(sym: vec!["apple", "banana", "cherry"]);
    println!("Symbol list: {}", symbol_list);

    // ========== Lists with Attributes ==========
    println!("\n--- Lists with Attributes ---");
    
    let sorted_list = k!(long: vec![1, 2, 3, 4, 5]; @sorted);
    println!("Sorted long list: {}", sorted_list);
    
    let unique_symbols = k!(sym: vec!["a", "b", "c"]; @unique);
    println!("Unique symbol list: {}", unique_symbols);

    // ========== Compound Lists ==========
    println!("\n--- Compound Lists ---");
    
    let compound = k!([
        k!(long: 42),
        k!(float: 3.14),
        k!(sym: "symbol"),
        k!(long: vec![1, 2, 3])
    ]);
    println!("Compound list: {}", compound);

    // ========== Dictionaries ==========
    println!("\n--- Dictionaries ---");
    
    let simple_dict = k!(dict:
        k!(int: vec![1, 2, 3]) => k!(sym: vec!["a", "b", "c"])
    );
    println!("Simple dictionary: {}", simple_dict);
    
    let complex_dict = k!(dict:
        k!(sym: vec!["name", "age", "city"]) =>
        k!([k!(string: "Alice"), k!(int: 30), k!(sym: "NYC")])
    );
    println!("Complex dictionary: {}", complex_dict);

    // ========== Tables ==========
    println!("\n--- Tables ---");
    
    let table = k!(table: {
        "id" => k!(int: vec![1, 2, 3]),
        "price" => k!(float: vec![10.5, 20.3, 15.8]),
        "qty" => k!(long: vec![100, 200, 150])
    });
    println!("Table: {}", table);

    // ========== Comparison: Before and After ==========
    println!("\n--- Before vs After Comparison ---");
    
    println!("\nOld way (verbose):");
    let old_way = K::new_dictionary(
        K::new_symbol_list(
            vec![
                String::from("fruit"),
                String::from("price"),
                String::from("qty")
            ],
            qattribute::NONE
        ),
        K::new_compound_list(vec![
            K::new_symbol_list(
                vec![
                    String::from("apple"),
                    String::from("banana")
                ],
                qattribute::NONE
            ),
            K::new_float_list(vec![1.5, 0.8], qattribute::NONE),
            K::new_long_list(vec![100, 150], qattribute::NONE),
        ])
    ).unwrap().flip().unwrap();
    println!("{}", old_way);
    
    println!("\nNew way (with k! macro):");
    let new_way = k!(table: {
        "fruit" => k!(sym: vec!["apple", "banana"]),
        "price" => k!(float: vec![1.5, 0.8]),
        "qty" => k!(long: vec![100, 150])
    });
    println!("{}", new_way);

    println!("\n=== Much cleaner and more intuitive! ===");
}


