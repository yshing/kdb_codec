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
        .max_message_size(1024)
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

    let result = codec.decode(&mut buffer);
    assert!(result.is_err(), "should reject oversized message header");
}

#[test]
fn test_reject_message_size_2gb() {
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Never)
        .validation_mode(ValidationMode::Strict)
        .max_message_size(1024)
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
    assert!(result.is_err(), "should reject oversized message header");
}

#[test]
fn test_reject_message_size_below_header_size() {
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Never)
        .validation_mode(ValidationMode::Strict)
        .max_message_size(1024)
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
    assert!(result.is_err(), "should reject message smaller than header");
}

#[test]
fn test_accept_reasonable_message_size() {
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Never)
        .validation_mode(ValidationMode::Strict)
        .max_message_size(1024)
        .build();
    let mut buffer = BytesMut::new();

    // Create header with reasonable size (1KB) to avoid large allocations in tests
    let message_size = 1024;
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
    assert!(result.unwrap().is_none());
}

#[test]
fn test_maximum_message_size_boundary() {
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Never)
        .validation_mode(ValidationMode::Strict)
        .max_message_size(1024)
        .build();
    let mut buffer = BytesMut::new();

    // Test at exactly the configured maximum (1KB)
    let max_size = 1024;
    buffer.extend_from_slice(&[
        0x01, // encoding: little endian
        0x01, // message_type: sync
        0x00, // compressed: no
        0x00, // reserved
    ]);
    buffer.extend_from_slice(&(max_size as u32).to_le_bytes());

    let result = codec.decode(&mut buffer);
    assert!(result.is_ok(), "should accept at boundary");
}

#[test]
fn test_zero_message_size() {
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Never)
        .validation_mode(ValidationMode::Strict)
        .max_message_size(1024)
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
    assert!(result.is_err(), "should reject zero-length message");
}
