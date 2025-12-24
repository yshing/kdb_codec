# KDB-Codec: Tokio Codec Pattern for kdb+ IPC

This document describes the new codec-based architecture for kdb+ IPC communication.

## Overview

The kdb-codec implementation provides a clean, idiomatic Rust interface for kdb+ IPC communication using the tokio ecosystem's codec pattern. This approach leverages `tokio-util::codec` traits to handle message framing, encoding, and decoding in a streaming fashion.

## Architecture

### Core Components

1. **KdbCodec**: The main codec struct implementing `Encoder` and `Decoder` traits
2. **MessageHeader**: Represents the 8-byte kdb+ IPC message header
3. **KdbMessage**: Wrapper for outgoing messages (K object + message type)
4. **KdbResponse**: Wrapper for incoming messages (K object + message type)

### Benefits of the Codec Pattern

- **Cleaner API**: Uses standard Rust streaming abstractions (futures::Sink, futures::Stream)
- **Better Resource Management**: Automatic buffering and backpressure handling
- **Type Safety**: Strong typing for message encoding/decoding
- **Composability**: Easy to layer additional codecs (e.g., for logging, metrics)
- **Testability**: Codecs can be tested independently of I/O

## Usage

### Basic Example

```rust
use kdbplus::ipc::*;
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
    framed.send(("1+1", qmsg_type::synchronous)).await?;
    
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

// Send it
framed.send(query).await?;

// Receive response
if let Some(Ok(response)) = framed.next().await {
    println!("Result: {}", response.payload);
}
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

### Encoding

The `Encoder` trait implementation:
- Serializes K objects using the existing `q_ipc_encode()` method
- Constructs the appropriate message header
- Handles compression for messages > 2000 bytes (when not local)
- Writes the complete message to the output buffer

### Decoding

The `Decoder` trait implementation:
- Reads and parses the 8-byte header
- Waits for complete message arrival
- Handles decompression if needed
- Deserializes the payload using synchronous deserialization
- Returns a `KdbResponse` with message type and K object

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

1. **Compression Support**: Full async compression/decompression in codec
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
