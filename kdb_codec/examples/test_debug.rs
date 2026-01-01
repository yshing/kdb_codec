use kdb_codec::*;

fn main() {
    // Try with just 105 levels
    let nesting_depth = 110;
    let mut bytes = Vec::new();

    for _ in 0..nesting_depth {
        bytes.push(0); // COMPOUND_LIST
        bytes.push(0); // Attribute
        bytes.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // Size: 1
    }

    bytes.push(250_u8); // INT_ATOM = -6
    bytes.extend_from_slice(&[0x2A, 0x00, 0x00, 0x00]); // Value: 42

    println!("Testing with {} nested levels", nesting_depth);
    println!("MAX_RECURSION_DEPTH = {}", MAX_RECURSION_DEPTH);

    match K::q_ipc_decode(&bytes, 1) {
        Ok(_) => println!("ERROR: Should have failed!"),
        Err(e) => println!("Good: Got error: {}", e),
    }
}
