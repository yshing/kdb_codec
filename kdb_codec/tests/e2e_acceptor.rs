use kdb_codec::*;
use sha1_smol::Sha1;
use std::fs;
use std::io;
use std::io::Write;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

fn q_bin() -> String {
    if let Ok(bin) = std::env::var("KDBPLUS_Q_BIN") {
        return bin;
    }

    // Common kdb+ install location on macOS: $HOME/q/m64/q
    if let Ok(home) = std::env::var("HOME") {
        let candidate = PathBuf::from(home).join("q").join("m64").join("q");
        if candidate.exists() {
            return candidate.to_string_lossy().to_string();
        }
    }

    "q".to_string()
}

fn pick_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let port = listener.local_addr().expect("local addr").port();
    drop(listener);
    port
}

fn write_account_file(user: &str, pass: &str) -> io::Result<PathBuf> {
    let mut hasher = Sha1::new();
    hasher.update(pass.as_bytes());
    let sha1_hex = hasher.digest().to_string();

    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "kdb_codec_e2e_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    ));
    fs::create_dir_all(&dir)?;
    let mut file_path = dir;
    file_path.push("kdbaccess");

    fs::write(&file_path, format!("{user}:{sha1_hex}\n"))?;
    Ok(file_path)
}

async fn run_acceptor_echo_once(port: u16) -> Result<()> {
    eprintln!("[e2e] acceptor listening on 127.0.0.1:{port}");
    let mut socket = QStream::accept(ConnectionMethod::TCP, "127.0.0.1", port).await?;
    eprintln!("[e2e] acceptor accepted connection");
    loop {
        match socket.receive_message().await {
            Ok((qmsg_type::synchronous, message)) => {
                eprintln!(
                    "[e2e] recv sync message qtype={} attr={}",
                    message.get_type(),
                    message.get_attribute()
                );
                // q may encode (`echo; `symbol) as a SYMBOL_LIST (homogeneous) instead of a COMPOUND_LIST.
                if message.get_type() == qtype::SYMBOL_LIST {
                    let syms = message.as_vec::<String>().unwrap();
                    if syms.len() == 2 && syms[0] == "echo" {
                        let resp = K::new_symbol(syms[1].clone());
                        socket.send_message(&resp, qmsg_type::response).await?;
                        eprintln!("[e2e] sent response qtype={}", resp.get_type());
                    } else {
                        // Ignore unexpected sync frames.
                    }
                    continue;
                }

                if message.get_type() != qtype::COMPOUND_LIST {
                    // Ignore unexpected sync frames.
                    continue;
                }
                let list = message.as_vec::<K>().unwrap();
                if list.len() != 2 || list[0].get_type() != qtype::SYMBOL_ATOM {
                    continue;
                }
                if list[0].get_symbol().unwrap() != "echo" {
                    continue;
                }

                socket.send_message(&list[1], qmsg_type::response).await?;
                eprintln!("[e2e] sent response qtype={}", list[1].get_type());
            }
            Ok((_other, _message)) => {
                // Ignore async/response frames from client.
                continue;
            }
            Err(_) => break,
        }
    }

    socket.shutdown().await?;
    Ok(())
}

#[test]
#[ignore] // Ignored by default since it requires q binary
fn e2e_q_script_to_rust_acceptor_echo_roundtrip() -> Result<()> {

    let user = "e2e";
    let pass = "e2e";
    let account_file = write_account_file(user, pass)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("write account file: {e}")))?;

    // Must be set before the acceptor is initialized.
    std::env::set_var("KDBPLUS_ACCOUNT_FILE", &account_file);

    let port = pick_free_port();

    // Drive the async acceptor on a dedicated thread.
    let acceptor_thread = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        rt.block_on(run_acceptor_echo_once(port))
    });

    // Give the acceptor thread a moment to start and bind before spawning q.
    std::thread::sleep(Duration::from_millis(200));

    // If the acceptor failed fast (e.g. bind/auth setup), surface it immediately.
    if acceptor_thread.is_finished() {
        match acceptor_thread.join() {
            Ok(Ok(())) => {
                return Err(io::Error::new(io::ErrorKind::Other, "acceptor exited unexpectedly").into())
            }
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                return Err(io::Error::new(io::ErrorKind::Other, "acceptor thread panicked").into())
            }
        }
    }

    // Confirm the port is actually bound (helps debug q-side hangs on hopen).
    // If we can bind here, the acceptor hasn't bound yet.
    if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
        return Err(
            io::Error::new(
                io::ErrorKind::AddrNotAvailable,
                format!("acceptor did not bind to 127.0.0.1:{port} (port was still available)"),
            )
            .into(),
        );
    }

    let q_script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("e2e_acceptor_echo.q");

    let mut child = Command::new(q_bin())
        .arg("-q")
        .stdin(Stdio::piped())
        .env("KDBCODEC_E2E_HOST", "127.0.0.1")
        .env("KDBCODEC_E2E_PORT", port.to_string())
        .env("KDBCODEC_E2E_USER", user)
        .env("KDBCODEC_E2E_PASS", pass)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("spawn q: {e}")))?;

    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "failed to open q stdin"))?;
        // Execute the script by reading it within q and evaluating it as a single program.
        // This avoids two issues observed in this environment:
        // - `q -q script.q` does not exit as expected.
        // - piping the raw file into stdin evaluates line-by-line and breaks multi-line defs.
        writeln!(
            stdin,
            "value \"\\n\" sv read0 `:{};",
            q_script.display()
        )?;
        // Quit q after script evaluation.
        writeln!(stdin, "\\\\")?;
    }

    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(30);
    let _status = loop {
        if let Some(status) = child.try_wait().expect("try_wait") {
            break status;
        }
        if start.elapsed() > timeout {
            let _ = child.kill();
            let output = child
                .wait_with_output()
                .expect("wait_with_output after kill");
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!(
                "q script timed out after {timeout:?}\nstdout:\n{stdout}\nstderr:\n{stderr}"
            );
        }
        std::thread::sleep(Duration::from_millis(50));
    };

    let output = child
        .wait_with_output()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("wait q: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // q often exits with status 0 even on errors; treat output as the source of truth.
    if !stdout.contains("ok") || !stderr.trim().is_empty() {
        panic!("q script did not report success\nstdout:\n{stdout}\nstderr:\n{stderr}");
    }

    // Ensure acceptor finishes and propagate its error (if any).
    match acceptor_thread.join() {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return Err(e),
        Err(_) => {
            return Err(io::Error::new(io::ErrorKind::Other, "acceptor thread panicked").into())
        }
    }

    Ok(())
}
