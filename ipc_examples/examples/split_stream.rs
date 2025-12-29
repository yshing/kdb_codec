//! Example demonstrating channel forwarding using split() to avoid tokio::select!
//!
//! This example shows a cleaner approach where the Framed stream is split into
//! separate read and write halves, eliminating the need for tokio::select!

use futures::{SinkExt, StreamExt};
use kdb_codec::*;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_util::codec::Framed;

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to q process running on localhost:5000
    let stream = TcpStream::connect("127.0.0.1:5000")
        .await
        .map_err(|e| Error::NetworkError(e.to_string()))?;

    let codec = KdbCodec::new(true);
    let framed = Framed::new(stream, codec);

    // Create a channel for sending messages
    let (tx, rx) = mpsc::channel::<KdbMessage>(100);

    // Spawn a task to send some messages
    let sender_handle = tokio::spawn(async move {
        for i in 0..5 {
            let query = KdbMessage::new(
                qmsg_type::synchronous,
                K::new_compound_list(vec![K::new_symbol(String::from("til")), K::new_long(i)]),
            );

            if tx.send(query).await.is_err() {
                eprintln!("Receiver dropped");
                break;
            }

            // Simulate some delay between messages
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        println!("Sender finished");
    });

    // Forward messages using split pattern (no select! needed)
    let result = forward_with_split(rx, framed).await;

    // Wait for sender to finish
    let _ = sender_handle.await;

    result
}

/// Forwards messages from a channel to kdb+ using split() pattern
///
/// This approach splits the Framed stream into separate sink and stream halves,
/// avoiding the need for tokio::select! and simplifying the code.
///
/// Benefits:
/// - Cleaner code without select! complexity
/// - Independent handling of sends and receives
/// - More composable and easier to test
async fn forward_with_split(
    mut rx: mpsc::Receiver<KdbMessage>,
    framed: Framed<TcpStream, KdbCodec>,
) -> Result<()> {
    // Split the framed stream into independent sink (write) and stream (read) halves
    let (mut sink, mut stream) = framed.split();

    // Spawn a task to handle responses from kdb+
    let response_handle = tokio::spawn(async move {
        let mut response_count = 0;
        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    response_count += 1;
                    println!("Response {}: {}", response_count, response.payload);
                }
                Err(e) => {
                    eprintln!("Error receiving response: {}", e);
                }
            }
        }
        println!(
            "Response handler finished. Total responses: {}",
            response_count
        );
    });

    // Forward messages from channel to kdb+ (no select! needed!)
    let mut messages_sent = 0;
    while let Some(msg) = rx.recv().await {
        // feed() buffers the message
        sink.feed(msg)
            .await
            .map_err(|e| Error::NetworkError(e.to_string()))?;

        // flush() sends the buffered message
        sink.flush()
            .await
            .map_err(|e| Error::NetworkError(e.to_string()))?;

        messages_sent += 1;
    }

    println!(
        "Channel forwarding finished. Total messages sent: {}",
        messages_sent
    );

    // Close the sink to signal we're done sending
    drop(sink);

    // Wait for the response handler to finish
    let _ = response_handle.await;

    Ok(())
}

/// Alternative: Using split with bidirectional communication
///
/// This example shows how to handle both requests and responses with split()
#[allow(dead_code)]
async fn bidirectional_with_split(
    request_rx: mpsc::Receiver<KdbMessage>,
    response_tx: mpsc::Sender<K>,
    framed: Framed<TcpStream, KdbCodec>,
) -> Result<()> {
    let (sink, stream) = framed.split();

    // Spawn task to send requests
    let send_handle = tokio::spawn(async move { forward_requests(request_rx, sink).await });

    // Spawn task to receive responses
    let recv_handle = tokio::spawn(async move { receive_responses(stream, response_tx).await });

    // Wait for both tasks
    let (send_result, recv_result) = tokio::join!(send_handle, recv_handle);

    send_result
        .map_err(|e| Error::NetworkError(e.to_string()))?
        .map_err(|e| Error::NetworkError(e.to_string()))?;
    recv_result
        .map_err(|e| Error::NetworkError(e.to_string()))?
        .map_err(|e| Error::NetworkError(e.to_string()))?;

    Ok(())
}

#[allow(dead_code)]
async fn forward_requests<S>(
    mut rx: mpsc::Receiver<KdbMessage>,
    mut sink: S,
) -> std::result::Result<(), String>
where
    S: SinkExt<KdbMessage> + Unpin,
    S::Error: std::fmt::Display,
{
    while let Some(msg) = rx.recv().await {
        sink.feed(msg).await.map_err(|e| e.to_string())?;
        sink.flush().await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[allow(dead_code)]
async fn receive_responses<St>(
    mut stream: St,
    tx: mpsc::Sender<K>,
) -> std::result::Result<(), String>
where
    St: StreamExt<Item = std::result::Result<KdbMessage, std::io::Error>> + Unpin,
{
    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                if tx.send(response.payload).await.is_err() {
                    break; // Receiver dropped
                }
            }
            Err(e) => {
                eprintln!("Error receiving: {}", e);
                break;
            }
        }
    }
    Ok(())
}

/// Comparison: Using select! (more complex, but needed when coordinating operations)
#[allow(dead_code)]
async fn forward_with_select(
    mut rx: mpsc::Receiver<KdbMessage>,
    mut framed: Framed<TcpStream, KdbCodec>,
) -> Result<()> {
    let mut messages_sent = 0;

    loop {
        tokio::select! {
            msg_opt = rx.recv() => {
                match msg_opt {
                    Some(msg) => {
                        framed.feed(msg).await
                            .map_err(|e| Error::NetworkError(e.to_string()))?;
                        SinkExt::<KdbMessage>::flush(&mut framed).await
                            .map_err(|e| Error::NetworkError(e.to_string()))?;

                        messages_sent += 1;

                        // Must handle response here in select! context
                        if let Some(result) = framed.next().await {
                            match result {
                                Ok(response) => {
                                    println!("Response: {}", response.payload);
                                }
                                Err(e) => {
                                    eprintln!("Error: {}", e);
                                }
                            }
                        }
                    }
                    None => {
                        println!("Channel closed. Messages sent: {}", messages_sent);
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
