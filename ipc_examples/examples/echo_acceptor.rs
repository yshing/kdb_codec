//! Echo acceptor server.
//!
//! This starts a `kdb_codec` acceptor (server-side IPC endpoint) that *does not evaluate* q.
//! It simply echoes back any synchronous message payload it receives.
//!
//! ## Run
//!
//! ```bash
//! # Create a credential file (username:sha1(password))
//! user=e2e
//! pass=e2e
//! hash=$(printf "%s" "$pass" | shasum -a 1 | awk '{print $1}')
//! acct=$(mktemp)
//! printf "%s:%s\n" "$user" "$hash" > "$acct"
//!
//! export KDBPLUS_ACCOUNT_FILE="$acct"
//! cargo run -p ipc_examples --example echo_acceptor
//! ```
//!
//! ## Inspect from q
//!
//! ```q
//! q)h:hopen `:127.0.0.1:7000:e2e:e2e
//! q)h 42
//! 42
//! q)h 1 2 3
//! 1 2 3
//! q)h `sym
//! `sym
//! q)h ("abc"; 1b; 2.3)
//! ("abc";1b;2.3)
//! ```

use kdb_codec::*;

fn env_u16(name: &str, default: u16) -> u16 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(default)
}

#[tokio::main]
async fn main() -> Result<()> {
    let host = std::env::var("KDBPLUS_ECHO_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env_u16("KDBPLUS_ECHO_PORT", 7000);

    eprintln!("Starting echo acceptor on {host}:{port}");
    eprintln!(
        "Auth: set KDBPLUS_ACCOUNT_FILE to a file with 'username:sha1(password)' per line"
    );

    let mut socket = QStream::accept(ConnectionMethod::TCP, &host, port).await?;
    eprintln!("Client connected. Echoing synchronous messages...");

    loop {
        match socket.receive_message().await {
            Ok((msg_type, payload)) => {
                eprintln!("recv type={msg_type} payload={payload}");

                // q sends synchronous queries and expects a response.
                if msg_type == qmsg_type::synchronous {
                    socket.send_message(&payload, qmsg_type::response).await?;
                }
            }
            Err(err) => {
                eprintln!("connection closed: {err}");
                socket.shutdown().await?;
                break;
            }
        }
    }

    Ok(())
}
