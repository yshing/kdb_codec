//! Example demonstrating the kdb-codec pattern with Framed streams
//!
//! This example shows how to use the KdbCodec with tokio's Framed to
//! communicate with a kdb+ process using a cleaner, more idiomatic approach.

use kdb_codec::error::Error;
use kdb_codec::*;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to q process running on localhost:5000
    let stream = TcpStream::connect("127.0.0.1:5000")
        .await
        .map_err(|e| Error::NetworkError(e.to_string()))?;

    println!("Connected using kdb-codec pattern!");

    use futures::sink::SinkExt;
    use futures::stream::StreamExt;

    // Create codec and framed stream
    let codec = KdbCodec::new(true); // true = local connection
    let mut framed = Framed::new(stream, codec);

    // Example 1: Send a simple query using KdbMessage
    let query1 = KdbMessage::new(
        qmsg_type::synchronous,
        K::new_compound_list(vec![
            K::new_symbol(String::from("+")),
            K::new_long(1),
            K::new_long(1),
        ]),
    );

    framed
        .send(query1)
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

    // Example 2: Send another K object query
    let query2 = KdbMessage::new(
        qmsg_type::synchronous,
        K::new_compound_list(vec![
            K::new_symbol(String::from("til")),
            K::new_long(5),
        ]),
    );

    framed
        .send(query2)
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
