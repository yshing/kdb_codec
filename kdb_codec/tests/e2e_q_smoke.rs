use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

fn should_run() -> bool {
    matches!(
        std::env::var("KDBPLUS_RUN_Q_E2E_TESTS").ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE")
    )
}

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

#[test]
fn e2e_q_smoke_prints_hello_and_exits() -> io::Result<()> {
    if !should_run() {
        eprintln!(
            "skipping q smoke test (set KDBPLUS_RUN_Q_E2E_TESTS=1 to enable; optionally set KDBPLUS_Q_BIN)"
        );
        return Ok(());
    }

    let mut child = Command::new(q_bin())
        .arg("-q")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "failed to open q stdin"))?;
        writeln!(stdin, "-1 \"hello\"")?;
        // quit q
        writeln!(stdin, "\\\\")?;
    }

    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(3);
    loop {
        if let Some(_status) = child.try_wait()? {
            break;
        }
        if start.elapsed() > timeout {
            let _ = child.kill();
            let output = child.wait_with_output()?;
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!(
                    "q smoke timed out after {timeout:?}\nstdout:\n{}\nstderr:\n{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    let output = child.wait_with_output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stdout.contains("hello") {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("q did not print hello\nstdout:\n{stdout}\nstderr:\n{stderr}"),
        ));
    }
    if !stderr.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("q printed to stderr\nstdout:\n{stdout}\nstderr:\n{stderr}"),
        ));
    }

    Ok(())
}
