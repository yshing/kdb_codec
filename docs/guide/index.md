# kdb_codec

A Rust library focused on handling the kdb+ IPC (Inter-Process Communication) wire protocol. This library provides efficient encoding, decoding, and communication with q/kdb+ processes using idiomatic Rust patterns.

**Inspired by the original [kdbplus](https://crates.io/crates/kdbplus) crate**, this library addresses critical **cancellation safety** issues while maintaining full compatibility with the kdb+ IPC protocol.

## Why This Library?

The original kdbplus crate had a fundamental cancellation safety issue in its `receive_message()` implementation. When used with `tokio::select!` or other cancellation-aware patterns, partial reads could cause message corruption:

```rust
// ⚠️ UNSAFE - could lose data on cancellation in original kdbplus
select! {
    msg = socket.receive_message() => { /* ... */ }
    _ = timeout => { /* partial read gets lost */ }
}
```

**Our Solution:** This library uses `tokio-util::codec::Framed` with a custom `KdbCodec`, ensuring true cancellation safety:

```rust
// ✅ SAFE - Framed maintains buffer state across cancellations
let mut framed = Framed::new(stream, KdbCodec::new(true));
select! {
    msg = framed.next() => { /* buffer state preserved */ }
    _ = timeout => { /* can safely retry */ }
}
```

The Framed pattern maintains internal buffer state, so cancelled reads never lose data. All partial reads are preserved in the codec's buffer and properly reassembled on the next attempt.

## Features

- **Cancellation Safe**: Built on `tokio-util::codec::Framed` for true cancellation safety
- **Tokio Codec Pattern**: Modern async/await interface with proper buffer management
- **QStream Client**: High-level async client for q/kdb+ communication
- **Intuitive Data Access**: Index trait for ergonomic K object access with `[]` syntax
- **Full Compression Support**: Compatible with kdb+ `-18!` (compress) and `-19!` (decompress)
- **Multiple Connection Methods**: TCP, TLS, and Unix Domain Socket support
- **Type-Safe**: Strong typing for all kdb+ data types
- **Minimal Dependencies**: No `async-recursion` or unnecessary proc-macros
- **Zero-Copy Operations**: Efficient message handling with minimal allocations

## Quick Example

```rust
use kdb_codec::*;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use futures::{SinkExt, StreamExt};

#[tokio::main]
async fn main() -> Result<()> {
    let stream = TcpStream::connect("127.0.0.1:5000").await?;
    let mut framed = Framed::new(stream, KdbCodec::new(true));
    
    // Send query - cancellation safe!
    let query = K::new_string("1+1".to_string(), 0);
    let msg = KdbMessage::new(qmsg_type::synchronous, query);
    framed.send(msg).await?;
    
    // Receive response - even if cancelled, buffer state is preserved
    if let Some(Ok(response)) = framed.next().await {
        println!("Result: {}", response.payload);
    }
    Ok(())
}
```

## Next Steps

- [Installation](/guide/installation) - Add kdb_codec to your project
- [Codec Pattern](/guide/codec-pattern) - Learn about the tokio codec architecture
- [K Macro](/guide/k-macro) - Simplified data construction
- [Index Trait](/guide/index-trait) - Ergonomic data access with [] syntax
