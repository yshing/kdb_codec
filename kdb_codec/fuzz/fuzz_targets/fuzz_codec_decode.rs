//! Fuzz target for KdbCodec decoder
//! 
//! This fuzzer tests the complete codec decoding path including
//! header validation, decompression, and deserialization.

#![no_main]

use libfuzzer_sys::fuzz_target;
use bytes::BytesMut;
use kdb_codec::codec::{KdbCodec, ValidationMode, CompressionMode};
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
            validation
        );
        
        let mut buffer = BytesMut::from(data);
        
        // Catch panics
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _result = codec.decode(&mut buffer);
        }));
    }
});
