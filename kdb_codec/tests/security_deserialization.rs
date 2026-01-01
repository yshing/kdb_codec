//! Security Tests: Deserialization Vulnerabilities
//!
//! Tests for integer overflow in list allocations, invalid UTF-8, and unbounded recursion

use kdb_codec::*;

#[test]
fn test_large_list_allocation_i64() {
    // Ensure list-size limits reject oversized lists without allocating.
    let size = (MAX_LIST_SIZE as u32) + 1;
    let bytes = vec![
        qtype::LONG_LIST as u8, // Type: long list
        0x00,                   // Attribute: none
        size as u8,
        (size >> 8) as u8,
        (size >> 16) as u8,
        (size >> 24) as u8, // Size: MAX_LIST_SIZE + 1
    ];

    let err = K::q_ipc_decode(&bytes, 1).expect_err("should reject oversized list");
    assert!(
        matches!(err, Error::ListTooLarge { .. }),
        "expected ListTooLarge, got: {err:?}"
    );
}

#[test]
fn test_large_list_allocation_guid() {
    // Ensure list-size limits reject oversized lists without allocating.
    let size = (MAX_LIST_SIZE as u32) + 1;
    let bytes = vec![
        qtype::GUID_LIST as u8, // Type: GUID list
        0x00,                   // Attribute: none
        size as u8,
        (size >> 8) as u8,
        (size >> 16) as u8,
        (size >> 24) as u8, // Size: MAX_LIST_SIZE + 1
    ];

    let err = K::q_ipc_decode(&bytes, 1).expect_err("should reject oversized list");
    assert!(
        matches!(err, Error::ListTooLarge { .. }),
        "expected ListTooLarge, got: {err:?}"
    );
}

#[test]
fn test_integer_overflow_in_size_calculation() {
    // The deserializer reads list lengths as u32 and checks MAX_LIST_SIZE before
    // any byte-count multiplication. This test ensures we still fail fast.
    let size = (MAX_LIST_SIZE as u32) + 1;
    let bytes = vec![
        qtype::LONG_LIST as u8,
        0x00,
        size as u8,
        (size >> 8) as u8,
        (size >> 16) as u8,
        (size >> 24) as u8,
    ];

    let err = K::q_ipc_decode(&bytes, 1).expect_err("should reject oversized list");
    assert!(matches!(err, Error::ListTooLarge { .. }));
}

#[test]
fn test_symbol_without_null_terminator() {
    // Test symbol deserialization without null terminator
    let bytes = vec![
        qtype::SYMBOL_ATOM as u8, // Type: symbol
        b'h',
        b'e',
        b'l',
        b'l',
        b'o', // "hello" without null terminator
    ];

    let err = K::q_ipc_decode(&bytes, 1).expect_err("should require null terminator");
    assert!(matches!(err, Error::MissingNullTerminator));
}

#[test]
fn test_symbol_with_invalid_utf8() {
    // Test symbol with invalid UTF-8 sequence
    let bytes = vec![
        qtype::SYMBOL_ATOM as u8, // Type: symbol
        0xFF,
        0xFE,
        0xFD, // Invalid UTF-8 bytes
        0x00, // Null terminator
    ];

    let err = K::q_ipc_decode(&bytes, 1).expect_err("should reject invalid UTF-8");
    assert!(matches!(err, Error::InvalidUtf8));
}

#[test]
fn test_string_with_invalid_utf8() {
    // Test string (char list) with invalid UTF-8
    let bytes = vec![
        qtype::STRING as u8, // Type: string
        0x00,                // Attribute: none
        0x05,
        0x00,
        0x00,
        0x00, // Size: 5 bytes
        0xFF,
        0xFE,
        0xFD,
        0xFC,
        0xFB, // Invalid UTF-8 sequence
    ];

    let err = K::q_ipc_decode(&bytes, 1).expect_err("should reject invalid UTF-8");
    assert!(matches!(err, Error::InvalidUtf8));
}

#[test]
fn test_symbol_list_with_invalid_utf8() {
    // Test symbol list with one invalid UTF-8 symbol
    let bytes = vec![
        qtype::SYMBOL_LIST as u8, // Type: symbol list
        0x00,                     // Attribute: none
        0x02,
        0x00,
        0x00,
        0x00, // Size: 2 symbols
        b'o',
        b'k',
        0x00, // First symbol: "ok"
        0xFF,
        0xFE,
        0x00, // Second symbol: invalid UTF-8
    ];

    let err = K::q_ipc_decode(&bytes, 1).expect_err("should reject invalid UTF-8");
    assert!(matches!(err, Error::InvalidUtf8));
}

#[test]
fn test_deeply_nested_compound_list() {
    // Test deeply nested structure - should hit recursion depth limit
    // MAX_RECURSION_DEPTH is 100 by default
    // Note: We need to use a larger stack for this test because debug builds
    // have larger stack frames and test threads have smaller default stacks
    let nesting_depth = 110; // Exceeds the limit

    let handle = std::thread::Builder::new()
        .stack_size(4 * 1024 * 1024) // 4MB stack
        .spawn(move || {
            let mut bytes = Vec::new();

            // Build nested compound lists
            for _ in 0..nesting_depth {
                bytes.push(qtype::COMPOUND_LIST as u8); // Type: compound list
                bytes.push(0x00); // Attribute: none
                bytes.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // Size: 1 element
            }

            // End with simple integer
            bytes.push(qtype::INT_ATOM as u8);
            bytes.extend_from_slice(&[0x2A, 0x00, 0x00, 0x00]); // Value: 42

            match K::q_ipc_decode(&bytes, 1) {
                Ok(_) => panic!("Should have returned MaxDepthExceeded error"),
                Err(e) => assert!(matches!(e, Error::MaxDepthExceeded { .. })),
            }
        })
        .unwrap();

    handle.join().unwrap();
}

#[test]
fn test_extremely_deep_nesting() {
    // Test with extreme nesting - should also hit recursion depth limit
    let nesting_depth = 150; // Way over the 100 limit

    let handle = std::thread::Builder::new()
        .stack_size(4 * 1024 * 1024) // 4MB stack
        .spawn(move || {
            let mut bytes = Vec::new();

            for _ in 0..nesting_depth {
                bytes.push(qtype::COMPOUND_LIST as u8);
                bytes.push(0x00);
                bytes.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
            }

            bytes.push(qtype::INT_ATOM as u8);
            bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

            println!(
                "Testing extremely deep nesting (depth: {})...",
                nesting_depth
            );
            match K::q_ipc_decode(&bytes, 1) {
                Ok(_) => panic!("Should have returned MaxDepthExceeded error"),
                Err(e) => assert!(matches!(e, Error::MaxDepthExceeded { .. })),
            }
        })
        .unwrap();

    handle.join().unwrap();
}

#[test]
fn test_nested_table_in_list() {
    // Test nested structures with tables - should also hit recursion limit
    let nesting_depth = 110; // Over the 100 limit

    let handle = std::thread::Builder::new()
        .stack_size(4 * 1024 * 1024) // 4MB stack
        .spawn(move || {
            let mut bytes = Vec::new();

            for _ in 0..nesting_depth {
                bytes.push(qtype::COMPOUND_LIST as u8);
                bytes.push(0x00);
                bytes.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
            }

            // End with a table
            bytes.push(qtype::TABLE as u8);
            bytes.push(0x00); // Attribute
            bytes.push(qtype::DICTIONARY as u8);
            // Simple empty table structure
            bytes.push(qtype::SYMBOL_LIST as u8); // keys
            bytes.push(0x00);
            bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
            bytes.push(qtype::COMPOUND_LIST as u8); // values
            bytes.push(0x00);
            bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

            match K::q_ipc_decode(&bytes, 1) {
                Ok(_) => panic!("Should have returned MaxDepthExceeded error"),
                Err(e) => assert!(matches!(e, Error::MaxDepthExceeded { .. })),
            }
        })
        .unwrap();

    handle.join().unwrap();
}

#[test]
fn test_reasonable_list_sizes() {
    // Test with reasonable list sizes that should work
    let bytes = vec![
        qtype::LONG_LIST as u8, // Type: long list
        0x00,                   // Attribute: none
        0x0A,
        0x00,
        0x00,
        0x00, // Size: 10 elements
        // 10 * 8 = 80 bytes of data
        0x01,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // 1
        0x02,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // 2
        0x03,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // 3
        0x04,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // 4
        0x05,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // 5
        0x06,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // 6
        0x07,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // 7
        0x08,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // 8
        0x09,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // 9
        0x0A,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // 10
    ];

    let k = K::q_ipc_decode(&bytes, 1).unwrap();
    let list = k.as_vec::<i64>().unwrap();

    assert_eq!(list.len(), 10);
    assert_eq!(list[0], 1);
    assert_eq!(list[9], 10);
}

#[test]
fn test_valid_symbol_list() {
    // Test with valid symbol list
    let bytes = vec![
        qtype::SYMBOL_LIST as u8, // Type: symbol list
        0x00,                     // Attribute: none
        0x03,
        0x00,
        0x00,
        0x00, // Size: 3 symbols
        b'o',
        b'n',
        b'e',
        0x00, // "one"
        b't',
        b'w',
        b'o',
        0x00, // "two"
        b't',
        b'h',
        b'r',
        b'e',
        b'e',
        0x00, // "three"
    ];

    let k = K::q_ipc_decode(&bytes, 1).unwrap();
    let list = k.as_vec::<String>().unwrap();

    assert_eq!(list.len(), 3);
    assert_eq!(list[0], "one");
    assert_eq!(list[1], "two");
    assert_eq!(list[2], "three");
}

#[test]
fn test_moderate_nesting_depth() {
    // Test with moderate nesting that should work
    let nesting_depth = 10;
    let mut bytes = Vec::new();

    for _ in 0..nesting_depth {
        bytes.push(qtype::COMPOUND_LIST as u8);
        bytes.push(0x00);
        bytes.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
    }

    bytes.push(qtype::INT_ATOM as u8);
    bytes.extend_from_slice(&[0x2A, 0x00, 0x00, 0x00]); // 42

    let k = K::q_ipc_decode(&bytes, 1).unwrap();

    // Navigate through nesting to verify
    let mut current = &k;
    for _ in 0..nesting_depth {
        let list = current.as_vec::<K>().unwrap();
        assert_eq!(list.len(), 1);
        current = &list[0];
    }

    assert_eq!(current.get_int().unwrap(), 42);
}
