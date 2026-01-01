# kdb_codec - Kdb+ IPC Codec Library

[![Tests](https://github.com/yshing/kdb_codec/actions/workflows/test.yml/badge.svg)](https://github.com/yshing/kdb_codec/actions/workflows/test.yml)

Cancellation-safe Rust codec + client for the kdb+ IPC wire protocol (q/kdb+).

Docs: https://yshing.github.io/kdb_codec/

## Key features

- Cancellation-safe message framing via `tokio-util::codec::Framed`
- IPC encode/decode, including kdb+ compression (`-18!`/`-19!`)
- Async client API (`QStream`) and lower-level codec API (`KdbCodec`)
- Multiple connection methods: TCP / TLS / Unix Domain Socket
- Ergonomic `K` value type for building/inspecting q objects

## Quick start

```rust
use futures::{SinkExt, StreamExt};
use kdb_codec::*;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

#[tokio::main]
async fn main() -> Result<()> {
    let stream = TcpStream::connect("127.0.0.1:5000").await?;
    let mut framed = Framed::new(stream, KdbCodec::new(true));

    let query = K::new_string("1+1".to_string(), 0);
    framed
        .send(KdbMessage::new(qmsg_type::synchronous, query))
        .await?;

    if let Some(Ok(resp)) = framed.next().await {
        println!("{}", resp.payload);
    }
    Ok(())
}
```

## Datatype coverage (IPC)

This project prioritizes safety when decoding untrusted IPC bytes (no panics, no OOM).

- Supported: basic atoms/lists (0–19), mixed lists, table (98), dictionary (99/127), null (101), error (-128)
- Not supported: enums (20–76), nested/other types (77+), function/derived types (100–112), foreign (112)
let mut dict = k!(dict: k!(sym: vec!["x"]) => k!(long: vec![42]));
dict[1] = k!(long: vec![100]);  // Replace values
```

### Table Column Access

Access table columns by name using string indices:

```rust
use kdb_codec::*;

// Create a table
let table = k!(table: {
    "fruit" => k!(sym: vec!["apple", "banana", "cherry"]),
    "price" => k!(float: vec![1.5, 2.3, 3.8]),
    "quantity" => k!(long: vec![100, 150, 75])
});

// Access columns by name
let fruits = &table["fruit"];
let prices = &table["price"];
let quantities = &table["quantity"];

println!("Fruits: {}", fruits);        // `apple`banana`cherry
println!("Prices: {}", prices);        // 1.5 2.3 3.8
println!("Quantities: {}", quantities); // 100 150 75

// Mutable access
let mut table = k!(table: {
    "price" => k!(float: vec![1.5, 2.3])
});
table["price"] = k!(float: vec![2.0, 2.5]);  // Update prices
```

### Safe Access Methods

For production code, use the safe `try_*` methods that return `Result` instead of panicking:

```rust
use kdb_codec::*;

let dict = k!(dict: k!(sym: vec!["x", "y"]) => k!(long: vec![10, 20]));

// Safe dictionary access
match dict.try_index(0) {
    Ok(keys) => println!("Keys: {}", keys),
    Err(e) => eprintln!("Error: {:?}", e),
}

// Try accessing out of bounds - won't panic
if dict.try_index(2).is_err() {
    println!("Index 2 is out of bounds");
}

// Safe table column access
let table = k!(table: {
    "name" => k!(sym: vec!["Alice", "Bob"])
});

match table.try_column("name") {
    Ok(col) => println!("Names: {}", col),
    Err(_) => println!("Column not found"),
}

// Check if column exists before accessing
if table.try_column("nonexistent").is_err() {
    println!("Column 'nonexistent' not found");
}
```

### Compound List Access

Access elements in compound (heterogeneous) lists:

```rust
use kdb_codec::*;

let list = k!([
    k!(long: 100),
    k!(float: 3.14),
    k!(sym: "hello"),
    k!(bool: vec![true, false, true])
]);

// Safe access to list elements
if let Ok(first) = list.try_index(0) {
    println!("First element: {}", first);  // 100
}

if let Ok(second) = list.try_index(1) {
    println!("Second element: {}", second); // 3.14
}
```

**Benefits:**
- ✅ Ergonomic `[]` syntax familiar to Rust developers
- ✅ Type-safe with compile-time borrow checking
- ✅ Both panicking (`[]`) and safe (`try_*`) variants available
- ✅ Works seamlessly with mutable access
- ✅ Supports dictionaries, tables, and compound lists

See `examples/index_trait_demo.rs` for more examples.

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
