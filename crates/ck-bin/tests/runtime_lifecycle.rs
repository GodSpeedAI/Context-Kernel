//! Runtime lifecycle contracts for the installed-binary distribution:
//! version identity, per-user default database, and standalone SSE survival
//! when stdin is closed (service-manager style launch).

use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn bin() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin("context-kernal"))
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn http_get_status(addr: &str, path: &str) -> Option<String> {
    let mut stream = TcpStream::connect(addr).ok()?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n"
    )
    .ok()?;
    let mut body = String::new();
    stream.read_to_string(&mut body).ok()?;
    body.lines().next().map(str::to_string)
}

#[test]
fn version_flag_reports_cargo_package_version() {
    let out = bin().arg("--version").output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(
        text.contains(env!("CARGO_PKG_VERSION")),
        "unexpected version output: {text}"
    );
}

#[test]
fn default_database_is_per_user_and_independent_of_cwd() {
    let data = tempfile::tempdir().unwrap();
    let work = tempfile::tempdir().unwrap();

    let out = bin()
        .current_dir(work.path())
        .env("CK_DATA_DIR", data.path())
        .arg("migrate")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "migrate failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(
        data.path().join("kernel.db").is_file(),
        "database must be created in the per-user data directory"
    );
    assert!(
        !work.path().join("kernel.db").exists(),
        "database must not be created in the working directory"
    );
}

#[test]
fn explicit_database_flag_still_wins_over_user_default() {
    let data = tempfile::tempdir().unwrap();
    let explicit = tempfile::tempdir().unwrap();
    let db = explicit.path().join("explicit.db");

    let out = bin()
        .env("CK_DATA_DIR", data.path())
        .args(["migrate", "--database", db.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(db.is_file());
    assert!(!data.path().join("kernel.db").exists());
}

/// A standalone SSE service with a closed stdin must keep serving until it is
/// signalled, not exit at EOF as the stdio-only mode does.
#[test]
fn sse_service_survives_closed_stdin() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("kernel.db");
    let addr = format!("127.0.0.1:{}", free_port());

    let mut child = bin()
        .args(["serve", "--database", db.to_str().unwrap(), "--sse", &addr])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(20);
    let mut ready = false;
    while Instant::now() < deadline {
        if http_get_status(&addr, "/healthz")
            .map(|l| l.contains("200"))
            .unwrap_or(false)
        {
            ready = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(ready, "SSE endpoint never became ready");

    // Give the process time to (incorrectly) exit after stdin EOF.
    std::thread::sleep(Duration::from_millis(800));
    assert!(
        child.try_wait().unwrap().is_none(),
        "SSE service exited after stdin closed"
    );
    assert!(
        http_get_status(&addr, "/healthz")
            .map(|l| l.contains("200"))
            .unwrap_or(false),
        "SSE endpoint stopped answering after stdin closed"
    );

    child.kill().ok();
    child.wait().ok();
}

/// Stdio-only mode keeps its contract: EOF on stdin ends the process.
#[test]
fn stdio_only_exits_at_stdin_eof() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("kernel.db");
    let mut child = bin()
        .args(["serve", "--database", db.to_str().unwrap()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        assert!(Instant::now() < deadline, "stdio-only serve did not exit");
        std::thread::sleep(Duration::from_millis(50));
    }
}
