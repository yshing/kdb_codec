//! Example demonstrating safe message forwarding from channels to kdb+ using the codec pattern
//!
//! This example shows how to receive messages from a tokio channel and forward them
//! to a kdb+ process without losing or duplicating messages.

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

    // Forward messages from channel to kdb+
    let result = forward_messages_safely(rx, framed).await;

    // Wait for sender to finish
    let _ = sender_handle.await;

    result
}

/// Safely forwards messages from a channel to kdb+ without loss or duplication
///
/// Key guarantees:
/// - Each message received from rx is sent exactly once
/// - If cancellation occurs before feed(), the message is lost but never sent
/// - If cancellation occurs after feed() but before flush(), the message is buffered
///   and will be sent on the next iteration
async fn forward_messages_safely(
    mut rx: mpsc::Receiver<KdbMessage>,
    mut framed: Framed<TcpStream, KdbCodec>,
) -> Result<()> {
    let mut messages_sent = 0;

    loop {
        tokio::select! {
            // Receive message from channel
            msg_opt = rx.recv() => {
                match msg_opt {
                    Some(msg) => {
                        // Step 1: feed() buffers the message
                        // If select! cancels here, msg is lost but never sent to kdb+
                        framed
                            .feed(msg)
                            .await
                            .map_err(|e| Error::NetworkError(e.to_string()))?;

                        // Step 2: flush() sends all buffered messages
                        // After this point, the message is guaranteed sent
                        SinkExt::<KdbMessage>::flush(&mut framed)
                            .await
                            .map_err(|e| Error::NetworkError(e.to_string()))?;

                        messages_sent += 1;

                        // Step 3: Receive response
                        if let Some(result) = framed.next().await {
                            match result {
                                Ok(response) => {
                                    println!("Response {}: {}", messages_sent, response.payload);
                                }
                                Err(e) => {
                                    eprintln!("Error receiving response: {}", e);
                                }
                            }
                        }
                    }
                    None => {
                        // Channel closed, exit loop
                        println!("Channel closed. Total messages sent: {}", messages_sent);
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Example of what NOT to do - batching without explicit control
#[allow(dead_code)]
async fn unsafe_batching_example(
    mut rx: mpsc::Receiver<KdbMessage>,
    mut framed: Framed<TcpStream, KdbCodec>,
) -> Result<()> {
    loop {
        tokio::select! {
            msg_opt = rx.recv() => {
                if let Some(msg) = msg_opt {
                    // ❌ WRONG: Multiple feed() calls without flush() in between
                    // This batches messages, which might not be what you want
                    framed.feed(msg).await
                        .map_err(|e| Error::NetworkError(e.to_string()))?;

                    // If you receive more messages before flush(), they accumulate
                    // and are all sent together on the next flush()
                } else {
                    break;
                }
            }
        }
    }

    // All buffered messages sent here
    SinkExt::<KdbMessage>::flush(&mut framed)
        .await
        .map_err(|e| Error::NetworkError(e.to_string()))?;

    Ok(())
}

/// Example showing intentional batching for performance
#[allow(dead_code)]
async fn safe_batching_example(
    mut rx: mpsc::Receiver<KdbMessage>,
    mut framed: Framed<TcpStream, KdbCodec>,
) -> Result<()> {
    const BATCH_SIZE: usize = 10;
    let mut batch_count = 0;

    loop {
        tokio::select! {
            msg_opt = rx.recv() => {
                if let Some(msg) = msg_opt {
                    // ✅ CORRECT: Intentionally batch messages for performance
                    framed.feed(msg).await
                        .map_err(|e| Error::NetworkError(e.to_string()))?;
                    batch_count += 1;

                    // Flush after reaching batch size
                    if batch_count >= BATCH_SIZE {
                        SinkExt::<KdbMessage>::flush(&mut framed).await
                            .map_err(|e| Error::NetworkError(e.to_string()))?;
                        println!("Flushed batch of {} messages", batch_count);
                        batch_count = 0;
                    }
                } else {
                    // Channel closed - flush any remaining messages
                    if batch_count > 0 {
                        SinkExt::<KdbMessage>::flush(&mut framed).await
                            .map_err(|e| Error::NetworkError(e.to_string()))?;
                        println!("Flushed final batch of {} messages", batch_count);
                    }
                    break;
                }
            }
        }
    }

    Ok(())
}
