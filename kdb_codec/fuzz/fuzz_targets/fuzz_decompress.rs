//! Fuzz target for decompress_sync
//! 
//! This fuzzer specifically targets the decompression logic,
//! which is a critical security boundary.

#![no_main]

use libfuzzer_sys::fuzz_target;
use kdb_codec::codec::decompress_sync;

fuzz_target!(|data: &[u8]| {
    // Skip very small inputs that will immediately fail
    if data.len() < 4 {
        return;
    }
    
    // Test with both encodings
    for encoding in [0u8, 1u8] {
        let _ = decompress_sync(data.to_vec(), encoding, None);
    }
});
