//! Example demonstrating explicit compression control and validation modes
//!
//! This example shows how to:
//! 1. Use different compression modes (Auto, Always, Never)
//! 2. Use different validation modes (Strict, Lenient)
//! 3. See the effects of these settings on message encoding/decoding

use kdb_codec::*;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

fn main() -> Result<()> {
    println!("=== Compression Control Demo ===\n");

    // Create a large K object that might benefit from compression
    let large_data = K::new_long_list(vec![42; 3000], qattribute::NONE);
    let message = KdbMessage::new(qmsg_type::synchronous, large_data);

    // Test 1: Auto mode with remote connection (should compress)
    println!("1. Auto mode with remote connection:");
    test_compression(false, CompressionMode::Auto, message.clone());

    // Test 2: Auto mode with local connection (should NOT compress)
    println!("\n2. Auto mode with local connection:");
    test_compression(true, CompressionMode::Auto, message.clone());

    // Test 3: Always mode with local connection (should compress)
    println!("\n3. Always mode with local connection:");
    test_compression(true, CompressionMode::Always, message.clone());

    // Test 4: Never mode with remote connection (should NOT compress)
    println!("\n4. Never mode with remote connection:");
    test_compression(false, CompressionMode::Never, message.clone());

    println!("\n=== Validation Mode Demo ===\n");

    // Test 5: Strict validation with invalid header (should reject)
    println!("5. Strict validation with invalid compressed flag:");
    test_validation_strict();

    // Test 6: Lenient validation with invalid header (should accept)
    println!("\n6. Lenient validation with invalid compressed flag:");
    test_validation_lenient();

    Ok(())
}

fn test_compression(is_local: bool, mode: CompressionMode, message: KdbMessage) {
    let mut codec = KdbCodec::with_options(is_local, mode, ValidationMode::Strict);
    let mut buffer = BytesMut::new();

    // Encode the message
    codec.encode(message, &mut buffer).unwrap();

    // Parse the header to check compression
    let header = MessageHeader::from_bytes(&buffer[..MessageHeader::size()]).unwrap();

    println!(
        "  Connection: {}",
        if is_local { "local" } else { "remote" }
    );
    println!("  Compression mode: {:?}", mode);
    println!("  Message size: {} bytes", buffer.len());
    println!(
        "  Compressed: {}",
        if header.compressed == 1 { "YES" } else { "NO" }
    );
}

fn test_validation_strict() {
    let mut codec = KdbCodec::with_options(false, CompressionMode::Never, ValidationMode::Strict);
    let mut buffer = BytesMut::new();

    // Create a message with invalid compressed flag (2)
    buffer.extend_from_slice(&[1, 1, 2, 0]); // encoding=1, msg_type=1, compressed=2 (invalid!)
    buffer.extend_from_slice(&[20, 0, 0, 0]); // length = 20
    buffer.extend_from_slice(&[0; 12]); // dummy payload

    // Try to decode
    let result = codec.decode(&mut buffer);

    match result {
        Ok(_) => println!("  ❌ UNEXPECTED: Strict mode accepted invalid header"),
        Err(e) => println!("  ✅ Correctly rejected: {}", e),
    }
}

fn test_validation_lenient() {
    let mut codec = KdbCodec::with_options(false, CompressionMode::Never, ValidationMode::Lenient);

    // Encode a small valid message first to get proper payload
    let small_message = KdbMessage::new(qmsg_type::synchronous, K::new_int(42));
    let mut temp_buffer = BytesMut::new();
    let mut temp_codec = KdbCodec::new(false);
    temp_codec.encode(small_message, &mut temp_buffer).unwrap();

    // Now create a buffer with "invalid" header values but valid payload
    let mut buffer = BytesMut::new();
    // Use non-standard but harmless values
    buffer.extend_from_slice(&[1, 5, 3, 0]); // msg_type=5, compressed=3 (both "invalid" in strict mode)
                                             // Copy the rest from the valid message (length and payload)
    buffer.extend_from_slice(&temp_buffer[4..]);

    // Try to decode
    let result = codec.decode(&mut buffer);

    match result {
        Ok(Some(msg)) => {
            println!("  ✅ Lenient mode accepted non-standard header");
            println!("     Message type: {}", msg.message_type);
            println!("     Payload value: {}", msg.payload);
        }
        Ok(None) => println!("  ⚠️ Need more data"),
        Err(e) => println!("  ❌ UNEXPECTED: Lenient mode rejected: {}", e),
    }
}
