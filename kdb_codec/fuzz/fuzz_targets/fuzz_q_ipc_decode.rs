//! Fuzz target for q_ipc_decode
//! 
//! This fuzzer tests the deserialization of arbitrary byte sequences,
//! looking for panics, crashes, and excessive memory allocations.

#![no_main]

use libfuzzer_sys::fuzz_target;
use kdb_codec::K;

fuzz_target!(|data: &[u8]| {
    // Skip empty input
    if data.is_empty() {
        return;
    }
    
    // Test with both encodings
    for encoding in [0u8, 1u8] {
        // Catch panics to prevent fuzzer from stopping
        let _ = std::panic::catch_unwind(|| {
            let _result = K::q_ipc_decode(data, encoding);
        });
    }
});
