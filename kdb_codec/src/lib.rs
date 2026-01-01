//! # kdb_codec - Kdb+ IPC Codec Library
//!
//! This library provides a codec for handling the kdb+ IPC (Inter-Process Communication) wire protocol.
//! It focuses on encoding and decoding kdb+ data types for communication with q/kdb+ processes.
//!
//! ## Features
//!
//! - **IPC Codec**: Tokio-based codec implementation for kdb+ IPC protocol
//! - **QStream**: High-level async client for connecting to q/kdb+ processes  
//! - **Compression Control**: Explicit control over compression behavior (Auto, Always, Never)
//! - **Header Validation**: Configurable validation strictness for incoming messages
//! - **Type Safety**: Strong typing for kdb+ data types
//! - **Multiple Connection Methods**: TCP, TLS, and Unix Domain Socket support
//!
//! ## Security Constants
//!
//! The library provides default limits to prevent attacks, which can be customized
//! via the `KdbCodec` builder or configuration methods:
//! - `MAX_LIST_SIZE`: 100 million elements (default for max_list_size)
//! - `MAX_RECURSION_DEPTH`: 100 levels (default for max_recursion_depth)
//! - `MAX_MESSAGE_SIZE`: 256 MB (default for max_message_size)
//! - `MAX_DECOMPRESSED_SIZE`: 512 MB (default for max_decompressed_size)
//!
//! These defaults are based on kdb+ database limits documented at:
//! https://www.timestored.com/kdb-guides/kdb-database-limits
//!
//! These can be adjusted per codec instance using:
//! ```no_run
//! use kdb_codec::*;
//!
//! let codec = KdbCodec::builder()
//!     .max_list_size(5_000_000)  // Custom limit
//!     .max_recursion_depth(50)    // Custom depth
//!     .build();
//! ```
//!
//! ## Usage
//!
//! ### Basic Example
//!
//! ```no_run
//! use kdb_codec::*;
//! use tokio::net::TcpStream;
//! use tokio_util::codec::Framed;
//! use futures::{SinkExt, StreamExt};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let stream = TcpStream::connect("127.0.0.1:5000").await?;
//!     let mut framed = Framed::new(stream, KdbCodec::new(true));
//!     
//!     let query = K::new_symbol("1+1".to_string());
//!     framed.feed(KdbMessage::new(qmsg_type::synchronous, query)).await?;
//!     framed.flush().await?;
//!     
//!     if let Some(Ok(response)) = framed.next().await {
//!         println!("Result: {}", response.payload);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ### Compression Control
//!
//! ```no_run
//! use kdb_codec::*;
//!
//! // Auto mode (default): compress large messages on remote connections only
//! let codec = KdbCodec::new(false);
//!
//! // Using builder pattern (recommended)
//! let codec = KdbCodec::builder()
//!     .is_local(false)
//!     .compression_mode(CompressionMode::Always)
//!     .validation_mode(ValidationMode::Strict)
//!     .build();
//!
//! // Using builder to disable compression
//! let codec = KdbCodec::builder()
//!     .is_local(false)
//!     .compression_mode(CompressionMode::Never)
//!     .validation_mode(ValidationMode::Strict)
//!     .build();
//! ```
//!
//! ### Header Validation
//!
//! ```no_run
//! use kdb_codec::*;
//!
//! // Strict mode (default): reject invalid headers
//! let codec = KdbCodec::builder()
//!     .is_local(false)
//!     .compression_mode(CompressionMode::Auto)
//!     .validation_mode(ValidationMode::Strict)
//!     .build();
//!
//! // Using builder pattern for lenient validation
//! let codec = KdbCodec::builder()
//!     .validation_mode(ValidationMode::Lenient)
//!     .build();
//! ```
//!
//! ## Type Mapping
//!
//! All types are expressed as `K` struct. The table below shows the input types of each q type:
//!
//! | q                | Rust                                              |
//! |------------------|---------------------------------------------------|
//! | `bool`           | `bool`                                            |
//! | `GUID`           | `[u8; 16]`                                        |
//! | `byte`           | `u8`                                              |
//! | `short`          | `i16`                                             |
//! | `int`            | `i32`                                             |
//! | `long`           | `i64`                                             |
//! | `real`           | `f32`                                             |
//! | `float`          | `f64`                                             |
//! | `char`           | `char`                                            |
//! | `symbol`         | `String`                                          |
//! | `timestamp`      | `chrono::DateTime<Utc>`                           |
//! | `month`          | `chrono::NaiveDate`                               |
//! | `date`           | `chrono::NaiveDate`                               |
//! | `datetime`       | `chrono::DateTime<Utc>`                           |
//! | `timespan`       | `chrono::Duration`                                |
//! | `minute`         | `chrono::Duration`                                |
//! | `second`         | `chrono::Duration`                                |
//! | `time`           | `chrono::Duration`                                |
//! | `list`           | `Vec<Item>`                                       |
//! | `compound list`  | `Vec<K>`                                          |
//! | `table`          | `Vec<K>`                                          |
//! | `dictionary`     | `Vec<K>`                                          |
//! | `null`           | `()`                                              |
//!
//! ## Environmental Variables
//!
//! - `KDBPLUS_ACCOUNT_FILE`: Credential file for acceptors (format: `username:sha1_password`)
//! - `KDBPLUS_TLS_KEY_FILE` and `KDBPLUS_TLS_KEY_FILE_SECRET`: TLS certificate files
//! - `QUDSPATH`: Optional path for Unix domain socket abstract namespace

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Security Constants
//++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Maximum allowed list size during deserialization (100 million elements)
///
/// This is a conservative default to prevent memory exhaustion attacks while allowing
/// most legitimate use cases. The actual kdb+ limit is 2^31-1 (2,147,483,647) elements,
/// but attempting to allocate such large lists can cause memory issues.
///
/// Reference: https://www.timestored.com/kdb-guides/kdb-database-limits
/// - kdb+ internal limit: 2 billion items (2^31)
/// - IPC transfer limit: 2GB object size
///
/// This serves as the default value for `KdbCodec::max_list_size`.
/// You can customize this limit per codec instance using the builder pattern.
pub const MAX_LIST_SIZE: usize = 100_000_000;

/// Maximum recursion depth for nested structures (100 levels)
///
/// This limit prevents stack overflow from deeply nested data structures.
/// While kdb+ itself doesn't explicitly document a recursion limit, this default
/// provides a reasonable balance between functionality and safety.
///
/// For most practical use cases, 100 levels of nesting is more than sufficient.
/// Excessive nesting often indicates a data structure design issue.
///
/// This serves as the default value for `KdbCodec::max_recursion_depth`.
/// You can customize this limit per codec instance using the builder pattern.
pub const MAX_RECURSION_DEPTH: usize = 100;

/// Maximum allowed message size in bytes (256 MB)
///
/// This limit protects against memory exhaustion from excessively large messages.
/// The kdb+ IPC protocol has a documented 2GB transfer limit, but this default
/// is more conservative to prevent resource exhaustion.
///
/// Reference: https://www.timestored.com/kdb-guides/kdb-database-limits
/// - kdb+ IPC transfer limit: 2GB object size
///
/// Set to `None` to disable message size checking (not recommended for untrusted connections).
/// This serves as the default value for `KdbCodec::max_message_size`.
pub const MAX_MESSAGE_SIZE: usize = 256 * 1024 * 1024; // 256 MB

/// Maximum allowed decompressed message size in bytes (512 MB)
///
/// This limit protects against compression bomb attacks where a small compressed
/// message decompresses to an enormous size, causing memory exhaustion.
///
/// A compression bomb might send 1KB that decompresses to 1GB, causing denial of service.
/// This limit ensures that even if compression is highly effective, the decompressed
/// size remains reasonable.
///
/// Set to `None` to disable decompressed size checking (not recommended for untrusted connections).
/// This serves as the default value for `KdbCodec::max_decompressed_size`.
pub const MAX_DECOMPRESSED_SIZE: usize = 512 * 1024 * 1024; // 512 MB

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Module Declarations
//++++++++++++++++++++++++++++++++++++++++++++++++++//

// Base modules - must come first
mod conversions;
pub mod error;
mod index;
mod macros;
mod qconsts;
mod qnull_inf;
mod types;

// IPC modules
pub mod codec;
mod connection;
mod deserialize_sync;
mod format;
mod serialize;

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Re-exports
//++++++++++++++++++++++++++++++++++++++++++++++++++//

// Re-export qconsts modules at root level
pub use qconsts::{qattribute, qinf_base, qninf_base, qnull_base, qtype};

// Re-export qnull_inf modules at root level
pub use qnull_inf::{qinf, qninf, qnull};

// Re-export types
pub use error::Error;
pub use types::{Result, C, E, F, G, H, I, J, K, S, U};
// Re-export internal types for use within the crate
pub(crate) use types::{k0, k0_inner, k0_list, AsAny, Klone};

// Re-export conversions
pub use conversions::*;

// Re-export from codec
pub use codec::*;

// Re-export from connection
pub use connection::*;
