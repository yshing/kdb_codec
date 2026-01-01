//! Example demonstrating custom security limits for KdbCodec
//!
//! This example shows how to configure custom security limits when creating a KdbCodec instance.
//!
//! ## Default Limits (for `new()` and `with_options()`)
//!
//! The defaults are based on kdb+ database limits:
//! - MAX_LIST_SIZE: 100 million elements (kdb+ supports up to 2^31)
//! - MAX_RECURSION_DEPTH: 100 levels
//! - MAX_MESSAGE_SIZE: 256 MB (kdb+ limit is 2GB)
//! - MAX_DECOMPRESSED_SIZE: 512 MB (protection against compression bombs)
//!
//! Note: When using `builder()`, max_message_size and max_decompressed_size default to `None`
//! (no limit). It's recommended to set these explicitly for untrusted connections.
//!
//! Reference: https://www.timestored.com/kdb-guides/kdb-database-limits

use kdb_codec::*;

fn main() {
    println!("Default security limits:");
    println!("  MAX_LIST_SIZE: {}", MAX_LIST_SIZE);
    println!("  MAX_RECURSION_DEPTH: {}", MAX_RECURSION_DEPTH);
    println!(
        "  MAX_MESSAGE_SIZE: {} MB",
        MAX_MESSAGE_SIZE / (1024 * 1024)
    );
    println!(
        "  MAX_DECOMPRESSED_SIZE: {} MB",
        MAX_DECOMPRESSED_SIZE / (1024 * 1024)
    );
    println!();

    // Create codec with default limits
    let codec_default = KdbCodec::new(false);
    println!("Codec with defaults:");
    println!("  max_list_size: {}", codec_default.max_list_size());
    println!(
        "  max_recursion_depth: {}",
        codec_default.max_recursion_depth()
    );
    println!(
        "  max_message_size: {:?} MB",
        codec_default.max_message_size().map(|s| s / (1024 * 1024))
    );
    println!(
        "  max_decompressed_size: {:?} MB",
        codec_default
            .max_decompressed_size()
            .map(|s| s / (1024 * 1024))
    );
    println!();

    // Create codec with custom limits using builder pattern
    let codec_custom = KdbCodec::builder()
        .is_local(false)
        .compression_mode(CompressionMode::Always)
        .validation_mode(ValidationMode::Strict)
        .max_list_size(50_000_000) // 50M elements instead of 100M default
        .max_recursion_depth(50) // 50 levels instead of 100
        .max_message_size(128 * 1024 * 1024) // 128 MB
        .max_decompressed_size(256 * 1024 * 1024) // 256 MB
        .build();

    println!("Codec with custom limits (builder):");
    println!("  max_list_size: {}", codec_custom.max_list_size());
    println!(
        "  max_recursion_depth: {}",
        codec_custom.max_recursion_depth()
    );
    println!(
        "  max_message_size: {:?} MB",
        codec_custom.max_message_size().map(|s| s / (1024 * 1024))
    );
    println!(
        "  max_decompressed_size: {:?} MB",
        codec_custom
            .max_decompressed_size()
            .map(|s| s / (1024 * 1024))
    );
    println!();

    // You can also modify limits after creation
    let mut codec_mutable = KdbCodec::new(false);
    codec_mutable.set_max_list_size(1_000_000);
    codec_mutable.set_max_recursion_depth(25);
    codec_mutable.set_max_message_size(Some(64 * 1024 * 1024)); // 64 MB
    codec_mutable.set_max_decompressed_size(Some(128 * 1024 * 1024)); // 128 MB

    println!("Codec with modified limits (setters):");
    println!("  max_list_size: {}", codec_mutable.max_list_size());
    println!(
        "  max_recursion_depth: {}",
        codec_mutable.max_recursion_depth()
    );
    println!(
        "  max_message_size: {:?} MB",
        codec_mutable.max_message_size().map(|s| s / (1024 * 1024))
    );
    println!(
        "  max_decompressed_size: {:?} MB",
        codec_mutable
            .max_decompressed_size()
            .map(|s| s / (1024 * 1024))
    );
    println!();

    println!("These limits help protect against:");
    println!("  - Memory exhaustion from excessively large lists");
    println!("  - Stack overflow from deeply nested structures");
    println!("  - Denial of service attacks via malformed messages");
    println!("  - Compression bomb attacks (small compressed → huge decompressed)");
    println!("  - Resource exhaustion from oversized messages");
    println!();
    println!("Note: You can disable size checks by setting to None (not recommended for untrusted connections):");
    let codec_unlimited = KdbCodec::builder().build(); // Using default None values
    println!(
        "  max_message_size: {:?}",
        codec_unlimited.max_message_size()
    );
    println!(
        "  max_decompressed_size: {:?}",
        codec_unlimited.max_decompressed_size()
    );
    println!();
    println!("Note: kdb+ internal limits are:");
    println!("  - Maximum list size: 2^31 elements (2.1 billion)");
    println!("  - Maximum IPC transfer: 2GB object size");
    println!("  See: https://www.timestored.com/kdb-guides/kdb-database-limits");
}
