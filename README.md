# kdb_codec - Kdb+ IPC Codec Library

[![Tests](https://github.com/yshing/kdbplus/actions/workflows/test.yml/badge.svg)](https://github.com/yshing/kdbplus/actions/workflows/test.yml)

A Rust library focused on handling the kdb+ IPC (Inter-Process Communication) wire protocol. This library provides efficient encoding, decoding, and communication with q/kdb+ processes using idiomatic Rust patterns.

## Features

- **Tokio Codec Pattern**: Modern async/await interface using `tokio-util::codec`
- **QStream Client**: High-level async client for q/kdb+ communication
- **Full Compression Support**: Compatible with kdb+ `-18!` (compress) and `-19!` (decompress)
- **Multiple Connection Methods**: TCP, TLS, and Unix Domain Socket support
- **Type-Safe**: Strong typing for all kdb+ data types
- **Zero-Copy Operations**: Efficient message handling with minimal allocations

## Rust IPC Interface for q/kdb+

This library provides a Rust client for communicating with q/kdb+ processes. Queries to kdb+ are supported in two ways:

- **Text queries**: Send q code as strings
- **Functional queries**: Represented as compound lists ([IPC details](https://code.kx.com/q4m3/11_IO/#116-interprocess-communication))

Compression/decompression of messages is fully implemented following the [kdb+ specification](https://code.kx.com/q/basics/ipc/#compression).

## Codec Pattern

The library provides a tokio codec implementation for kdb+ IPC communication, offering a cleaner and more idiomatic Rust interface. The codec pattern leverages `tokio-util::codec` traits for efficient message framing and streaming.

**Key Features:**
- ✅ Full compression/decompression support compatible with kdb+ (-18!/-19!)
- ✅ Automatic message framing and buffering
- ✅ Zero-copy operations where possible
- ✅ Type-safe encoder/decoder traits
- ✅ Shared compression implementation with QStream

See [CODEC_PATTERN.md](CODEC_PATTERN.md) for detailed documentation.

**Quick Example:**
```rust
use kdb_codec::ipc::*;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use futures::{SinkExt, StreamExt};

#[tokio::main]
async fn main() -> Result<()> {
    let stream = TcpStream::connect("127.0.0.1:5000").await?;
    let mut framed = Framed::new(stream, KdbCodec::new(true));
    
    // Using feed() + flush() for cancellation safety
    framed.feed(("1+1", qmsg_type::synchronous)).await?;
    framed.flush().await?;
    if let Some(Ok(response)) = framed.next().await {
        println!("Result: {}", response.payload);
    }
    Ok(())
}
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
use kdb_codec::ipc::*;

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
use kdb_codec::ipc::*;

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

### Installation

Add `kdb_codec` to your `Cargo.toml`:

```toml
[dependencies]
kdb_codec = "0.4"
```

The IPC feature is enabled by default.

## Documentation

The full API documentation is available on [docs.rs/kdb_codec](https://docs.rs/kdb_codec/).

For details of the kdb+ IPC protocol, see:

- [kdb+ IPC Reference](https://code.kx.com/q/basics/ipc/)
- [Serialization](https://code.kx.com/q/basics/serialization/)

## License

This library is licensed under Apache-2.0.

See [LICENSE](kdb_codec/LICENSE).
