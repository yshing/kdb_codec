//! # Connection Module
//!
//! This module provides high-level connection abstractions for communicating with kdb+/q processes
//! using the IPC protocol with Framed codec support for cancellation-safe operations.

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Load Libraries
//++++++++++++++++++++++++++++++++++++++++++++++++++//

use super::codec::{CompressionMode, KdbCodec, KdbMessage, ValidationMode};
use super::Result;
use super::K;
use futures::{SinkExt, StreamExt};
use io::BufRead;
use once_cell::sync::Lazy;
use sha1_smol::Sha1;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;
use std::{env, fs, io, str};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};
use tokio_native_tls::native_tls::{
    Identity, TlsAcceptor as TlsAcceptorInner, TlsConnector as TlsConnectorInner,
};
use tokio_native_tls::{TlsAcceptor, TlsConnector, TlsStream};
use tokio_util::codec::Framed;
use trust_dns_resolver::TokioAsyncResolver;

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Global Variable
//++++++++++++++++++++++++++++++++++++++++++++++++++//

//%% QStream %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

pub mod qmsg_type {
    //! Message types for kdb+ IPC protocol
    //!
    //! # Example
    //! ```rust,no_run
    //! use kdb_codec::*;
    //!
    //! async fn print(arg: &K) {
    //!     println!("{}", arg);
    //! }
    //!
    //! async fn nonsense(x: i64, y: i64) -> i64 {
    //!     x * y - 1
    //! }
    //!
    //! #[tokio::main]
    //! async fn main() -> Result<()> {
    //!     let mut socket = QStream::accept(ConnectionMethod::UDS, "", 5000).await?;
    //!
    //!     // Receive an asynchronous call from the function.
    //!     match socket.receive_message().await {
    //!         Ok((qmsg_type::asynchronous, message)) => {
    //!             println!("asynchronous call: {}", message);
    //!             let list = message.as_vec::<K>().unwrap();
    //!             if list[0].get_symbol().unwrap() == "print" {
    //!                 print(&list[1]).await
    //!             }
    //!         }
    //!         _ => unreachable!(),
    //!     }
    //!
    //!     // Receive a synchronous call from the function.
    //!     match socket.receive_message().await {
    //!         Ok((qmsg_type::synchronous, message)) => {
    //!             println!("synchronous call: {}", message);
    //!             let list = message.as_vec::<K>().unwrap();
    //!             if list[0].get_symbol().unwrap() == "nonsense" {
    //!                 let res = nonsense(list[1].get_long().unwrap(), list[2].get_long().unwrap()).await;
    //!                 // Send bach a response.
    //!                 socket
    //!                     .send_message(&K::new_long(res), qmsg_type::response)
    //!                     .await?;
    //!             }
    //!         }
    //!         _ => unreachable!(),
    //!     }
    //!
    //!     // Receive a final result.
    //!     match socket.receive_message().await {
    //!         Ok((qmsg_type::response, message)) => {
    //!             println!("final: {}", message);
    //!         }
    //!         _ => unreachable!(),
    //!     }
    //!
    //!     Ok(())
    //! }
    //!```
    /// Used to send a message to q/kdb+ asynchronously.
    pub const asynchronous: u8 = 0;
    /// Used to send a message to q/kdb+ synchronously.
    pub const synchronous: u8 = 1;
    /// Used by q/kdb+ to identify a response for a synchronous query.
    pub const response: u8 = 2;
}

//%% QStream Acceptor %%//vvvvvvvvvvvvvvvvvvvvvvvvvvv/

/// Default path for acceptor account file (relative to the current working directory).
const DEFAULT_ACCOUNT_FILE: &str = "credential/kdbaccess";

/// Environment variable to override acceptor account file path.
///
/// Format: `username:sha1_password` per line.
const ACCOUNT_FILE_ENV: &str = "KDBPLUS_ACCOUNT_FILE";

/// Map from user name to password hashed with SHA1.
const ACCOUNTS: Lazy<HashMap<String, String>> = Lazy::new(|| {
    // Map from user to password
    let mut map: HashMap<String, String> = HashMap::new();

    let path = env::var(ACCOUNT_FILE_ENV).unwrap_or_else(|_| DEFAULT_ACCOUNT_FILE.to_string());

    // Open credential file (if missing, keep empty map so acceptor auth fails gracefully)
    let file = match fs::OpenOptions::new().read(true).open(&path) {
        Ok(f) => f,
        Err(_) => return map,
    };
    let mut reader = io::BufReader::new(file);
    let mut line = String::new();
    loop {
        match reader.read_line(&mut line) {
            Ok(0) => {
                //EOF
                break;
            }
            Ok(_) => {
                let credential: Vec<&str> = line.trim_end().split(':').collect();
                if credential.len() >= 2 {
                    map.insert(credential[0].to_string(), credential[1].to_string());
                }
                line.clear();
            }
            Err(_) => break,
        }
    }
    map
});

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Structs
//++++++++++++++++++++++++++++++++++++++++++++++++++//

//%% ConnectionMethod %%//vvvvvvvvvvvvvvvvvvvvvvvvvvv/

/// Connection method to q/kdb+.
pub enum ConnectionMethod {
    TCP = 0,
    TLS = 1,
    /// Unix domanin socket.
    UDS = 2,
}

//%% Query %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

/// Feature of query object.
pub trait Query: Send + Sync {
    /// Convert into a KdbMessage for encoding
    /// # Parameters
    /// - `message_type`: Message type. One of followings:
    ///   - `qmsg_type::asynchronous`
    ///   - `qmsg_type::synchronous`
    ///   - `qmsg_type::response`
    fn to_kdb_message(&self, message_type: u8) -> KdbMessage;
}

//%% FramedStream %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

/// Type alias for framed streams
enum FramedStream {
    Tcp(Framed<TcpStream, KdbCodec>),
    Tls(Framed<TlsStream<TcpStream>, KdbCodec>),
    #[cfg(unix)]
    Uds(Framed<UnixStream, KdbCodec>),
}

//%% QStream %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

/// Stream to communicate with q/kdb+.
pub struct QStream {
    /// Framed stream with codec
    stream: FramedStream,
    /// Connection method. One of followings:
    /// - TCP
    /// - TLS
    /// - UDS
    method: ConnectionMethod,
    /// Indicator of whether the stream is an acceptor or client.
    /// - `true`: Acceptor
    /// - `false`: Client
    listener: bool,
}

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Implementation
//++++++++++++++++++++++++++++++++++++++++++++++++++//

//%% Query %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

/// Text query.
impl Query for &str {
    fn to_kdb_message(&self, message_type: u8) -> KdbMessage {
        // Build a K string object from the query
        let k_string = K::new_string(self.to_string(), 0);
        KdbMessage::new(message_type, k_string)
    }
}

/// Functional query.
impl Query for K {
    fn to_kdb_message(&self, message_type: u8) -> KdbMessage {
        KdbMessage::new(message_type, self.clone())
    }
}

//%% QStream %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

#[bon::bon]
impl QStream {
    /// General constructor of `QStream`.
    fn new(stream: FramedStream, method: ConnectionMethod, is_listener: bool) -> Self {
        QStream {
            stream,
            method,
            listener: is_listener,
        }
    }

    /// Create a builder for connecting to q/kdb+ with fluent API
    ///
    /// # Example
    /// ```ignore
    /// use kdb_codec::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     // Using builder pattern
    ///     let mut stream = QStream::builder()
    ///         .method(ConnectionMethod::TCP)
    ///         .host("localhost")
    ///         .port(5000)
    ///         .credential("user:pass")
    ///         .compression_mode(CompressionMode::Always)
    ///         .validation_mode(ValidationMode::Lenient)
    ///         .connect()
    ///         .await?;
    ///     
    ///     let result = stream.send_sync_message(&"2+2").await?;
    ///     println!("Result: {}", result.get_int()?);
    ///     
    ///     stream.shutdown().await?;
    ///     Ok(())
    /// }
    /// ```
    #[builder(on(String, into), on(&str, into))]
    pub async fn builder(
        method: ConnectionMethod,
        #[builder(default = String::new())] host: String,
        port: u16,
        #[builder(default = String::new())] credential: String,
        #[builder(default)] compression_mode: CompressionMode,
        #[builder(default)] validation_mode: ValidationMode,
    ) -> Result<Self> {
        Self::connect_with_options(
            method,
            &host,
            port,
            &credential,
            compression_mode,
            validation_mode,
        )
        .await
    }

    /// Connect to q/kdb+ specifying a connection method, destination host, destination port and access credential.
    /// # Parameters
    /// - `method`: Connection method. One of followings:
    ///   - TCP
    ///   - TLS
    ///   - UDS
    /// - `host`: Hostname or IP address of the target q process. Empty `str` for Unix domain socket.
    /// - `port`: Port of the target q process.
    /// - `credential`: Credential in the form of `username:password` to connect to the target q process.
    /// # Example
    /// ```no_run
    /// use kdb_codec::*;
    ///
    /// #[tokio::main(flavor = "multi_thread", worker_threads = 2)]
    /// async fn main() -> Result<()> {
    ///     let mut socket =
    ///         QStream::connect(ConnectionMethod::UDS, "", 5000_u16, "ideal:person").await?;
    ///     println!("Connection type: {}", socket.get_connection_type());
    ///
    ///     // Set remote function with asynchronous message
    ///     socket.send_async_message(&"collatz:{[n] seq:enlist n; while[not n = 1; seq,: n:$[n mod 2; 1 + 3 * n; `long$n % 2]]; seq}").await?;
    ///
    ///     // Send a query synchronously
    ///     let mut result = socket.send_sync_message(&"collatz[12]").await?;
    ///     println!("collatz[12]: {}", result);
    ///
    ///     // Send a functional query.
    ///     let mut message = K::new_compound_list(vec![
    ///         K::new_symbol(String::from("collatz")),
    ///         K::new_long(100),
    ///     ]);
    ///     result = socket.send_sync_message(&message).await?;
    ///     println!("collatz[100]: {}", result);
    ///
    ///     // Send a functional asynchronous query.
    ///     message = K::new_compound_list(vec![
    ///         K::new_string(String::from("show"), qattribute::NONE),
    ///         K::new_symbol(String::from("goodbye")),
    ///     ]);
    ///     socket.send_async_message(&message).await?;
    ///
    ///     socket.shutdown().await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn connect(
        method: ConnectionMethod,
        host: &str,
        port: u16,
        credential: &str,
    ) -> Result<Self> {
        Self::connect_with_options(
            method,
            host,
            port,
            credential,
            CompressionMode::Auto,
            ValidationMode::Strict,
        )
        .await
    }

    /// Connect to q/kdb+ with explicit compression and validation options
    ///
    /// # Parameters
    /// - `method`: Connection method (TCP, TLS, or UDS)
    /// - `host`: Hostname or IP address of the target q process. Empty `str` for Unix domain socket.
    /// - `port`: Port of the target q process.
    /// - `credential`: Credential in the form of `username:password` to connect to the target q process.
    /// - `compression_mode`: How to handle message compression
    /// - `validation_mode`: How strictly to validate incoming messages
    ///
    /// # Example
    /// ```no_run
    /// use kdb_codec::*;
    ///
    /// #[tokio::main(flavor = "multi_thread", worker_threads = 2)]
    /// async fn main() -> Result<()> {
    ///     // Connect with always compress and lenient validation
    ///     let mut socket = QStream::connect_with_options(
    ///         ConnectionMethod::TCP,
    ///         "localhost",
    ///         5000,
    ///         "user:pass",
    ///         CompressionMode::Always,
    ///         ValidationMode::Lenient
    ///     ).await?;
    ///
    ///     let result = socket.send_sync_message(&"2+2").await?;
    ///     println!("Result: {}", result.get_int()?);
    ///
    ///     socket.shutdown().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn connect_with_options(
        method: ConnectionMethod,
        host: &str,
        port: u16,
        credential: &str,
        compression_mode: CompressionMode,
        validation_mode: ValidationMode,
    ) -> Result<Self> {
        match method {
            ConnectionMethod::TCP => {
                let stream = connect_tcp(host, port, credential).await?;
                let is_local = matches!(host, "localhost" | "127.0.0.1");
                let codec = KdbCodec::builder()
                    .is_local(is_local)
                    .compression_mode(compression_mode)
                    .validation_mode(validation_mode)
                    .build();
                let framed = Framed::new(stream, codec);
                Ok(QStream::new(
                    FramedStream::Tcp(framed),
                    ConnectionMethod::TCP,
                    false,
                ))
            }
            ConnectionMethod::TLS => {
                let stream = connect_tls(host, port, credential).await?;
                let codec = KdbCodec::builder()
                    .is_local(false)
                    .compression_mode(compression_mode)
                    .validation_mode(validation_mode)
                    .build(); // TLS is always remote
                let framed = Framed::new(stream, codec);
                Ok(QStream::new(
                    FramedStream::Tls(framed),
                    ConnectionMethod::TLS,
                    false,
                ))
            }
            ConnectionMethod::UDS => {
                let stream = connect_uds(port, credential).await?;
                let codec = KdbCodec::builder()
                    .is_local(true)
                    .compression_mode(compression_mode)
                    .validation_mode(validation_mode)
                    .build(); // UDS is always local
                let framed = Framed::new(stream, codec);
                Ok(QStream::new(
                    FramedStream::Uds(framed),
                    ConnectionMethod::UDS,
                    false,
                ))
            }
        }
    }

    /// Accept connection and does handshake.
    /// # Parameters
    /// - `method`: Connection method. One of followings:
    ///   - TCP
    ///   - TLS
    ///   - UDS
    /// - host: Hostname or IP address of this listener. Empty `str` for Unix domain socket.
    /// - port: Listening port.
    /// # Example
    /// ```no_run
    /// use kdb_codec::*;
    ///  
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     // Start listenening over UDS at the port 7000 with authentication enabled.
    ///     while let Ok(mut socket) = QStream::accept(ConnectionMethod::UDS, "", 7000).await {
    ///         tokio::task::spawn(async move {
    ///             loop {
    ///                 match socket.receive_message().await {
    ///                     Ok((_, message)) => {
    ///                         println!("request: {}", message);
    ///                     }
    ///                     _ => {
    ///                         socket.shutdown().await.unwrap();
    ///                         break;
    ///                     }
    ///                 }
    ///             }
    ///         });
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    /// q processes can connect and send messages to this acceptor.
    /// ```q
    /// q)// Process1
    /// q)h:hopen `:unix://7000:reluctant:slowday
    /// q)neg[h] (`monalizza; 3.8)
    /// q)neg[h] (`pizza; 125)
    /// ```
    /// ```q
    /// q)// Process2
    /// q)h:hopen `:unix://7000:mattew:oracle
    /// q)neg[h] (`teddy; "bear")
    /// ```
    /// # Note
    /// - TLS acceptor sets `.kdbplus.close_tls_connection_` on q clien via an asynchronous message. This function is necessary to close
    ///  the socket from the server side without crashing server side application.
    /// - TLS acceptor and UDS acceptor use specific environmental variables to work. See the [Environmental Variable](../ipc/index.html#environmentl-variables) section for details.
    pub async fn accept(method: ConnectionMethod, host: &str, port: u16) -> Result<Self> {
        Self::accept_with_options(
            method,
            host,
            port,
            CompressionMode::Auto,
            ValidationMode::Strict,
        )
        .await
    }

    /// Accept connection with explicit compression and validation options
    ///
    /// # Parameters
    /// - `method`: Connection method (TCP, TLS, or UDS)
    /// - `host`: Hostname or IP address of this listener. Empty `str` for Unix domain socket.
    /// - `port`: Listening port.
    /// - `compression_mode`: How to handle message compression
    /// - `validation_mode`: How strictly to validate incoming messages
    ///
    /// # Example
    /// ```no_run
    /// use kdb_codec::*;
    ///  
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     // Start listening with never compress and lenient validation
    ///     let mut socket = QStream::accept_with_options(
    ///         ConnectionMethod::TCP,
    ///         "127.0.0.1",
    ///         7000,
    ///         CompressionMode::Never,
    ///         ValidationMode::Lenient
    ///     ).await?;
    ///     
    ///     let greeting = socket.send_sync_message(&"string `Hello").await?;
    ///     println!("Greeting: {}", greeting);
    ///     
    ///     socket.shutdown().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn accept_with_options(
        method: ConnectionMethod,
        host: &str,
        port: u16,
        compression_mode: CompressionMode,
        validation_mode: ValidationMode,
    ) -> Result<Self> {
        match method {
            ConnectionMethod::TCP => {
                // Bind to the endpoint.
                let listener = TcpListener::bind(&format!("{}:{}", host, port)).await?;
                // Listen to the endpoint.
                let (mut socket, ip_address) = listener.accept().await?;
                // Read untill null bytes and send back capacity.
                while let Err(_) = read_client_input(&mut socket).await {
                    // Continue to listen in case of error.
                    socket = listener.accept().await?.0;
                }
                // Check if the connection is local
                let is_local = ip_address.ip() == IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
                let codec = KdbCodec::builder()
                    .is_local(is_local)
                    .compression_mode(compression_mode)
                    .validation_mode(validation_mode)
                    .build();
                let framed = Framed::new(socket, codec);
                Ok(QStream::new(
                    FramedStream::Tcp(framed),
                    ConnectionMethod::TCP,
                    true,
                ))
            }
            ConnectionMethod::TLS => {
                // Bind to the endpoint.
                let listener = TcpListener::bind(&format!("{}:{}", host, port)).await?;
                // Check if key exists and decode an identity with a given password.
                let identity = build_identity_from_cert().await?;
                // Build TLS acceptor.
                let tls_acceptor = TlsAcceptor::from(TlsAcceptorInner::new(identity).unwrap());
                // Listen to the endpoint.
                let (mut socket, _) = listener.accept().await?;
                // TLS processing.
                let mut tls_socket = tls_acceptor
                    .accept(socket)
                    .await
                    .expect("failed to accept TLS connection");
                // Read untill null bytes and send back a capacity.
                while let Err(_) = read_client_input(&mut tls_socket).await {
                    // Continue to listen in case of error.
                    socket = listener.accept().await?.0;
                    tls_socket = tls_acceptor
                        .accept(socket)
                        .await
                        .expect("failed to accept TLS connection");
                }
                // TLS is always a remote connection
                let codec = KdbCodec::builder()
                    .is_local(false)
                    .compression_mode(compression_mode)
                    .validation_mode(validation_mode)
                    .build();
                let framed = Framed::new(tls_socket, codec);
                let mut qstream =
                    QStream::new(FramedStream::Tls(framed), ConnectionMethod::TLS, true);
                // In order to close the connection from the server side, it needs to tell a client to close the connection.
                // The `kdbplus_close_tls_connection_` will be called from the server at shutdown.
                qstream
                    .send_async_message(&".kdbplus.close_tls_connection_:{[] hclose .z.w;}")
                    .await?;
                Ok(qstream)
            }
            ConnectionMethod::UDS => {
                // Build a sockt file path.
                let uds_path = create_sockfile_path(port)?;
                let abstract_sockfile_ = format!("\x00{}", uds_path);
                let abstract_sockfile = Path::new(&abstract_sockfile_);
                // Bind to the file
                let listener = UnixListener::bind(&abstract_sockfile).unwrap();
                // Listen to the endpoint
                let (mut socket, _) = listener.accept().await?;
                // Read untill null bytes and send back capacity.
                while let Err(_) = read_client_input(&mut socket).await {
                    // Continue to listen in case of error.
                    socket = listener.accept().await?.0;
                }
                // UDS is always a local connection
                let codec = KdbCodec::builder()
                    .is_local(true)
                    .compression_mode(compression_mode)
                    .validation_mode(validation_mode)
                    .build();
                let framed = Framed::new(socket, codec);
                Ok(QStream::new(
                    FramedStream::Uds(framed),
                    ConnectionMethod::UDS,
                    true,
                ))
            }
        }
    }

    /// Shutdown the socket for a q process.
    /// # Example
    /// See the example of [`connect`](#method.connect).
    pub async fn shutdown(mut self) -> Result<()> {
        // For TLS listener, send the close command
        if self.listener && matches!(self.method, ConnectionMethod::TLS) {
            self.send_async_message(&".kdbplus.close_tls_connection_[]")
                .await?;
        }

        // Close the underlying stream
        match self.stream {
            FramedStream::Tcp(framed) => {
                AsyncWriteExt::shutdown(&mut framed.into_inner()).await?;
            }
            FramedStream::Tls(framed) => {
                if !self.listener {
                    framed.into_inner().get_mut().shutdown()?;
                }
            }
            #[cfg(unix)]
            FramedStream::Uds(framed) => {
                AsyncWriteExt::shutdown(&mut framed.into_inner()).await?;
            }
        }
        Ok(())
    }

    /// Send a message with a specified message type without waiting for a response even for a synchronous message.
    ///  If you need to receive a response you need to use [`receive_message`](#method.receive_message).
    /// # Note
    /// The usage of this function for a synchronous message is to handle an asynchronous message or a synchronous message
    ///   sent by a remote function during its execution.
    /// # Parameters
    /// - `message`: q command to execute on the remote q process.
    ///   - `&str`: q command in a string form.
    ///   - `K`: Query in a functional form.
    /// - `message_type`: Asynchronous or synchronous.
    /// # Example
    /// See the example of [`connect`](#method.connect).
    pub async fn send_message(&mut self, message: &dyn Query, message_type: u8) -> Result<()> {
        let kdb_message = message.to_kdb_message(message_type);
        match &mut self.stream {
            FramedStream::Tcp(framed) => {
                framed.send(kdb_message).await?;
            }
            FramedStream::Tls(framed) => {
                framed.send(kdb_message).await?;
            }
            #[cfg(unix)]
            FramedStream::Uds(framed) => {
                framed.send(kdb_message).await?;
            }
        }
        Ok(())
    }

    /// Send a message asynchronously.
    /// # Parameters
    /// - `message`: q command to execute on the remote q process.
    ///   - `&str`: q command in a string form.
    ///   - `K`: Query in a functional form.
    /// # Example
    /// See the example of [`connect`](#method.connect).
    pub async fn send_async_message(&mut self, message: &dyn Query) -> Result<()> {
        self.send_message(message, qmsg_type::asynchronous).await
    }

    /// Send a message synchronously.
    /// # Note
    /// Remote function must NOT send back a message of asynchronous or synchronous type durning execution of the function.
    /// # Parameters
    /// - `message`: q command to execute on the remote q process.
    ///   - `&str`: q command in a string form.
    ///   - `K`: Query in a functional form.
    /// # Example
    /// See the example of [`connect`](#method.connect).
    pub async fn send_sync_message(&mut self, message: &dyn Query) -> Result<K> {
        // Send the synchronous message
        self.send_message(message, qmsg_type::synchronous).await?;

        // Receive the response
        match self.receive_message().await? {
            (qmsg_type::response, response) => Ok(response),
            (_, message) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("expected a response: {}", message),
            )
            .into()),
        }
    }

    /// Receive a message from a remote q process. The received message is parsed as `K` and message type is
    ///  stored in the first returned value.
    /// # Example
    /// See the example of [`accept`](#method.accept).
    pub async fn receive_message(&mut self) -> Result<(u8, K)> {
        match &mut self.stream {
            FramedStream::Tcp(framed) => match framed.next().await {
                Some(Ok(response)) => Ok((response.message_type, response.payload)),
                Some(Err(e)) => Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    format!("Connection dropped: {}", e),
                )
                .into()),
                None => Err(
                    io::Error::new(io::ErrorKind::ConnectionAborted, "Connection closed").into(),
                ),
            },
            FramedStream::Tls(framed) => match framed.next().await {
                Some(Ok(response)) => Ok((response.message_type, response.payload)),
                Some(Err(e)) => Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    format!("Connection dropped: {}", e),
                )
                .into()),
                None => Err(
                    io::Error::new(io::ErrorKind::ConnectionAborted, "Connection closed").into(),
                ),
            },
            #[cfg(unix)]
            FramedStream::Uds(framed) => match framed.next().await {
                Some(Ok(response)) => Ok((response.message_type, response.payload)),
                Some(Err(e)) => Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    format!("Connection dropped: {}", e),
                )
                .into()),
                None => Err(
                    io::Error::new(io::ErrorKind::ConnectionAborted, "Connection closed").into(),
                ),
            },
        }
    }

    /// Return underlying connection type. One of `TCP`, `TLS` or `UDS`.
    /// # Example
    /// See the example of [`connect`](#method.connect).
    pub fn get_connection_type(&self) -> &str {
        match self.method {
            ConnectionMethod::TCP => "TCP",
            ConnectionMethod::TLS => "TLS",
            ConnectionMethod::UDS => "UDS",
        }
    }
}

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Private Functions
//++++++++++++++++++++++++++++++++++++++++++++++++++//

//%% QStream Connector %%//vvvvvvvvvvvvvvvvvvvvvvvvvv/

/// Inner function of `connect_tcp` and `connect_tls` to establish a TCP connection with the sepcified
///  endpoint. The hostname is resolved to an IP address with a system DNS resolver or parsed directly
///  as an IP address.
///
/// Tries to connect to multiple resolved IP addresses until the first successful connection. Error is
///  returned if none of them are valid.
/// # Parameters
/// - `host`: Hostname or IP address of the target q/kdb+ process.
/// - `port`: Port of the target q process
async fn connect_tcp_impl(host: &str, port: u16) -> Result<TcpStream> {
    // DNS system resolver (should not fail)
    let resolver =
        TokioAsyncResolver::tokio_from_system_conf().expect("failed to create DNS resolver");

    // Check if we were given an IP address
    let ips;
    if let Ok(ip) = host.parse::<IpAddr>() {
        ips = vec![ip.to_string()]
    } else {
        // Resolve hostname to IP addresses
        let response = resolver
            .lookup_ip(host)
            .await
            .expect(&format!("failed to resolve host: {}", host));
        ips = response.iter().map(|ip| ip.to_string()).collect();
    }

    // Try each resolved IP
    for answer in ips {
        match TcpStream::connect(format!("{}:{}", answer, port)).await {
            Ok(socket) => return Ok(socket),
            Err(_) => continue,
        }
    }
    // All addresses failed.
    Err(io::Error::new(io::ErrorKind::ConnectionRefused, "failed to connect").into())
}

/// Send a credential and receive a common capacity.
pub async fn handshake<S>(socket: &mut S, credential_: &str, method_bytes: &str) -> Result<()>
where
    S: Unpin + AsyncWriteExt + AsyncReadExt,
{
    // Send credential and method
    let mut credential = credential_.to_string();
    credential.push_str(method_bytes);
    socket.write_all(credential.as_bytes()).await?;
    // Read a single byte
    let mut capacity = [0u8; 1];
    socket.read_exact(&mut capacity).await?;
    Ok(())
}

/// Connect to q process running on a specified `host` and `port` via TCP with a credential `username:password`.
/// # Parameters
/// - `host`: Hostname or IP address of the target q process.
/// - `port`: Port of the target q process.
/// - `credential`: Credential in the form of `username:password` to connect to the target q process.
async fn connect_tcp(host: &str, port: u16, credential: &str) -> Result<TcpStream> {
    let mut socket = connect_tcp_impl(host, port).await?;
    handshake(&mut socket, credential, "\x03\x00").await?;
    Ok(socket)
}

/// TLS version of `connect_tcp`.
/// # Parameters
/// - `host`: Hostname or IP address of the target q process.
/// - `port`: Port of the target q process.
/// - `credential`: Credential in the form of `username:password` to connect to the target q process.
async fn connect_tls(host: &str, port: u16, credential: &str) -> Result<TlsStream<TcpStream>> {
    // Connect via TCP
    let socket_ = connect_tcp_impl(host, port).await?;
    // Use TLS
    let connector = TlsConnector::from(TlsConnectorInner::new().unwrap());
    let mut socket = connector
        .connect(host, socket_)
        .await
        .expect("failed to create TLS session");
    // Handshake
    handshake(&mut socket, credential, "\x03\x00").await?;
    Ok(socket)
}

/// Build a path of a socket file.
fn create_sockfile_path(port: u16) -> Result<String> {
    // Create file path
    let udspath = match env::var("QUDSPATH") {
        Ok(dir) => format!("{}/kx.{}", dir, port),
        Err(_) => format!("/tmp/kx.{}", port),
    };

    Ok(udspath)
}

/// Connect to q process running on the specified `port` via Unix domain socket with a credential `username:password`.
/// # Parameters
/// - `port`: Port of the target q process.
/// - `credential`: Credential in the form of `username:password` to connect to the target q process.
#[cfg(unix)]
async fn connect_uds(port: u16, credential: &str) -> Result<UnixStream> {
    // Create a file path.
    let uds_path = create_sockfile_path(port)?;
    let abstract_sockfile_ = format!("\x00{}", uds_path);
    let abstract_sockfile = Path::new(&abstract_sockfile_);
    // Connect to kdb+.
    let mut socket = UnixStream::connect(&abstract_sockfile).await?;
    // Handshake
    handshake(&mut socket, credential, "\x06\x00").await?;

    Ok(socket)
}

//%% QStream Acceptor %%//vvvvvvvvvvvvvvvvvvvvvvvvvvv/

/// Read username, password, capacity and null byte from q client at the connection and does authentication.
///  Close the handle if the authentication fails.
async fn read_client_input<S>(socket: &mut S) -> Result<()>
where
    S: Unpin + AsyncWriteExt + AsyncReadExt,
{
    let debug_auth = matches!(std::env::var("KDBPLUS_DEBUG_AUTH").ok().as_deref(), Some("1"));
    // Buffer to read inputs.
    let mut client_input = [0u8; 32];
    // credential will be built from small fractions of bytes.
    let mut passed_credential = String::new();
    loop {
        // Read a client credential input.
        match socket.read(&mut client_input).await {
            Ok(0) => {
                // Client closed the connection.
                socket.shutdown().await?;
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "client disconnected").into());
            }
            Ok(n) => {
                let chunk = &client_input[..n];
                // Locate a byte denoting a capacity.
                if let Some(index) = chunk.iter().position(|byte| *byte == 0x03 || *byte == 0x06)
                {
                    let capacity = chunk[index];
                    passed_credential
                        .push_str(str::from_utf8(&chunk[0..index]).expect("invalid bytes"));
                    let credential = passed_credential.as_str().split(':').collect::<Vec<&str>>();
                    if credential.len() < 2 {
                        if debug_auth {
                            eprintln!("[acceptor auth] invalid credential format");
                        }
                        // Authentication failure.
                        socket.shutdown().await?;
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "authentication failed",
                        )
                        .into());
                    }
                    if debug_auth {
                        eprintln!(
                            "[acceptor auth] user='{}' capacity=0x{:02x}",
                            credential[0], capacity
                        );
                    }
                    if let Some(encoded) = ACCOUNTS.get(&credential[0].to_string()) {
                        // User exists
                        let mut hasher = Sha1::new();
                        hasher.update(credential[1].as_bytes());
                        let encoded_password = hasher.digest().to_string();
                        if encoded == &encoded_password {
                            // Client passed correct credential
                            if debug_auth {
                                eprintln!("[acceptor auth] success");
                            }
                            socket.write_all(&[capacity; 1]).await?;
                            return Ok(());
                        } else {
                            if debug_auth {
                                eprintln!("[acceptor auth] password mismatch");
                            }
                            // Authentication failure.
                            // Close connection.
                            socket.shutdown().await?;
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "authentication failed",
                            )
                            .into());
                        }
                    } else {
                        if debug_auth {
                            eprintln!("[acceptor auth] unknown user");
                        }
                        // Authentication failure.
                        // Close connection.
                        socket.shutdown().await?;
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "authentication failed",
                        )
                        .into());
                    }
                } else {
                    // Append a fraction of credential
                    passed_credential
                        .push_str(str::from_utf8(chunk).expect("invalid bytes"));
                }
            }
            Err(error) => {
                return Err(error.into());
            }
        }
    }
}

/// Check if server key exists and return teh contents.
async fn build_identity_from_cert() -> Result<Identity> {
    // Check if server key exists.
    if let Ok(path) = env::var("KDBPLUS_TLS_KEY_FILE") {
        if let Ok(password) = env::var("KDBPLUS_TLS_KEY_FILE_SECRET") {
            let cert_file = tokio::fs::File::open(Path::new(&path)).await.unwrap();
            let mut reader = BufReader::new(cert_file);
            let mut der: Vec<u8> = Vec::new();
            // Read the key file.
            reader.read_to_end(&mut der).await?;
            // Create identity.
            if let Ok(identity) = Identity::from_pkcs12(&der, &password) {
                return Ok(identity);
            } else {
                return Err(
                    io::Error::new(io::ErrorKind::InvalidData, "authentication failed").into(),
                );
            }
        } else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "KDBPLUS_TLS_KEY_FILE_SECRET is not set",
            )
            .into());
        }
    } else {
        return Err(
            io::Error::new(io::ErrorKind::NotFound, "KDBPLUS_TLS_KEY_FILE is not set").into(),
        );
    }
}
