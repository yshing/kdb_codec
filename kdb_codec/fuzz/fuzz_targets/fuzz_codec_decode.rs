//! Fuzz target for KdbCodec decoder
//! 
//! This fuzzer tests the complete codec decoding path including
//! header validation, decompression, and deserialization.

#![no_main]

use libfuzzer_sys::fuzz_target;
use bytes::BytesMut;
use kdb_codec::codec::{KdbCodec, ValidationMode, CompressionMode};
use kdb_codec::{MAX_LIST_SIZE, MAX_RECURSION_DEPTH};
use tokio_util::codec::Decoder;

fuzz_target!(|data: &[u8]| {
    // Skip very small inputs
    if data.len() < 8 {
        return;
    }
    
    // Test with both validation modes
    for validation in [ValidationMode::Strict, ValidationMode::Lenient] {
        let mut codec = KdbCodec::with_options(
            false,
            CompressionMode::Auto,
            validation,
            MAX_LIST_SIZE,
            MAX_RECURSION_DEPTH,
        );
        
        let mut buffer = BytesMut::from(data);
        
        // Now returns Result, no need to catch panics
        let _ = codec.decode(&mut buffer);
    }
});
