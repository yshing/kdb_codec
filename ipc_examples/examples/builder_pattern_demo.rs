//! Example demonstrating the builder pattern for KdbCodec and QStream
//!
//! This example shows how to use the builder pattern for a cleaner, more readable API.

use kdb_codec::*;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

fn main() -> Result<()> {
    println!("=== Builder Pattern Demo ===\n");

    // Example 1: KdbCodec builder with all options
    println!("1. KdbCodec builder with full configuration:");
    let codec1 = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Always)
        .validation_mode(ValidationMode::Lenient)
        .build();
    
    println!("   Compression: {:?}", codec1.compression_mode());
    println!("   Validation: {:?}", codec1.validation_mode());

    // Example 2: KdbCodec builder with defaults
    println!("\n2. KdbCodec builder with defaults:");
    let codec2 = KdbCodec::builder().build();
    
    println!("   Compression: {:?} (default)", codec2.compression_mode());
    println!("   Validation: {:?} (default)", codec2.validation_mode());

    // Example 3: KdbCodec builder with partial configuration
    println!("\n3. KdbCodec builder with partial configuration:");
    let codec3 = KdbCodec::builder()
        .compression_mode(CompressionMode::Never)
        .build();
    
    println!("   Compression: {:?}", codec3.compression_mode());
    println!("   Validation: {:?} (default)", codec3.validation_mode());

    // Example 4: Using the codec built with builder pattern
    println!("\n4. Using codec with builder pattern:");
    let mut codec4 = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Auto)
        .build();

    let large_data = K::new_long_list(vec![42; 3000], qattribute::NONE);
    let message = KdbMessage::new(qmsg_type::synchronous, large_data);
    let mut buffer = BytesMut::new();
    
    codec4.encode(message.clone(), &mut buffer).unwrap();
    let header = MessageHeader::from_bytes(&buffer[..MessageHeader::size()]).unwrap();
    
    println!("   Encoded message size: {} bytes", buffer.len());
    println!("   Compressed: {}", if header.compressed == 1 { "YES" } else { "NO" });

    // Example 5: Comparison of different approaches
    println!("\n5. API comparison:");
    
    println!("   Traditional: KdbCodec::new(false)");
    let _trad = KdbCodec::new(false);
    
    println!("   With options: KdbCodec::with_options(false, CompressionMode::Always, ValidationMode::Strict)");
    let _opts = KdbCodec::with_options(false, CompressionMode::Always, ValidationMode::Strict);
    
    println!("   Builder: KdbCodec::builder()");
    println!("              .is_local(false)");
    println!("              .compression_mode(CompressionMode::Always)");
    println!("              .validation_mode(ValidationMode::Strict)");
    println!("              .build()");
    let _builder = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Always)
        .validation_mode(ValidationMode::Strict)
        .build();

    println!("\n✅ Builder pattern provides the most readable and flexible API!");

    Ok(())
}
