# kdb_codec - Kdb+ IPC Codec Library

[![Tests](https://github.com/yshing/kdb_codec/actions/workflows/test.yml/badge.svg)](https://github.com/yshing/kdb_codec/actions/workflows/test.yml)

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
- **Full Compression Support**: Compatible with kdb+ `-18!` (compress) and `-19!` (decompress)
- **Multiple Connection Methods**: TCP, TLS, and Unix Domain Socket support
- **Type-Safe**: Strong typing for all kdb+ data types
- **Minimal Dependencies**: No `async-recursion` or unnecessary proc-macros
- **Zero-Copy Operations**: Efficient message handling with minimal allocations

## Rust IPC Interface for q/kdb+

This library provides a Rust client for communicating with q/kdb+ processes. Queries to kdb+ are supported in two ways:

- **Text queries**: Send q code as strings
- **Functional queries**: Represented as compound lists ([IPC details](https://code.kx.com/q4m3/11_IO/#116-interprocess-communication))

Compression/decompression of messages is fully implemented following the [kdb+ specification](https://code.kx.com/q/basics/ipc/#compression).

## Codec Pattern

The library provides a tokio codec implementation for kdb+ IPC communication, offering a cleaner and more idiomatic Rust interface. The codec pattern leverages `tokio-util::codec` traits for efficient message framing and streaming with **guaranteed cancellation safety**.

**Key Features:**
- ✅ **Cancellation safe** - buffer state preserved across cancellations
- ✅ Full compression/decompression support compatible with kdb+ (-18!/-19!)
- ✅ Automatic message framing and buffering
- ✅ Zero-copy operations where possible
- ✅ Type-safe encoder/decoder traits
- ✅ No `async-recursion` dependency (uses synchronous deserialization)

See [CODEC_PATTERN.md](CODEC_PATTERN.md) for detailed documentation.

**Quick Example:**
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

### Compression Control

The codec provides explicit control over compression behavior:

```rust
use kdb_codec::*;

// Auto mode (default): compress large messages on remote connections only
let codec = KdbCodec::new(false);

// Always compress (if beneficial): compress large messages even on local connections
let codec = KdbCodec::with_options(true, CompressionMode::Always, ValidationMode::Strict);

// Never compress: disable compression entirely
let codec = KdbCodec::with_options(false, CompressionMode::Never, ValidationMode::Strict);
```

**Compression Modes:**
- `Auto` (default): Compress large messages (>2000 bytes) only on remote connections
- `Always`: Attempt to compress large messages even on local connections
- `Never`: Disable compression entirely

### Header Validation

The codec validates incoming message headers to detect protocol violations:

```rust
use kdb_codec::*;

// Strict mode (default): reject invalid headers
let codec = KdbCodec::with_options(false, CompressionMode::Auto, ValidationMode::Strict);

// Lenient mode: accept non-standard header values (for debugging/compatibility)
let codec = KdbCodec::with_options(false, CompressionMode::Auto, ValidationMode::Lenient);
```

**Validation Modes:**
- `Strict` (default): Validates that compressed flag is 0 or 1, and message type is 0, 1, or 2
- `Lenient`: Accepts any header values (useful for debugging or handling non-standard implementations)

### QStream - High-Level Client

For a more convenient API, use `QStream` which wraps the codec:

```rust
use kdb_codec::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut stream = QStream::connect(
        ConnectionMethod::TCP, 
        "localhost", 
        5000, 
        "user:pass"
    ).await?;
    
    // All operations are cancellation safe
    let result = stream.send_sync_message(&"2+2").await?;
    println!("Result: {}", result.get_int()?);
    
    Ok(())
}
```

**With Explicit Options:**

```rust
use kdb_codec::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Connect with always compress and lenient validation
    let mut stream = QStream::connect_with_options(
        ConnectionMethod::TCP, 
        "localhost", 
        5000, 
        "user:pass",
        CompressionMode::Always,
        ValidationMode::Lenient
    ).await?;
    
    let result = stream.send_sync_message(&"2+2").await?;
    println!("Result: {}", result.get_int()?);
    
    Ok(())
}
```

**Tip:** For advanced use cases requiring separate send/receive channels, you can split the underlying Framed stream:

```rust
let stream = TcpStream::connect("127.0.0.1:5000").await?;
let framed = Framed::new(stream, KdbCodec::new(true));
let (mut writer, mut reader) = framed.split();

// Use writer and reader independently
tokio::spawn(async move {
    while let Some(Ok(msg)) = reader.next().await {
        println!("Received: {:?}", msg);
    }
});

writer.send(msg).await?;
```

### Connection Methods

As for connect method, usually client interfaces of q/kdb+ do not provide a listener due to its protocol. However, sometimes Rust process is connecting to an upstream and q/kdb+ starts afterward or is restarted more frequently. Then providing a listener method is a natural direction and it was achieved here. Following ways are supported to connect to kdb+:

- TCP
- TLS
- Unix domain socket

Furthermore, in order to improve inter-operatability some casting, getter and setter methods are provided.

### Environmental Variables

This crate uses q-native or crate-specific environmental variables.

- `KDBPLUS_ACCOUNT_FILE`: A file path to a credential file which an acceptor loads in order to manage access from a q client. This file contains a user name and SHA-1 hashed password in each line which are delimited by `':'` without any space. For example, a file containing two credentials `"mattew:oracle"` and `"reluctant:slowday"` looks like this:

      mattew:431364b6450fc47ccdbf6a2205dfdb1baeb79412
      reluctant:d03f5cc1cdb11a77410ee34e26ca1102e67a893c

      
    The hashed password can be generated with q using a function `.Q.sha1`:
 
      q).Q.sha1 "slowday"
      0xd03f5cc1cdb11a77410ee34e26ca1102e67a893c
 
- `KDBPLUS_TLS_KEY_FILE` and `KDBPLUS_TLS_KEY_FILE_SECRET`: The pkcs12 file and its password which TLS acceptor uses.
- `QUDSPATH` (optional): q-native environmental variable to define an astract namespace. This environmental variable is used by UDS acceptor too. The abstract nameapace will be `@${QUDSPATH}/kx.[server process port]` if this environmental variable is defined; otherwise it will be `@/tmp/kx.[server process port]`.

*Notes:*

- Messages will be sent with OS native endian.
- When using this crate for a TLS client you need to set two environmental variables `KX_SSL_CERT_FILE` and `KX_SSL_KEY_FILE` on q side to make q/kdb+ to work as a TLS server. For details, see [the KX website](https://code.kx.com/q/kb/ssl/).

### Type Mapping

All types are expressed as `K` struct which is quite similar to the `K` struct of `api` module but its structure is optimized for IPC
usage and for the convenience to interact with. The table below shows the input types of each q type which is used to construct `K` object.
Note that the input type can be different from the inner type. For example, timestamp has an input type of `chrono::DateTime<Utc>` but
the inner type is `i64` denoting an elapsed time in nanoseconds since `2000.01.01D00:00:00`.

| q                | Rust                                              |
|------------------|---------------------------------------------------|
| `bool`           | `bool`                                            |
| `GUID`           | `[u8; 16]`                                        |
| `byte`           | `u8`                                              |
| `short`          | `i16`                                             |
| `int`            | `i32`                                             |
| `long`           | `i64`                                             |
| `real`           | `f32`                                             |
| `float`          | `f64`                                             |
| `char`           | `char`                                            |
| `symbol`         | `String`                                          |
| `timestamp`      | `chrono::DateTime<Utc>`                           |
| `month`          | `chrono::NaiveDate`                               |
| `date`           | `chrono::NaiveDate`                               |
| `datetime`       | `chrono::DateTime<Utc>`                           |
| `timespan`       | `chrono::Duration`                                |
| `minute`         | `chrono::Duration`                                |
| `second`         | `chrono::Duration`                                |
| `time`           | `chrono::Duration`                                |
| `list`           | `Vec<Item>` (`Item` is a corrsponding type above) |
| `compound list`  | `Vec<K>`                                          |
| `table`          | `Vec<K>`                                          |
| `dictionary`     | `Vec<K>`                                          |
| `null`           | `()`                                              |
 
### Examples

#### Client

```rust
use kdb_codec::*;

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<()> {

    // Connect to qprocess running on localhost:5000 via UDS
    let mut socket = QStream::connect(ConnectionMethod::UDS, "", 5000_u16, "ideal:person").await?;
    println!("Connection type: {}", socket.get_connection_type());

    // Set remote function with asynchronous message
    socket.send_async_message(&"collatz:{[n] seq:enlist n; while[not n = 1; seq,: n:$[n mod 2; 1 + 3 * n; `long$n % 2]]; seq}").await?;

    // Send a query synchronously
    let mut result = socket.send_sync_message(&"collatz[12]").await?;
    println!("collatz[12]: {}", result);

    result = socket.send_sync_message(&"collatz[`a]").await?;
    println!("collatz[`a]: {}", result);

    // Send a functional query.
    let mut message = K::new_compound_list(vec![K::new_symbol(String::from("collatz")), K::new_long(100)]);
    result = socket.send_sync_message(&message).await?;
    println!("collatz[100]: {}", result);

    // Modify query to (`collatz; 20)
    message.pop().unwrap();
    message.push(&K::new_long(20)).unwrap();
    result=socket.send_sync_message(&message).await?;
    println!("collatz[20]: {}", result);

    // Send a functional asynchronous query.
    message = K::new_compound_list(vec![K::new_string(String::from("show"), qattribute::NONE), K::new_symbol(String::from("goodbye"))]);
    socket.send_async_message(&message).await?;

    socket.shutdown().await?;

    Ok(())
}
```

#### Listener

```rust
use kdb_codec::*;

#[tokio::main]
async fn main() -> Result<()> {

  // Start listenening over TCP at the port 7000 with authentication enabled.
  let mut socket_tcp = QStream::accept(ConnectionMethod::TCP, "127.0.0.1", 7000).await?;

  // Send a query with the socket.
  let greeting = socket_tcp.send_sync_message(&"string `Hello").await?;
  println!("Greeting: {}", greeting);

  socket_tcp.shutdown().await?;

  Ok(())
}
```

Then q client can connect to this acceptor with the acceptor's host, port and the credential configured in `KDBPLUS_ACCOUNT_FILE`:

```q
q)h:hopen `::7000:reluctant:slowday
```

## Architecture & Design

### Cancellation Safety

The core innovation of this library is its use of `tokio-util::codec::Framed` which provides automatic buffer management:

- **Buffer Preservation**: Partial reads are stored in the codec's internal buffer
- **Resumable Operations**: Cancelled reads can be safely retried without data loss
- **No Manual State Management**: The Framed wrapper handles all buffer lifecycle

This is critical for production systems using patterns like:
- `tokio::select!` for timeouts or concurrent operations
- Graceful shutdown with cancellation
- Request racing or fallback logic

### Synchronous Deserialization

Unlike the original kdbplus crate, we use **synchronous deserialization** without `async-recursion`:

- **Simpler**: No async recursion complexity
- **Faster**: Eliminates async overhead for CPU-bound deserialization
- **Smaller**: No `async-recursion` proc-macro dependency
- **Safer**: Avoids potential stack overflow from deep async recursion

The deserialization happens in `deserialize_sync.rs` and is called from the codec's `decode()` method after the complete message is buffered.

### Why Not Add `split()` to QStream?

While we show how to split the underlying Framed stream in examples, we **don't recommend** adding a `split()` method directly to `QStream` because:

1. **Protocol Semantics**: KDB+ IPC is request-response oriented. Splitting would allow sending multiple requests before receiving responses, which can confuse message correlation.

2. **Complexity**: Users would need to manually track which response corresponds to which request.

3. **Better Alternatives**: For concurrent operations, use multiple `QStream` instances or the lower-level `Framed` API directly when you need full control.

If you need independent send/receive channels, access the underlying stream:

```rust
let stream = TcpStream::connect("127.0.0.1:5000").await?;
let framed = Framed::new(stream, KdbCodec::new(true));
let (writer, reader) = framed.split();
// Now you have full control
```

### Installation

Add `kdb_codec` to your `Cargo.toml`:

```toml
[dependencies]
kdb_codec = "0.4"
```

The IPC feature is enabled by default.

## Testing

### Unit Tests

Run the standard unit tests (no kdb+ server required):

```bash
cargo test --package kdb_codec --lib --tests
```

### Integration Tests

Some tests require a running kdb+ server and are marked as `#[ignore]` by default. To run these tests:

1. Start a kdb+ server on `localhost:5001` with credentials `kdbuser:pass`:
   ```bash
   q -p 5001 -u path/to/passwd/file
   ```

2. Run the ignored tests:
   ```bash
   cargo test --package kdb_codec --tests -- --ignored
   ```

The integration tests include:
- `functional_message_test`: Tests various message types and operations
- `compression_test`: Tests compression functionality with large data

**Note**: These tests are automatically skipped in CI/CD unless a kdb+ server is explicitly configured.

## Documentation

The full API documentation is available on [docs.rs/kdb_codec](https://docs.rs/kdb_codec/).

For details of the kdb+ IPC protocol, see:

- [kdb+ IPC Reference](https://code.kx.com/q/basics/ipc/)
- [Serialization](https://code.kx.com/q/basics/serialization/)

## License

This library is licensed under Apache-2.0.

See [LICENSE](kdb_codec/LICENSE).
