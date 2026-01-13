//! Security Tests: Decompression Vulnerabilities
//!
//! Tests for decompression bombs, invalid compressed data, and bounds checking

use kdb_codec::codec::decompress_sync;

#[test]
fn test_decompress_insufficient_data() {
    // Compressed data must have at least 4 bytes for size field
    let invalid_data = vec![0x01, 0x02]; // Only 2 bytes

    let result = decompress_sync(invalid_data, 1, None);

    assert!(result.is_err(), "Should return error for insufficient data");
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("need at least 4 bytes"));
}

#[test]
fn test_decompress_negative_size() {
    // Test with negative size when interpreted as i32
    let mut compressed = vec![
        0xFF, 0xFF, 0xFF, 0xFF, // -1 as i32 in little endian
    ];
    // Add some dummy data
    compressed.extend_from_slice(&[0x00; 10]);

    let result = decompress_sync(compressed, 1, None);

    assert!(result.is_err(), "Should return error for negative size");
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("less than minimum"));
}

#[test]
fn test_decompress_size_below_minimum() {
    // Size field indicates 4 bytes (less than 8-byte minimum)
    let compressed = vec![
        0x04, 0x00, 0x00, 0x00, // size_with_header = 4
        0x00, 0x00, 0x00, 0x00,
    ];

    let result = decompress_sync(compressed, 1, None);

    assert!(
        result.is_err(),
        "Should return error for size below minimum"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("less than minimum"));
}

#[test]
fn test_decompression_bomb_large_size() {
    // Test with extremely large decompressed size claim
    let compressed = vec![
        0xFF, 0xFF, 0xFF, 0x7F, // 2,147,483,647 bytes (~2GB) in little endian
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    // Use default max limit (512 MB) to catch decompression bomb
    let max_size = Some(512 * 1024 * 1024);
    let result = decompress_sync(compressed, 1, max_size);

    // Should reject the 2GB decompression request immediately
    assert!(
        result.is_err(),
        "Should reject decompression bomb exceeding max size"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("exceeds maximum allowed size")
            || err_msg.contains("compression bomb"),
        "Error should mention size limit or compression bomb, got: {}",
        err_msg
    );
}

#[test]
fn test_decompression_bomb_compression_ratio() {
    // Test suspicious compression ratio
    // 10 bytes compressed -> claims to decompress to 100KB
    let mut compressed = vec![
        0x00, 0x00, 0x02, 0x00, // 131,072 bytes decompressed (128KB)
    ];
    // Only 10 bytes of actual compressed data
    compressed.extend_from_slice(&[0x00; 10]);

    // Compression ratio: 131,072 / 14 = ~9,362x
    // This is suspiciously high and likely a decompression bomb

    println!("Testing suspicious compression ratio...");
    let result = decompress_sync(compressed, 1, None);

    println!("Suspicious ratio result: {:?}", result.is_ok());

    // Note: This may succeed but be slow. Future: add compression ratio validation
}

#[test]
fn test_decompress_out_of_bounds_read() {
    // Test with valid size header but insufficient compressed data
    let compressed = vec![
        0x20, 0x00, 0x00, 0x00, // Claims 32 bytes decompressed (24 after header)
        0xFF, // Control byte indicating compressed data
        0x10, // Back-reference to position 0x10
              // Missing length byte and data - will read out of bounds
    ];

    println!("Testing out-of-bounds read...");
    let result = decompress_sync(compressed, 1, None);

    // Should return Err about malformed compressed data
    assert!(result.is_err(), "Should detect out-of-bounds read");
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Invalid compressed data"));
}

#[test]
fn test_decompress_invalid_back_reference() {
    // Test with back-reference pointing beyond current position
    let mut compressed = vec![
        0x30, 0x00, 0x00, 0x00, // 48 bytes decompressed (40 after header)
    ];

    // Start with valid decompression
    for _ in 0..10 {
        compressed.push(0x00); // Control byte: literal
        compressed.push(0x41); // Literal 'A'
    }

    // Now add invalid back-reference
    compressed.push(0xFF); // Control byte: all compressed
    compressed.push(0xFF); // Back-reference to position 0xFF (invalid)
    compressed.push(0x05); // Length 5

    println!("Testing invalid back-reference...");
    let result = decompress_sync(compressed, 1, None);

    // May succeed or fail depending on decompressed buffer size
    println!("Invalid back-reference result: {:?}", result.is_ok());
}

#[test]
fn test_decompress_valid_small_data() {
    // Test with valid small compressed data
    let compressed = vec![
        0x10, 0x00, 0x00, 0x00, // 16 bytes total (8 after header)
        0x00, // Control byte: all literal
        0x41, 0x42, 0x43, 0x44, // "ABCD"
        0x45, 0x46, 0x47, 0x48, // "EFGH"
    ];

    let result = decompress_sync(compressed, 1, None);

    // Should succeed for valid data
    assert!(result.is_ok(), "Valid small data should decompress");

    if let Ok(decompressed) = result {
        println!("Decompressed {} bytes", decompressed.len());
        assert_eq!(decompressed.len(), 8);
    }
}

#[test]
fn test_decompress_size_overflow() {
    // Test with size that would overflow when converted
    let compressed = vec![
        0x00, 0x00, 0x00, 0xFF, // Large size in little endian: 4,278,190,080
        0x00, 0x00, 0x00, 0x00,
    ];

    println!("Testing size overflow...");
    let result = decompress_sync(compressed, 1, None);

    // Should handle large sizes gracefully (may fail with allocation error)
    println!("Size overflow result: {:?}", result.is_ok());
}

#[test]
fn test_decompress_empty_compressed_data() {
    // Test with size header indicating 8 bytes (just header, no payload)
    let compressed = vec![
        0x08, 0x00, 0x00, 0x00, // size_with_header = 8 (minimum)
    ];

    let result = decompress_sync(compressed, 1, None);

    // Should return empty decompressed data
    if let Ok(decompressed) = result {
        assert_eq!(decompressed.len(), 0, "Should return empty data");
    }
}

#[test]
fn test_decompress_big_endian() {
    // Test decompression with big endian encoding
    let compressed = vec![
        0x00, 0x00, 0x00, 0x10, // 16 bytes in big endian
        0x00, // Control byte: all literal
        0x41, 0x42, 0x43, 0x44, // "ABCD"
        0x45, 0x46, 0x47, 0x48, // "EFGH"
    ];

    let result = decompress_sync(compressed, 0, None); // encoding = 0 for big endian

    assert!(result.is_ok(), "Big endian decompression should work");

    if let Ok(decompressed) = result {
        assert_eq!(decompressed.len(), 8);
    }
}
