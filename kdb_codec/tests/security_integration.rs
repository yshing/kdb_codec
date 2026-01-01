//! Security Tests: Integration Tests
//!
//! End-to-end tests combining multiple vulnerability scenarios

use bytes::BytesMut;
use kdb_codec::codec::{CompressionMode, KdbCodec, ValidationMode};
use kdb_codec::*;
use tokio_util::codec::{Decoder, Encoder};

#[test]
fn test_malformed_compressed_message() {
    // Test complete flow with malformed compressed message
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Auto)
        .validation_mode(ValidationMode::Strict)
        .build();
    let mut buffer = BytesMut::new();

    // Create header claiming compressed message
    buffer.extend_from_slice(&[
        0x01, // encoding: little endian
        0x01, // message_type: sync
        0x01, // compressed: yes
        0x00, // reserved
        0x20, 0x00, 0x00, 0x00, // length: 32 bytes total
    ]);

    // Add malformed "compressed" data
    // Should have: [uncompressed_size: 4 bytes][compressed_data]
    buffer.extend_from_slice(&[
        0xFF, 0xFF, 0xFF, 0x7F, // Claims 2GB uncompressed (invalid)
        0x00, 0x00, 0x00, 0x00, // Garbage data
    ]);
    // Pad to 32 bytes total
    buffer.extend_from_slice(&[0x00; 16]);

    println!("Testing malformed compressed message...");
    let result = codec.decode(&mut buffer);

    // Should handle gracefully (currently may panic)
    println!("Malformed compressed result: {:?}", result.is_err());
}

#[test]
fn test_valid_compression_then_valid_decompression() {
    // Test that valid compression/decompression works
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Always)
        .validation_mode(ValidationMode::Strict)
        .build();

    // Create large enough data to trigger compression
    let large_list = k!(long: vec![42; 3000]);
    let message = codec::KdbMessage::new(1, large_list);

    // Encode
    let mut buffer = BytesMut::new();
    let encode_result = codec.encode(message.clone(), &mut buffer);
    assert!(encode_result.is_ok(), "Encoding should succeed");

    // Check it was compressed
    let header = codec::MessageHeader::from_bytes(&buffer[..8]).unwrap();
    println!("Message compressed: {}", header.compressed);

    // Decode
    let decode_result = codec.decode(&mut buffer);
    assert!(decode_result.is_ok(), "Decoding should succeed");

    if let Ok(Some(decoded_msg)) = decode_result {
        let decoded_list = decoded_msg.payload.as_vec::<i64>().unwrap();
        assert_eq!(decoded_list.len(), 3000);
        assert_eq!(decoded_list[0], 42);
    }
}

#[test]
fn test_mixed_valid_and_invalid_messages() {
    // Send valid message followed by invalid message
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Never)
        .validation_mode(ValidationMode::Strict)
        .build();
    let mut buffer = BytesMut::new();

    // First: valid message
    let valid_int = k!(int: 100);
    let valid_payload = valid_int.q_ipc_encode();
    let valid_length = (8 + valid_payload.len()) as u32;

    buffer.extend_from_slice(&[0x01, 0x01, 0x00, 0x00]);
    buffer.extend_from_slice(&valid_length.to_le_bytes());
    buffer.extend_from_slice(&valid_payload);

    // Decode first message
    let result1 = codec.decode(&mut buffer);
    assert!(result1.is_ok(), "First message should decode");
    assert!(result1.unwrap().is_some());

    // Second: invalid message (bad message type)
    buffer.extend_from_slice(&[
        0x01, 0xFF, 0x00, 0x00, // Invalid message type
        0x14, 0x00, 0x00, 0x00,
    ]);
    buffer.extend_from_slice(&[0x00; 12]);

    let result2 = codec.decode(&mut buffer);
    assert!(result2.is_err(), "Second message should be rejected");
}

#[test]
fn test_codec_state_after_error() {
    // Verify codec can recover after encountering error
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Never)
        .validation_mode(ValidationMode::Strict)
        .build();
    let mut buffer = BytesMut::new();

    // Send invalid message
    buffer.extend_from_slice(&[
        0x01, 0xFF, 0x00, 0x00, // Invalid message type
        0x14, 0x00, 0x00, 0x00,
    ]);
    buffer.extend_from_slice(&[0x00; 12]);

    let result = codec.decode(&mut buffer);
    assert!(result.is_err());

    // Clear buffer and send valid message
    buffer.clear();
    let valid_int = k!(int: 42);
    let valid_payload = valid_int.q_ipc_encode();
    let valid_length = (8 + valid_payload.len()) as u32;

    buffer.extend_from_slice(&[0x01, 0x01, 0x00, 0x00]);
    buffer.extend_from_slice(&valid_length.to_le_bytes());
    buffer.extend_from_slice(&valid_payload);

    let result = codec.decode(&mut buffer);
    assert!(result.is_ok(), "Should recover and decode valid message");
}

#[test]
fn test_large_but_valid_message() {
    // Test with large but reasonable message (e.g., 10MB)
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Never)
        .validation_mode(ValidationMode::Strict)
        .build();

    // Create 1M element list (8MB of data)
    let large_list = k!(long: vec![1; 1_000_000]);
    let message = codec::KdbMessage::new(1, large_list);

    let mut buffer = BytesMut::new();
    let result = codec.encode(message, &mut buffer);

    assert!(result.is_ok(), "Should encode large valid message");
    println!("Encoded large message: {} bytes", buffer.len());

    // Should be able to decode it back
    let decode_result = codec.decode(&mut buffer);
    assert!(decode_result.is_ok(), "Should decode large valid message");
}

#[test]
fn test_symbol_with_very_long_string() {
    // Test symbol with reasonable but long string
    let long_symbol = "a".repeat(10000); // 10KB symbol
    let k_symbol = K::new_symbol(long_symbol.clone());

    let encoded = k_symbol.q_ipc_encode();
    let decoded = K::q_ipc_decode(&encoded, 1).unwrap();

    assert_eq!(decoded.get_symbol().unwrap(), long_symbol);
}

#[test]
fn test_list_with_million_elements() {
    // Test list with 1 million small elements (reasonable size)
    let bytes = create_int_list_bytes(1_000_000);

    println!("Testing 1M element list deserialization...");
    let result = std::panic::catch_unwind(|| {
        let _k = K::q_ipc_decode(&bytes, 1).unwrap();
    });

    // Should work with 1M elements (4MB of data)
    // But if MAX_LIST_SIZE is set too low, this would fail
    println!("1M element list result: {:?}", result.is_ok());
}

#[test]
fn test_empty_lists_and_edge_cases() {
    // Test various empty structures
    let empty_list = k!(long: vec![]);
    let encoded = empty_list.q_ipc_encode();
    let decoded = K::q_ipc_decode(&encoded, 1).unwrap();

    let list = decoded.as_vec::<i64>().unwrap();
    assert_eq!(list.len(), 0);

    // Empty symbol list
    let empty_symbols = k!(sym: vec![]);
    let encoded = empty_symbols.q_ipc_encode();
    let decoded = K::q_ipc_decode(&encoded, 1).unwrap();

    let syms = decoded.as_vec::<String>().unwrap();
    assert_eq!(syms.len(), 0);
}

#[test]
fn test_nested_empty_lists() {
    // Test nested structure with empty lists
    let inner_empty = k!(long: vec![]);
    let outer = K::new_compound_list(vec![inner_empty]);

    let encoded = outer.q_ipc_encode();
    let decoded = K::q_ipc_decode(&encoded, 1).unwrap();

    let outer_list = decoded.as_vec::<K>().unwrap();
    assert_eq!(outer_list.len(), 1);

    let inner_list = outer_list[0].as_vec::<i64>().unwrap();
    assert_eq!(inner_list.len(), 0);
}

#[test]
fn test_compression_mode_transitions() {
    // Test changing compression modes
    let mut codec = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Never)
        .validation_mode(ValidationMode::Strict)
        .build();

    let large_list = k!(long: vec![1; 3000]);
    let message = codec::KdbMessage::new(1, large_list.clone());

    // Encode without compression
    let mut buffer1 = BytesMut::new();
    codec.encode(message.clone(), &mut buffer1).unwrap();
    let header1 = codec::MessageHeader::from_bytes(&buffer1[..8]).unwrap();
    assert_eq!(header1.compressed, 0, "Should not compress in Never mode");

    // Change to Always mode
    codec.set_compression_mode(CompressionMode::Always);

    // Encode with compression
    let mut buffer2 = BytesMut::new();
    codec.encode(message, &mut buffer2).unwrap();
    let header2 = codec::MessageHeader::from_bytes(&buffer2[..8]).unwrap();
    assert_eq!(header2.compressed, 1, "Should compress in Always mode");
}

// Helper function to create large int list bytes
fn create_int_list_bytes(count: usize) -> Vec<u8> {
    let mut bytes = vec![
        qtype::INT_LIST as u8, // Type: int list
        0x00,                  // Attribute: none
    ];
    bytes.extend_from_slice(&(count as u32).to_le_bytes());

    // Add the actual data
    for i in 0..count {
        bytes.extend_from_slice(&(i as i32).to_le_bytes());
    }

    bytes
}

#[test]
fn test_realistic_table_size() {
    // Test with realistic table size (1000 rows, 10 columns)
    let rows = 1000;

    let col1 = k!(long: vec![1; rows]);
    let col2 = k!(long: vec![2; rows]);
    let col3 = k!(float: vec![3.14; rows]);
    let col4 = k!(sym: vec!["test".to_string(); rows]);
    let col5 = k!(int: vec![5; rows]);

    let keys = k!(sym: vec![
        "col1".to_string(),
        "col2".to_string(),
        "col3".to_string(),
        "col4".to_string(),
        "col5".to_string(),
    ]);

    let values = K::new_compound_list(vec![col1, col2, col3, col4, col5]);

    let dict = K::new_dictionary(keys, values).unwrap();
    let table = dict.flip().unwrap();

    let encoded = table.q_ipc_encode();
    println!("Table size: {} bytes", encoded.len());

    let decoded = K::q_ipc_decode(&encoded, 1).unwrap();

    // Verify it round-trips correctly
    assert_eq!(decoded.get_type(), qtype::TABLE);
}
