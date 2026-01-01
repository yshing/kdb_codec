//! Security Tests: Enum Deserialization
//!
//! Tests for safe decoding of enum atom and enum list types.
//! Enums are decoded conservatively as integer values without symbol table mapping.

use kdb_codec::*;

#[test]
fn test_enum_atom_valid() {
    // Test valid enum atom decode - enum atom is type -20, stores an i32 index
    // Format: type byte + domain name (null-terminated) + value (4 bytes)
    let value: i32 = 42;
    let domain = "sym";
    let mut bytes = vec![qtype::ENUM_ATOM as u8];
    bytes.extend_from_slice(domain.as_bytes());
    bytes.push(0x00); // Null terminator
    bytes.extend_from_slice(&[
        (value & 0xFF) as u8,
        ((value >> 8) & 0xFF) as u8,
        ((value >> 16) & 0xFF) as u8,
        ((value >> 24) & 0xFF) as u8,
    ]);

    let result = K::q_ipc_decode(&bytes, 1);
    assert!(result.is_ok(), "enum atom decode should succeed");
    
    let k = result.unwrap();
    assert_eq!(k.get_type(), qtype::ENUM_ATOM);
    
    // The value should be accessible as an int since enum is stored as i32
    let int_val = k.get_int();
    assert!(int_val.is_ok(), "should be able to extract as int");
    assert_eq!(int_val.unwrap(), value);
}

#[test]
fn test_enum_atom_zero() {
    // Test enum atom with zero value
    let value: i32 = 0;
    let domain = "sym";
    let mut bytes = vec![qtype::ENUM_ATOM as u8];
    bytes.extend_from_slice(domain.as_bytes());
    bytes.push(0x00); // Null terminator
    bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

    let result = K::q_ipc_decode(&bytes, 1);
    assert!(result.is_ok());
    let k = result.unwrap();
    assert_eq!(k.get_int().unwrap(), value);
}

#[test]
fn test_enum_atom_negative() {
    // Test enum atom with negative value (null enum)
    let value: i32 = -2147483648; // i32::MIN
    let domain = "sym";
    let mut bytes = vec![qtype::ENUM_ATOM as u8];
    bytes.extend_from_slice(domain.as_bytes());
    bytes.push(0x00); // Null terminator
    bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x80]); // Little endian i32::MIN

    let result = K::q_ipc_decode(&bytes, 1);
    assert!(result.is_ok());
    let k = result.unwrap();
    assert_eq!(k.get_int().unwrap(), value);
}

#[test]
fn test_enum_atom_truncated() {
    // Test enum atom with insufficient bytes
    let bytes = vec![
        qtype::ENUM_ATOM as u8,
        b's', b'y', b'm', 0x00, // Domain name
        0x01, 0x02, // Only 2 bytes instead of 4
    ];

    let err = K::q_ipc_decode(&bytes, 1).expect_err("should reject truncated enum atom");
    assert!(
        matches!(err, Error::InsufficientData { .. }),
        "expected InsufficientData, got: {err:?}"
    );
}

#[test]
fn test_enum_list_valid_small() {
    // Test valid small enum list - type 20
    // Format: type + attribute + size + domain name (null-terminated) + values
    let size: u32 = 3;
    let domain = "sym";
    let mut bytes = vec![
        qtype::ENUM_LIST as u8,
        0x00, // Attribute: none
        (size & 0xFF) as u8,
        ((size >> 8) & 0xFF) as u8,
        ((size >> 16) & 0xFF) as u8,
        ((size >> 24) & 0xFF) as u8,
    ];
    
    // Add domain name
    bytes.extend_from_slice(domain.as_bytes());
    bytes.push(0x00); // Null terminator
    
    // Add 3 i32 values (12 bytes)
    bytes.extend_from_slice(&[
        0x01, 0x00, 0x00, 0x00, // 1
        0x02, 0x00, 0x00, 0x00, // 2
        0x03, 0x00, 0x00, 0x00, // 3
    ]);

    let result = K::q_ipc_decode(&bytes, 1);
    assert!(result.is_ok(), "enum list decode should succeed");
    
    let k = result.unwrap();
    assert_eq!(k.get_type(), qtype::ENUM_LIST);
    
    // Should be accessible as int list
    let list = k.as_vec::<I>();
    assert!(list.is_ok(), "should be able to extract as i32 vec");
    let values = list.unwrap();
    assert_eq!(values.len(), 3);
    assert_eq!(values[0], 1);
    assert_eq!(values[1], 2);
    assert_eq!(values[2], 3);
}

#[test]
fn test_enum_list_empty() {
    // Test empty enum list
    let _size: u32 = 0;
    let domain = "sym";
    let mut bytes = vec![
        qtype::ENUM_LIST as u8,
        0x00, // Attribute: none
        0x00, 0x00, 0x00, 0x00, // Size: 0
    ];
    bytes.extend_from_slice(domain.as_bytes());
    bytes.push(0x00); // Null terminator

    let result = K::q_ipc_decode(&bytes, 1);
    assert!(result.is_ok());
    let k = result.unwrap();
    assert_eq!(k.get_type(), qtype::ENUM_LIST);
    let list = k.as_vec::<I>().unwrap();
    assert_eq!(list.len(), 0);
}

#[test]
fn test_enum_list_with_sorted_attribute() {
    // Test enum list with sorted attribute
    let size: u32 = 2;
    let domain = "sym";
    let mut bytes = vec![
        qtype::ENUM_LIST as u8,
        qattribute::SORTED as u8,
        (size & 0xFF) as u8,
        ((size >> 8) & 0xFF) as u8,
        ((size >> 16) & 0xFF) as u8,
        ((size >> 24) & 0xFF) as u8,
    ];
    
    bytes.extend_from_slice(domain.as_bytes());
    bytes.push(0x00); // Null terminator
    
    bytes.extend_from_slice(&[
        0x05, 0x00, 0x00, 0x00,
        0x0A, 0x00, 0x00, 0x00,
    ]);

    let result = K::q_ipc_decode(&bytes, 1);
    assert!(result.is_ok());
    let k = result.unwrap();
    assert_eq!(k.get_attribute(), qattribute::SORTED);
}

#[test]
fn test_enum_list_truncated_header() {
    // Test enum list with truncated header
    let bytes = vec![
        qtype::ENUM_LIST as u8,
        0x00, // Attribute
        0x03, 0x00, // Only 2 size bytes instead of 4
    ];

    let err = K::q_ipc_decode(&bytes, 1).expect_err("should reject truncated header");
    assert!(
        matches!(err, Error::InsufficientData { .. }),
        "expected InsufficientData, got: {err:?}"
    );
}

#[test]
fn test_enum_list_truncated_data() {
    // Test enum list with truncated data
    let size: u32 = 5;
    let mut bytes = vec![
        qtype::ENUM_LIST as u8,
        0x00,
        (size & 0xFF) as u8,
        ((size >> 8) & 0xFF) as u8,
        ((size >> 16) & 0xFF) as u8,
        ((size >> 24) & 0xFF) as u8,
    ];
    
    // Only provide 2 i32 values (8 bytes) instead of 5 (20 bytes)
    bytes.extend_from_slice(&[
        0x01, 0x00, 0x00, 0x00,
        0x02, 0x00, 0x00, 0x00,
    ]);

    let err = K::q_ipc_decode(&bytes, 1).expect_err("should reject truncated data");
    assert!(
        matches!(err, Error::InsufficientData { .. }),
        "expected InsufficientData, got: {err:?}"
    );
}

#[test]
fn test_enum_list_oversized() {
    // Test enum list with size exceeding MAX_LIST_SIZE
    let size = (MAX_LIST_SIZE as u32) + 1;
    let bytes = vec![
        qtype::ENUM_LIST as u8,
        0x00,
        (size & 0xFF) as u8,
        ((size >> 8) & 0xFF) as u8,
        ((size >> 16) & 0xFF) as u8,
        ((size >> 24) & 0xFF) as u8,
    ];

    let err = K::q_ipc_decode(&bytes, 1).expect_err("should reject oversized list");
    assert!(
        matches!(err, Error::ListTooLarge { .. }),
        "expected ListTooLarge, got: {err:?}"
    );
}

#[test]
fn test_enum_list_size_overflow() {
    // Test that size multiplication overflow is caught
    // If we have a very large size that would overflow when multiplied by 4
    // We use a size that will definitely exceed MAX_LIST_SIZE
    let size = u32::MAX / 2; // Large enough to be caught by MAX_LIST_SIZE check
    let bytes = vec![
        qtype::ENUM_LIST as u8,
        0x00,
        (size & 0xFF) as u8,
        ((size >> 8) & 0xFF) as u8,
        ((size >> 16) & 0xFF) as u8,
        ((size >> 24) & 0xFF) as u8,
    ];

    let err = K::q_ipc_decode(&bytes, 1).expect_err("should catch size overflow");
    // This should be caught by either ListTooLarge (during size check) or SizeOverflow
    assert!(
        matches!(err, Error::ListTooLarge { .. } | Error::SizeOverflow),
        "expected ListTooLarge or SizeOverflow, got: {err:?}"
    );
}

#[test]
fn test_enum_list_big_endian() {
    // Test enum list with big-endian encoding
    let _size: u32 = 2;
    let domain = "sym";
    let mut bytes = vec![
        qtype::ENUM_LIST as u8,
        0x00,
        0x00, 0x00, 0x00, 0x02, // Big-endian size
    ];
    
    bytes.extend_from_slice(domain.as_bytes());
    bytes.push(0x00); // Null terminator
    
    // Big-endian i32 values
    bytes.extend_from_slice(&[
        0x00, 0x00, 0x00, 0x0A, // 10
        0x00, 0x00, 0x00, 0x14, // 20
    ]);

    let result = K::q_ipc_decode(&bytes, 0); // 0 = big-endian
    assert!(result.is_ok());
    let k = result.unwrap();
    let list = k.as_vec::<I>().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0], 10);
    assert_eq!(list[1], 20);
}
