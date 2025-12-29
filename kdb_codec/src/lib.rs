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
//! // Using with_options method
//! let codec = KdbCodec::with_options(false, CompressionMode::Always, ValidationMode::Strict);
//!
//! // Using builder pattern (recommended)
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
//! let codec = KdbCodec::with_options(false, CompressionMode::Auto, ValidationMode::Strict);
//!
//! // Using builder pattern
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
mod codec;
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
