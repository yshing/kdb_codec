# QStream Client

`QStream` provides a high-level async client interface for communicating with q/kdb+ processes. It wraps the lower-level codec pattern and provides convenient methods for sending queries and receiving responses.

## Connection Methods

QStream supports multiple connection methods:

- **TCP**: Standard TCP connection
- **TLS**: Encrypted TLS connection
- **Unix Domain Socket (UDS)**: Local Unix socket connection

## Basic Usage

### Connecting to kdb+

```rust
use kdb_codec::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to qprocess running on localhost:5000
    let mut socket = QStream::connect(
        ConnectionMethod::TCP, 
        "localhost", 
        5000, 
        "user:pass"
    ).await?;
    
    println!("Connection type: {}", socket.get_connection_type());

    // Send a query
    let result = socket.send_sync_message(&"1+1").await?;
    println!("Result: {}", result);

    socket.shutdown().await?;
    Ok(())
}
```

### Builder Pattern (Recommended)

```rust
use kdb_codec::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut stream = QStream::builder()
        .method(ConnectionMethod::TCP)
        .host("localhost")
        .port(5000)
        .credential("user:pass")
        .compression_mode(CompressionMode::Always)
        .validation_mode(ValidationMode::Lenient)
        .connect()
        .await?;
    
    let result = stream.send_sync_message(&"2+2").await?;
    println!("Result: {}", result.get_int()?);
    
    Ok(())
}
```

## Sending Messages

### Synchronous Messages

Synchronous messages wait for a response from the kdb+ server:

```rust
// Text query
let result = socket.send_sync_message(&"til 10").await?;
println!("Result: {}", result);

// Functional query
let message = K::new_compound_list(vec![
    K::new_symbol(String::from("collatz")), 
    K::new_long(100)
]);
let result = socket.send_sync_message(&message).await?;
println!("Result: {}", result);
```

### Asynchronous Messages

Asynchronous messages are fire-and-forget (no response expected):

```rust
// Define a remote function
socket.send_async_message(&"collatz:{[n] seq:enlist n; while[not n = 1; seq,: n:$[n mod 2; 1 + 3 * n; `long$n % 2]]; seq}").await?;

// Send a functional async query
let message = K::new_compound_list(vec![
    K::new_string(String::from("show"), qattribute::NONE), 
    K::new_symbol(String::from("goodbye"))
]);
socket.send_async_message(&message).await?;
```

## Listener Mode

You can also accept connections from q/kdb+ clients:

```rust
use kdb_codec::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Start listening over TCP at port 7000 with authentication
    let mut socket_tcp = QStream::accept(ConnectionMethod::TCP, "127.0.0.1", 7000).await?;

    // Send a query with the socket
    let greeting = socket_tcp.send_sync_message(&"string `Hello").await?;
    println!("Greeting: {}", greeting);

    socket_tcp.shutdown().await?;
    Ok(())
}
```

Then q clients can connect:

```
q)h:hopen `::7000:reluctant:slowday
```

## Environmental Variables

### Authentication

`KDBPLUS_ACCOUNT_FILE`: Path to a credential file for authentication. The file contains username and SHA-1 hashed password pairs:

```
matthew:431364b6450fc47ccdbf6a2205dfdb1baeb79412
reluctant:d03f5cc1cdb11a77410ee34e26ca1102e67a893c
```

Generate hashed passwords in q:
```
q).Q.sha1 "slowday"
0xd03f5cc1cdb11a77410ee34e26ca1102e67a893c
```

### TLS Configuration

- `KDBPLUS_TLS_KEY_FILE`: The pkcs12 file for TLS acceptor
- `KDBPLUS_TLS_KEY_FILE_SECRET`: Password for the pkcs12 file
- `KX_SSL_CERT_FILE` and `KX_SSL_KEY_FILE`: Required on the q side for TLS server

### Unix Domain Socket

`QUDSPATH` (optional): Defines the abstract namespace for UDS connections:
- If set: `@${QUDSPATH}/kx.[port]`
- If not set: `@/tmp/kx.[port]`

## Type Mapping

| q Type       | Rust Type                    |
|--------------|------------------------------|
| `bool`       | `bool`                       |
| `GUID`       | `[u8; 16]`                   |
| `byte`       | `u8`                         |
| `short`      | `i16`                        |
| `int`        | `i32`                        |
| `long`       | `i64`                        |
| `real`       | `f32`                        |
| `float`      | `f64`                        |
| `char`       | `char`                       |
| `symbol`     | `String`                     |
| `timestamp`  | `chrono::DateTime<Utc>`      |
| `month`      | `chrono::NaiveDate`          |
| `date`       | `chrono::NaiveDate`          |
| `datetime`   | `chrono::DateTime<Utc>`      |
| `timespan`   | `chrono::Duration`           |
| `minute`     | `chrono::Duration`           |
| `second`     | `chrono::Duration`           |
| `time`       | `chrono::Duration`           |
| `list`       | `Vec<Item>`                  |
| `compound`   | `Vec<K>`                     |
| `table`      | `Vec<K>`                     |
| `dictionary` | `Vec<K>`                     |
| `null`       | `()`                         |
