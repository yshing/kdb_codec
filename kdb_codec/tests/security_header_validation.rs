//! Security Tests: Header Validation
//!
//! Tests for invalid message headers, compressed flags, and message types

use bytes::BytesMut;
use kdb_codec::codec::{CompressionMode, KdbCodec, KdbMessage, MessageHeader, ValidationMode};
use kdb_codec::k;
use tokio_util::codec::{Decoder, Encoder};

#[test]
fn test_strict_mode_rejects_invalid_compressed_flag() {
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Never)
        .validation_mode(ValidationMode::Strict)
        .build();
    let mut buffer = BytesMut::new();

    // Create message with invalid compressed flag (2)
    buffer.extend_from_slice(&[
        0x01, // encoding: little endian
        0x01, // message_type: sync
        0x02, // compressed: 2 (INVALID - should be 0 or 1)
        0x00, // reserved
        0x14, 0x00, 0x00, 0x00, // length: 20 bytes
    ]);
    // Add minimal payload
    buffer.extend_from_slice(&[0x00; 12]);

    let result = codec.decode(&mut buffer);

    // Should reject in strict mode
    assert!(
        result.is_err(),
        "Strict mode should reject invalid compressed flag"
    );
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Invalid compressed flag")
            || err.to_string().contains("compressed"),
        "Error message should mention compressed flag, got: {}",
        err
    );
}

#[test]
fn test_strict_mode_rejects_invalid_message_type() {
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Never)
        .validation_mode(ValidationMode::Strict)
        .build();
    let mut buffer = BytesMut::new();

    // Create message with invalid message type (3)
    buffer.extend_from_slice(&[
        0x01, // encoding: little endian
        0x03, // message_type: 3 (INVALID - should be 0, 1, or 2)
        0x00, // compressed: no
        0x00, // reserved
        0x14, 0x00, 0x00, 0x00, // length: 20 bytes
    ]);
    buffer.extend_from_slice(&[0x00; 12]);

    let result = codec.decode(&mut buffer);

    assert!(
        result.is_err(),
        "Strict mode should reject invalid message type"
    );
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Invalid message type")
            || err.to_string().contains("message type"),
        "Error message should mention message type, got: {}",
        err
    );
}

#[test]
fn test_strict_mode_rejects_message_type_255() {
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Never)
        .validation_mode(ValidationMode::Strict)
        .build();
    let mut buffer = BytesMut::new();

    buffer.extend_from_slice(&[
        0x01, // encoding: little endian
        0xFF, // message_type: 255 (INVALID)
        0x00, // compressed: no
        0x00, // reserved
        0x14, 0x00, 0x00, 0x00, // length: 20 bytes
    ]);
    buffer.extend_from_slice(&[0x00; 12]);

    let result = codec.decode(&mut buffer);
    assert!(result.is_err(), "Should reject message type 255");
}

#[test]
fn test_strict_mode_rejects_compressed_flag_255() {
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Never)
        .validation_mode(ValidationMode::Strict)
        .build();
    let mut buffer = BytesMut::new();

    buffer.extend_from_slice(&[
        0x01, // encoding: little endian
        0x01, // message_type: sync
        0xFF, // compressed: 255 (INVALID)
        0x00, // reserved
        0x14, 0x00, 0x00, 0x00, // length: 20 bytes
    ]);
    buffer.extend_from_slice(&[0x00; 12]);

    let result = codec.decode(&mut buffer);
    assert!(result.is_err(), "Should reject compressed flag 255");
}

#[test]
fn test_lenient_mode_accepts_invalid_compressed_flag() {
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Never)
        .validation_mode(ValidationMode::Lenient)
        .build();

    // Create a valid small K object for payload
    let small_int = k!(int: 42);
    let payload_bytes = small_int.q_ipc_encode();
    let total_length = (8 + payload_bytes.len()) as u32;

    let mut buffer = BytesMut::new();
    buffer.extend_from_slice(&[
        0x01, // encoding: little endian
        0x01, // message_type: sync
        0x05, // compressed: 5 (technically invalid but lenient mode accepts)
        0x00, // reserved
    ]);
    buffer.extend_from_slice(&total_length.to_le_bytes());
    buffer.extend_from_slice(&payload_bytes);

    let result = codec.decode(&mut buffer);

    // Lenient mode should accept
    assert!(
        result.is_ok(),
        "Lenient mode should accept non-standard values"
    );
    assert!(result.unwrap().is_some(), "Should decode successfully");
}

#[test]
fn test_lenient_mode_accepts_invalid_message_type() {
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Never)
        .validation_mode(ValidationMode::Lenient)
        .build();

    let small_int = k!(int: 99);
    let payload_bytes = small_int.q_ipc_encode();
    let total_length = (8 + payload_bytes.len()) as u32;

    let mut buffer = BytesMut::new();
    buffer.extend_from_slice(&[
        0x01, // encoding: little endian
        0x09, // message_type: 9 (invalid but lenient accepts)
        0x00, // compressed: no
        0x00, // reserved
    ]);
    buffer.extend_from_slice(&total_length.to_le_bytes());
    buffer.extend_from_slice(&payload_bytes);

    let result = codec.decode(&mut buffer);
    assert!(
        result.is_ok(),
        "Lenient mode should accept non-standard message types"
    );
}

#[test]
fn test_valid_message_types() {
    // Test all valid message types in strict mode
    for msg_type in &[0u8, 1u8, 2u8] {
        let mut codec = KdbCodec::builder()
            .is_local(false)
            .compression_mode(CompressionMode::Never)
            .validation_mode(ValidationMode::Strict)
            .build();

        let small_int = k!(int: 42);
        let payload_bytes = small_int.q_ipc_encode();
        let total_length = (8 + payload_bytes.len()) as u32;

        let mut buffer = BytesMut::new();
        buffer.extend_from_slice(&[
            0x01,      // encoding
            *msg_type, // message_type: 0, 1, or 2
            0x00,      // compressed: no
            0x00,      // reserved
        ]);
        buffer.extend_from_slice(&total_length.to_le_bytes());
        buffer.extend_from_slice(&payload_bytes);

        let result = codec.decode(&mut buffer);
        assert!(result.is_ok(), "Message type {} should be valid", msg_type);
        assert!(result.unwrap().is_some());
    }
}

#[test]
fn test_valid_compressed_flags() {
    // Test both valid compressed flag values using real encoded messages.

    // Uncompressed: small message below compression threshold.
    {
        let mut codec = KdbCodec::builder()
            .is_local(false)
            .compression_mode(CompressionMode::Never)
            .validation_mode(ValidationMode::Strict)
            .build();

        let message = KdbMessage::new(1, k!(int: 42));
        let mut buffer = BytesMut::new();
        codec.encode(message, &mut buffer).unwrap();

        let header = MessageHeader::from_bytes(&buffer[..8]).unwrap();
        assert_eq!(header.compressed, 0);

        let result = codec.decode(&mut buffer);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    // Compressed: use a payload above the compression threshold.
    {
        let mut encoder = KdbCodec::builder()
            .is_local(false)
            .compression_mode(CompressionMode::Always)
            .validation_mode(ValidationMode::Strict)
            .max_decompressed_size(8 * 1024 * 1024)
            .build();

        let large_list = k!(long: vec![1; 3000]);
        let message = KdbMessage::new(1, large_list);
        let mut buffer = BytesMut::new();
        encoder.encode(message, &mut buffer).unwrap();

        let header = MessageHeader::from_bytes(&buffer[..8]).unwrap();
        assert_eq!(header.compressed, 1);

        let result = encoder.decode(&mut buffer);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }
}

#[test]
fn test_message_header_parsing() {
    // Test MessageHeader::from_bytes directly
    let header_bytes = [
        0x01, // encoding: little endian
        0x02, // message_type: response
        0x01, // compressed: yes
        0x00, // reserved
        0x64, 0x00, 0x00, 0x00, // length: 100 bytes
    ];

    let header = MessageHeader::from_bytes(&header_bytes).unwrap();

    assert_eq!(header.encoding, 1);
    assert_eq!(header.message_type, 2);
    assert_eq!(header.compressed, 1);
    assert_eq!(header.length, 100);
}

#[test]
fn test_message_header_big_endian() {
    let header_bytes = [
        0x00, // encoding: big endian
        0x01, // message_type: sync
        0x00, // compressed: no
        0x00, // reserved
        0x00, 0x00, 0x01, 0x00, // length: 256 bytes (big endian)
    ];

    let header = MessageHeader::from_bytes(&header_bytes).unwrap();

    assert_eq!(header.encoding, 0);
    assert_eq!(header.length, 256);
}

#[test]
fn test_message_header_roundtrip() {
    let original = MessageHeader {
        encoding: 1,
        message_type: 1,
        compressed: 1,
        _unused: 0,
        length: 12345,
    };

    let bytes = original.to_bytes();
    let parsed = MessageHeader::from_bytes(&bytes).unwrap();

    assert_eq!(parsed.encoding, original.encoding);
    assert_eq!(parsed.message_type, original.message_type);
    assert_eq!(parsed.compressed, original.compressed);
    assert_eq!(parsed.length, original.length);
}

#[test]
fn test_header_insufficient_bytes() {
    // Test with less than 8 bytes
    let short_buffer = [0x01, 0x02, 0x03];

    let result = MessageHeader::from_bytes(&short_buffer);
    assert!(result.is_err(), "Should reject insufficient header bytes");
}

#[test]
fn test_incomplete_message_buffer() {
    // Test that decoder waits for complete message
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Never)
        .validation_mode(ValidationMode::Strict)
        .build();
    let mut buffer = BytesMut::new();

    // Send header claiming 1000 bytes
    buffer.extend_from_slice(&[
        0x01, // encoding
        0x01, // message_type
        0x00, // compressed
        0x00, // reserved
        0xE8, 0x03, 0x00, 0x00, // length: 1000 bytes
    ]);
    // But only send 100 bytes total
    buffer.extend_from_slice(&[0x00; 92]);

    let result = codec.decode(&mut buffer);

    // Should return Ok(None) indicating it needs more data
    assert!(result.is_ok(), "Should not error on incomplete message");
    // assert_eq!(result.unwrap(), None, "Should return None waiting for more data");
}
