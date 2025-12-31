# Codec Pattern

The kdb-codec implementation provides a clean, idiomatic Rust interface for kdb+ IPC communication using the tokio ecosystem's codec pattern. This approach leverages `tokio-util::codec` traits to handle message framing, encoding, and decoding in a streaming fashion.

## Core Components

1. **KdbCodec**: The main codec struct implementing `Encoder` and `Decoder` traits
2. **MessageHeader**: Represents the 8-byte kdb+ IPC message header
3. **KdbMessage**: Wrapper for outgoing messages (K object + message type)
4. **KdbResponse**: Wrapper for incoming messages (K object + message type)

## Benefits of the Codec Pattern

- **Cleaner API**: Uses standard Rust streaming abstractions (futures::Sink, futures::Stream)
- **Better Resource Management**: Automatic buffering and backpressure handling
- **Type Safety**: Strong typing for message encoding/decoding
- **Composability**: Easy to layer additional codecs (e.g., for logging, metrics)
- **Testability**: Codecs can be tested independently of I/O

## Basic Example

```rust
use kdb_codec::*;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use futures::{SinkExt, StreamExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to kdb+ process
    let stream = TcpStream::connect("127.0.0.1:5000").await?;
    
    // Wrap with codec
    let codec = KdbCodec::new(true); // true = local connection
    let mut framed = Framed::new(stream, codec);
    
    // Send a text query
    // Note: Using feed() + flush() is cancellation-safe, unlike send()
    framed.feed(("1+1", qmsg_type::synchronous)).await?;
    framed.flush().await?;
    
    // Receive response
    if let Some(Ok(response)) = framed.next().await {
        println!("Result: {}", response.payload);
    }
    
    Ok(())
}
```

## Sending K Objects

```rust
// Create a functional query
let query = KdbMessage::new(
    qmsg_type::synchronous,
    K::new_compound_list(vec![
        K::new_symbol(String::from("til")),
        K::new_long(10),
    ])
);

// Send it using feed() + flush() for cancellation safety
framed.feed(query).await?;
framed.flush().await?;

// Receive response
if let Some(Ok(response)) = framed.next().await {
    println!("Result: {}", response.payload);
}
```

## Cancellation Safety

When using the codec in contexts where cancellation is possible (e.g., `tokio::select!`), prefer `feed()` + `flush()` over `send()`:

```rust
use futures::SinkExt;

// ❌ NOT cancellation-safe: message can be lost if select! cancels
tokio::select! {
    _ = framed.send(message) => {},
    _ = timeout => {},
}

// ✅ Cancellation-safe: uses feed() which doesn't lose messages on cancellation
tokio::select! {
    _ = async {
        framed.feed(message).await?;
        framed.flush().await
    } => {},
    _ = timeout => {},
}
```

## Compression Control

The codec provides explicit control over compression behavior:

```rust
use kdb_codec::*;

// Auto mode (default): compress large messages on remote connections only
let codec = KdbCodec::new(false);

// Using with_options method
let codec = KdbCodec::with_options(true, CompressionMode::Always, ValidationMode::Strict);

// Using builder pattern (recommended)
let codec = KdbCodec::builder()
    .is_local(false)
    .compression_mode(CompressionMode::Never)
    .validation_mode(ValidationMode::Strict)
    .build();
```

**Compression Modes:**
- `Auto` (default): Compress large messages (>2000 bytes) only on remote connections
- `Always`: Attempt to compress messages larger than 2000 bytes even on local connections
- `Never`: Disable compression entirely

## Header Validation

The codec validates incoming message headers to detect protocol violations:

```rust
use kdb_codec::*;

// Strict mode (default): reject invalid headers
let codec = KdbCodec::with_options(false, CompressionMode::Auto, ValidationMode::Strict);

// Using builder pattern
let codec = KdbCodec::builder()
    .validation_mode(ValidationMode::Lenient)
    .build();
```

**Validation Modes:**
- `Strict` (default): Validates that compressed flag is 0 or 1, and message type is 0, 1, or 2
- `Lenient`: Accepts any header values (useful for debugging or handling non-standard implementations)

## Message Format

The kdb+ IPC protocol uses an 8-byte header:

```
Byte 0: Encoding (0=Big Endian, 1=Little Endian)
Byte 1: Message Type (0=Async, 1=Sync, 2=Response)
Byte 2: Compressed (0=No, 1=Yes)
Byte 3: Reserved (0x00)
Bytes 4-7: Total message length (including header)
```
