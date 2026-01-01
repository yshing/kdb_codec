# KDB-Codec: Tokio Codec Pattern for kdb+ IPC

This document describes the new codec-based architecture for kdb+ IPC communication.

## Overview

The kdb-codec implementation provides a clean, idiomatic Rust interface for kdb+ IPC communication using the tokio ecosystem's codec pattern. This approach leverages `tokio-util::codec` traits to handle message framing, encoding, and decoding in a streaming fashion.

## Architecture

### Core Components

1. **KdbCodec**: The main codec struct implementing `Encoder` and `Decoder` traits
2. **MessageHeader**: Represents the 8-byte kdb+ IPC message header
3. **KdbMessage**: Wrapper for outgoing messages (K object + message type)

### Benefits of the Codec Pattern

- **Cleaner API**: Uses standard Rust streaming abstractions (futures::Sink, futures::Stream)
- **Better Resource Management**: Automatic buffering and backpressure handling
- **Type Safety**: Strong typing for message encoding/decoding
- **Composability**: Easy to layer additional codecs (e.g., for logging, metrics)
- **Testability**: Codecs can be tested independently of I/O

## Usage

### Basic Example

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

### Sending K Objects

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

### Cancellation Safety

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

### Forwarding Messages from Channels

When receiving messages from a channel and forwarding them to kdb+, you have two approaches:

#### Approach 1: Using split() to avoid select! (Recommended)

Split the framed stream into separate read and write halves. This eliminates the need for `tokio::select!` and simplifies the code:

```rust
use tokio::sync::mpsc;
use futures::{SinkExt, StreamExt};

async fn forward_with_split(
    mut rx: mpsc::Receiver<KdbMessage>,
    framed: Framed<TcpStream, KdbCodec>,
) -> Result<()> {
    // Split into independent sink (write) and stream (read)
    let (mut sink, mut stream) = framed.split();
    
    // Spawn task to handle responses
    let response_handle = tokio::spawn(async move {
        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => println!("Response: {}", response.payload),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    });
    
    // Forward messages from channel to kdb+ (no select! needed)
    while let Some(msg) = rx.recv().await {
        sink.feed(msg).await?;
        sink.flush().await?;
    }
    
    // Wait for response handler to finish
    let _ = response_handle.await;
    Ok(())
}
```

**Benefits:**
- ✅ No `tokio::select!` complexity
- ✅ Cleaner separation of concerns
- ✅ Can process responses independently
- ✅ More composable and easier to test

#### Approach 2: Using select! (When needed)

Use this when you need to coordinate between receiving messages and other operations:

```rust
async fn forward_with_select(
    mut rx: mpsc::Receiver<KdbMessage>,
    mut framed: Framed<TcpStream, KdbCodec>,
) -> Result<()> {
    loop {
        tokio::select! {
            // Receive message from channel
            Some(msg) = rx.recv() => {
                // feed() buffers the message without sending it yet
                // If this select! is cancelled, msg is lost but wasn't sent
                framed.feed(msg).await?;
                
                // flush() actually sends the buffered messages
                // Only call this after successfully feeding
                framed.flush().await?;
                
                // At this point, the message is guaranteed to be sent
                // If cancellation happens before flush(), the message is buffered
                // and will be sent on the next flush()
            }
            else => break, // Channel closed
        }
    }
    Ok(())
}
```

**Key Points:**
- ✅ **No double-send**: Each `msg` from `rx.recv()` is fed and flushed exactly once
- ✅ **No message loss after feed()**: If `feed()` succeeds, the message is buffered and will be sent on the next `flush()`
- ⚠️ **Cancellation before feed()**: If `select!` cancels before `feed()` completes, `msg` is lost (but never sent)
- ✅ **Separation of concerns**: `feed()` buffers, `flush()` sends - this makes the flow explicit

**Avoiding double-send:**
```rust
// ❌ WRONG: Don't call feed() again without flush() in between
framed.feed(msg1).await?;
framed.feed(msg2).await?; // Both buffered
framed.flush().await?;     // Both sent together

// ✅ CORRECT: Feed and flush each message
framed.feed(msg1).await?;
framed.flush().await?;     // msg1 sent
framed.feed(msg2).await?;
framed.flush().await?;     // msg2 sent
```

## Message Format

The kdb+ IPC protocol uses an 8-byte header:

```
Byte 0: Encoding (0=Big Endian, 1=Little Endian)
Byte 1: Message Type (0=Async, 1=Sync, 2=Response)
Byte 2: Compressed (0=No, 1=Yes)
Byte 3: Reserved (0x00)
Bytes 4-7: Total message length (including header)
```

## Codec Implementation Details

### Shared Compression Functions

The codec module provides public `compress_sync` and `decompress_sync` functions that implement the kdb+ IPC compression algorithm. These functions are:
- Used internally by the `KdbCodec` for encoding/decoding
- Also used by the traditional `QStream` API to avoid code duplication
- Fully compatible with kdb+ `-18!` (compress) and `-19!` (decompress) functions
- Implemented as synchronous functions for optimal performance in the codec pattern

### Encoding

The `Encoder` trait implementation:
- Serializes K objects using the existing `q_ipc_encode()` method
- Constructs the appropriate message header
- **Handles compression for messages > 2000 bytes (when not local)**
- Writes the complete message to the output buffer

**Compression Logic:**
- Messages larger than 2000 bytes and on non-local connections are automatically compressed
- The codec uses synchronous compression for efficiency within the codec pattern
- If compression doesn't reduce size to less than half, the original uncompressed message is sent

### Decoding

The `Decoder` trait implementation:
- Reads and parses the 8-byte header
- Waits for complete message arrival
- **Handles decompression automatically when the compressed flag is set**
- Deserializes the payload using synchronous deserialization
- Returns a `KdbMessage` with message type and K object

**Decompression Logic:**
- Automatically detects compressed messages via the header flag
- Uses synchronous decompression for efficient processing
- Fully compatible with kdb+ compression format (-18!/-19!)

## Migration from QStream

The existing `QStream` API continues to work. The codec pattern is an alternative approach that can be used alongside it:

**Traditional QStream:**
```rust
let mut socket = QStream::connect(ConnectionMethod::TCP, "localhost", 5000, "user:pass").await?;
let result = socket.send_sync_message(&"1+1").await?;
```

**Codec Pattern:**
```rust
let stream = TcpStream::connect("127.0.0.1:5000").await?;
let mut framed = Framed::new(stream, KdbCodec::new(true));
framed.send(("1+1", qmsg_type::synchronous)).await?;
let result = framed.next().await.unwrap()?.payload;
```

## Future Enhancements

Potential improvements to the codec implementation:

1. ~~**Compression Support**: Full async compression/decompression in codec~~ ✅ **Implemented**
2. **Connection Pooling**: Codec-based connection pool
3. **Metrics Codec**: Wrapper codec for automatic metrics collection
4. **Retry Logic**: Codec wrapper for automatic retry on failure
5. **TLS Support**: Native TLS codec integration

## Performance Considerations

The codec pattern:
- Minimizes copies through use of `bytes::BytesMut`
- Provides zero-copy message framing where possible
- Allows efficient streaming of multiple messages
- Supports backpressure through futures::Sink/Stream

## Testing

The codec can be tested independently:

```rust
use bytes::BytesMut;

#[test]
fn test_encode_decode() {
    let codec = KdbCodec::new(true);
    let message = KdbMessage::new(qmsg_type::synchronous, K::new_long(42));
    
    let mut buf = BytesMut::new();
    codec.encode(message, &mut buf).unwrap();
    
    let decoded = codec.decode(&mut buf).unwrap().unwrap();
    assert_eq!(decoded.payload.get_long().unwrap(), 42);
}
```

## References

- [Tokio Codec Documentation](https://docs.rs/tokio-util/latest/tokio_util/codec/)
- [kdb+ IPC Protocol](https://code.kx.com/q/basics/ipc/)
- [Bytes Crate](https://docs.rs/bytes/)
