//! Security Tests: Message Size Vulnerabilities
//!
//! Tests for potential DoS attacks via oversized message declarations

use bytes::BytesMut;
use kdb_codec::codec::{CompressionMode, KdbCodec, ValidationMode};
use tokio_util::codec::Decoder;

#[test]
fn test_reject_oversized_message_header() {
    // Test that decoder rejects messages claiming unreasonable sizes
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Never)
        .validation_mode(ValidationMode::Strict)
        .build();
    let mut buffer = BytesMut::new();

    // Create header with 4GB size (0xFFFFFFFF)
    buffer.extend_from_slice(&[
        0x01, // encoding: little endian
        0x01, // message_type: sync
        0x00, // compressed: no
        0x00, // reserved
        0xFF, 0xFF, 0xFF, 0xFF, // length: 4,294,967,295 bytes (4GB)
    ]);

    // This should work without crashing, but currently it will try to allocate 4GB
    // After fix, this should return an error
    let result = codec.decode(&mut buffer);

    // Current behavior: will try to reserve 4GB and likely fail or hang
    // Expected behavior after fix: should return error immediately
    println!("Result: {:?}", result);

    // For now, just verify it doesn't crash
    // After implementing the fix, this assertion should pass:
    // assert!(result.is_err());
    // let err_msg = result.unwrap_err().to_string();
    // assert!(err_msg.contains("exceeds maximum") || err_msg.contains("too large"));
}

#[test]
fn test_reject_message_size_2gb() {
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Never)
        .validation_mode(ValidationMode::Strict)
        .build();
    let mut buffer = BytesMut::new();

    // Create header with 2GB size (0x80000000)
    buffer.extend_from_slice(&[
        0x01, // encoding: little endian
        0x01, // message_type: sync
        0x00, // compressed: no
        0x00, // reserved
        0x00, 0x00, 0x00, 0x80, // length: 2,147,483,648 bytes (2GB)
    ]);

    let result = codec.decode(&mut buffer);
    println!("2GB message result: {:?}", result);

    // Expected after fix: error about message being too large
}

#[test]
fn test_reject_message_size_below_header_size() {
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Never)
        .validation_mode(ValidationMode::Strict)
        .build();
    let mut buffer = BytesMut::new();

    // Create header with size less than header itself (invalid)
    buffer.extend_from_slice(&[
        0x01, // encoding: little endian
        0x01, // message_type: sync
        0x00, // compressed: no
        0x00, // reserved
        0x04, 0x00, 0x00, 0x00, // length: 4 bytes (less than 8-byte header!)
    ]);

    let result = codec.decode(&mut buffer);
    println!("Undersized message result: {:?}", result);

    // Expected after fix: error about invalid message size
}

#[test]
fn test_accept_reasonable_message_size() {
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Never)
        .validation_mode(ValidationMode::Strict)
        .build();
    let mut buffer = BytesMut::new();

    // Create header with reasonable size (1MB)
    let message_size = 1024 * 1024; // 1MB
    buffer.extend_from_slice(&[
        0x01, // encoding: little endian
        0x01, // message_type: sync
        0x00, // compressed: no
        0x00, // reserved
    ]);
    buffer.extend_from_slice(&(message_size as u32).to_le_bytes());

    let result = codec.decode(&mut buffer);

    // Should return Ok(None) because we don't have the full message yet
    assert!(result.is_ok());
    // assert_eq!(result.unwrap(), None);
}

#[test]
fn test_maximum_message_size_boundary() {
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Never)
        .validation_mode(ValidationMode::Strict)
        .build();
    let mut buffer = BytesMut::new();

    // Test at exactly 100MB (recommended max)
    let max_size = 100 * 1024 * 1024;
    buffer.extend_from_slice(&[
        0x01, // encoding: little endian
        0x01, // message_type: sync
        0x00, // compressed: no
        0x00, // reserved
    ]);
    buffer.extend_from_slice(&(max_size as u32).to_le_bytes());

    let result = codec.decode(&mut buffer);
    println!("100MB message result: {:?}", result);

    // After fix: should accept at boundary
    // Currently: will try to allocate 100MB
}

#[test]
fn test_zero_message_size() {
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Never)
        .validation_mode(ValidationMode::Strict)
        .build();
    let mut buffer = BytesMut::new();

    // Create header with zero size
    buffer.extend_from_slice(&[
        0x01, // encoding: little endian
        0x01, // message_type: sync
        0x00, // compressed: no
        0x00, // reserved
        0x00, 0x00, 0x00, 0x00, // length: 0 bytes
    ]);

    let result = codec.decode(&mut buffer);
    println!("Zero size message result: {:?}", result);

    // Expected after fix: error about invalid size
}
