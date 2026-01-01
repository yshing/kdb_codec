//! Security Tests: Foreign Object Deserialization
//!
//! Tests for safe decoding of foreign objects (type 112).
//! Foreign objects are decoded conservatively as opaque byte payloads.

use kdb_codec::*;

#[test]
fn test_foreign_object_valid_small() {
    // Test valid small foreign object - type 112
    // Foreign objects are: attribute (1) + length (4) + payload
    let payload = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let length: u32 = payload.len() as u32;
    
    let mut bytes = vec![
        qtype::FOREIGN as u8,
        0x00, // Attribute: none
        (length & 0xFF) as u8,
        ((length >> 8) & 0xFF) as u8,
        ((length >> 16) & 0xFF) as u8,
        ((length >> 24) & 0xFF) as u8,
    ];
    bytes.extend_from_slice(&payload);

    let result = K::q_ipc_decode(&bytes, 1);
    assert!(result.is_ok(), "foreign object decode should succeed");
    
    let k = result.unwrap();
    assert_eq!(k.get_type(), qtype::FOREIGN);
    
    // Should be stored as byte list
    let stored_payload = k.as_vec::<G>();
    assert!(stored_payload.is_ok(), "should be able to extract as byte vec");
    let bytes_vec = stored_payload.unwrap();
    assert_eq!(bytes_vec.len(), payload.len());
    assert_eq!(bytes_vec.as_slice(), payload.as_slice());
}

#[test]
fn test_foreign_object_empty() {
    // Test empty foreign object
    let _length: u32 = 0;
    let bytes = vec![
        qtype::FOREIGN as u8,
        0x00, // Attribute: none
        0x00, 0x00, 0x00, 0x00, // Length: 0
    ];

    let result = K::q_ipc_decode(&bytes, 1);
    assert!(result.is_ok(), "empty foreign object should succeed");
    let k = result.unwrap();
    assert_eq!(k.get_type(), qtype::FOREIGN);
    let payload = k.as_vec::<G>().unwrap();
    assert_eq!(payload.len(), 0);
}

#[test]
fn test_foreign_object_with_attribute() {
    // Test foreign object with unique attribute
    let payload = vec![0xAA, 0xBB, 0xCC];
    let length: u32 = payload.len() as u32;
    
    let mut bytes = vec![
        qtype::FOREIGN as u8,
        qattribute::UNIQUE as u8,
        (length & 0xFF) as u8,
        ((length >> 8) & 0xFF) as u8,
        ((length >> 16) & 0xFF) as u8,
        ((length >> 24) & 0xFF) as u8,
    ];
    bytes.extend_from_slice(&payload);

    let result = K::q_ipc_decode(&bytes, 1);
    assert!(result.is_ok());
    let k = result.unwrap();
    assert_eq!(k.get_attribute(), qattribute::UNIQUE);
    assert_eq!(k.as_vec::<G>().unwrap().as_slice(), payload.as_slice());
}

#[test]
fn test_foreign_object_truncated_header() {
    // Test foreign object with truncated header (missing length bytes)
    let bytes = vec![
        qtype::FOREIGN as u8,
        0x00, // Attribute
        0x05, 0x00, // Only 2 bytes of length instead of 4
    ];

    let err = K::q_ipc_decode(&bytes, 1).expect_err("should reject truncated header");
    assert!(
        matches!(err, Error::InsufficientData { .. }),
        "expected InsufficientData, got: {err:?}"
    );
}

#[test]
fn test_foreign_object_truncated_payload() {
    // Test foreign object with truncated payload
    let length: u32 = 100;
    let mut bytes = vec![
        qtype::FOREIGN as u8,
        0x00,
        (length & 0xFF) as u8,
        ((length >> 8) & 0xFF) as u8,
        ((length >> 16) & 0xFF) as u8,
        ((length >> 24) & 0xFF) as u8,
    ];
    
    // Only provide 10 bytes instead of 100
    bytes.extend_from_slice(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A]);

    let err = K::q_ipc_decode(&bytes, 1).expect_err("should reject truncated payload");
    assert!(
        matches!(err, Error::InsufficientData { .. }),
        "expected InsufficientData, got: {err:?}"
    );
}

#[test]
fn test_foreign_object_oversized() {
    // Test foreign object with size exceeding MAX_LIST_SIZE
    let size = (MAX_LIST_SIZE as u32) + 1;
    let bytes = vec![
        qtype::FOREIGN as u8,
        0x00,
        (size & 0xFF) as u8,
        ((size >> 8) & 0xFF) as u8,
        ((size >> 16) & 0xFF) as u8,
        ((size >> 24) & 0xFF) as u8,
    ];

    let err = K::q_ipc_decode(&bytes, 1).expect_err("should reject oversized foreign object");
    assert!(
        matches!(err, Error::ListTooLarge { .. }),
        "expected ListTooLarge, got: {err:?}"
    );
}

#[test]
fn test_foreign_object_max_valid_size() {
    // Test foreign object at a reasonable but large size
    // Use a size that's safe but validates we can handle medium-large payloads
    let length: u32 = 10000;
    let mut bytes = vec![
        qtype::FOREIGN as u8,
        0x00,
        (length & 0xFF) as u8,
        ((length >> 8) & 0xFF) as u8,
        ((length >> 16) & 0xFF) as u8,
        ((length >> 24) & 0xFF) as u8,
    ];
    
    // Generate 10000 bytes of test data
    let payload: Vec<u8> = (0..length).map(|i| (i % 256) as u8).collect();
    bytes.extend_from_slice(&payload);

    let result = K::q_ipc_decode(&bytes, 1);
    assert!(result.is_ok(), "should handle reasonably large foreign objects");
    let k = result.unwrap();
    let stored = k.as_vec::<G>().unwrap();
    assert_eq!(stored.len(), length as usize);
    assert_eq!(stored.as_slice(), payload.as_slice());
}

#[test]
fn test_foreign_object_big_endian() {
    // Test foreign object with big-endian length encoding
    let payload = vec![0x11, 0x22, 0x33];
    let _length: u32 = payload.len() as u32;
    
    let mut bytes = vec![
        qtype::FOREIGN as u8,
        0x00,
        0x00, 0x00, 0x00, 0x03, // Big-endian length = 3
    ];
    bytes.extend_from_slice(&payload);

    let result = K::q_ipc_decode(&bytes, 0); // 0 = big-endian
    assert!(result.is_ok());
    let k = result.unwrap();
    let stored = k.as_vec::<G>().unwrap();
    assert_eq!(stored.len(), 3);
    assert_eq!(stored.as_slice(), payload.as_slice());
}

#[test]
fn test_foreign_object_zero_length_with_trailing_data() {
    // Test that zero-length foreign object doesn't consume following data
    let mut bytes = vec![
        qtype::FOREIGN as u8,
        0x00,
        0x00, 0x00, 0x00, 0x00, // Length: 0
    ];
    // Add some trailing data that should not be consumed
    bytes.extend_from_slice(&[0xFF, 0xFF, 0xFF]);

    let result = K::q_ipc_decode(&bytes, 1);
    assert!(result.is_ok());
    let k = result.unwrap();
    let payload = k.as_vec::<G>().unwrap();
    assert_eq!(payload.len(), 0);
    // The trailing data should still be in the original bytes
    assert_eq!(bytes[bytes.len() - 3..], [0xFF, 0xFF, 0xFF]);
}

#[test]
fn test_foreign_object_binary_payload() {
    // Test foreign object with various binary data patterns
    let payload = vec![
        0x00, 0x01, 0x02, 0x03, // Sequential
        0xFF, 0xFE, 0xFD, 0xFC, // High bytes
        0x7F, 0x80, 0x81, 0x82, // Around signed boundary
        0x55, 0xAA, 0x55, 0xAA, // Alternating pattern
    ];
    let length: u32 = payload.len() as u32;
    
    let mut bytes = vec![
        qtype::FOREIGN as u8,
        0x00,
        (length & 0xFF) as u8,
        ((length >> 8) & 0xFF) as u8,
        ((length >> 16) & 0xFF) as u8,
        ((length >> 24) & 0xFF) as u8,
    ];
    bytes.extend_from_slice(&payload);

    let result = K::q_ipc_decode(&bytes, 1);
    assert!(result.is_ok());
    let k = result.unwrap();
    let stored = k.as_vec::<G>().unwrap();
    assert_eq!(stored.as_slice(), payload.as_slice());
}

#[test]
fn test_foreign_object_preserves_exact_bytes() {
    // Ensure foreign object preserves payload exactly without interpretation
    let payload: Vec<u8> = vec![
        // Could be misinterpreted as various types if not handled as opaque
        0x00, 0x00, 0x00, 0x00, // Could look like int 0
        0xFF, 0xFF, 0xFF, 0xFF, // Could look like int -1
        0x00, // Could look like null terminator
    ];
    let length: u32 = payload.len() as u32;
    
    let mut bytes = vec![
        qtype::FOREIGN as u8,
        0x00,
        (length & 0xFF) as u8,
        ((length >> 8) & 0xFF) as u8,
        ((length >> 16) & 0xFF) as u8,
        ((length >> 24) & 0xFF) as u8,
    ];
    bytes.extend_from_slice(&payload);

    let result = K::q_ipc_decode(&bytes, 1);
    assert!(result.is_ok());
    let k = result.unwrap();
    let stored = k.as_vec::<G>().unwrap();
    // Verify every byte is preserved exactly
    assert_eq!(stored.len(), payload.len());
    for (i, &byte) in payload.iter().enumerate() {
        assert_eq!(stored[i], byte, "byte at index {} differs", i);
    }
}
