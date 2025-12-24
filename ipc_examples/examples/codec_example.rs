//! Example demonstrating the kdb-codec pattern with Framed streams
//!
//! This example shows how to use the KdbCodec with tokio's Framed to
//! communicate with a kdb+ process using a cleaner, more idiomatic approach.

use kdbplus::ipc::*;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to q process running on localhost:5000
    let stream = TcpStream::connect("127.0.0.1:5000")
        .await
        .map_err(|e| Error::NetworkError(e.to_string()))?;

    // Create a framed stream with our KdbCodec
    let codec = KdbCodec::new(true); // true = local connection
    let mut framed = Framed::new(stream, codec);

    println!("Connected using kdb-codec pattern!");

    // Example 1: Send a simple string query
    use futures::sink::SinkExt;
    use futures::stream::StreamExt;

    // Send a synchronous text query
    // Note: Using feed() + flush() is cancellation-safe, unlike send()
    // which can lose the message if used in tokio::select! and another branch completes first
    framed
        .feed(("1+1", qmsg_type::synchronous))
        .await
        .map_err(|e| Error::NetworkError(e.to_string()))?;
    framed
        .flush()
        .await
        .map_err(|e| Error::NetworkError(e.to_string()))?;

    // Receive response
    if let Some(result) = framed.next().await {
        match result {
            Ok(response) => {
                println!("Response to '1+1': {}", response.payload);
            }
            Err(e) => {
                eprintln!("Error receiving response: {}", e);
            }
        }
    }

    // Example 2: Send a K object query
    let query = KdbMessage::new(
        qmsg_type::synchronous,
        K::new_compound_list(vec![
            K::new_symbol(String::from("til")),
            K::new_long(5),
        ]),
    );

    // Using feed() + flush() for cancellation safety
    framed
        .feed(query)
        .await
        .map_err(|e| Error::NetworkError(e.to_string()))?;
    framed
        .flush()
        .await
        .map_err(|e| Error::NetworkError(e.to_string()))?;

    // Receive response
    if let Some(result) = framed.next().await {
        match result {
            Ok(response) => {
                println!("Response to 'til 5': {}", response.payload);
            }
            Err(e) => {
                eprintln!("Error receiving response: {}", e);
            }
        }
    }

    println!("\nCodec-based communication successful!");

    Ok(())
}
