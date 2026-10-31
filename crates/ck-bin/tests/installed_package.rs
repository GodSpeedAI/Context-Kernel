//! Installed-package qualification.
//!
//! These tests execute an INSTALLED Context Kernel executable (not the cargo
//! build output) selected with `CK_QUALIFY_BIN`. They are skipped when the
//! variable is unset, so `cargo test --workspace` stays hermetic. Package CI
//! and the Jetson smoke test set the variable to the installed binary.
//!
//! Optional: `CK_QUALIFY_VERSION` asserts the exact `--version` string.
//! Every MCP response is parsed as JSON; assertions never match raw text.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::{json, Value};

fn qualify_bin() -> Option<PathBuf> {
    let raw = std::env::var_os("CK_QUALIFY_BIN")?;
    let path = PathBuf::from(raw);
    assert!(
        path.is_file(),
        "CK_QUALIFY_BIN does not point to a file: {}",
        path.display()
    );
    Some(path)
}

macro_rules! require_bin {
    () => {
        match qualify_bin() {
            Some(bin) => bin,
            None => {
                eprintln!("skipped: CK_QUALIFY_BIN is not set");
                return;
            }
        }
    };
}

fn rpc_tool(id: u64, name: &str, arguments: Value) -> String {
    json!({"jsonrpc": "2.0", "id": id, "method": "tools/call",
           "params": {"name": name, "arguments": arguments}})
    .to_string()
}

/// Run `serve --database <db>` once, feed the request lines on stdin, and
/// return the parsed JSON-RPC responses (one per non-empty stdout line).
fn serve_stdio(bin: &Path, db: &Path, data_dir: &Path, lines: &[String]) -> Vec<Value> {
    let mut child = Command::new(bin)
        .args(["serve", "--database"])
        .arg(db)
        .arg("--data-dir")
        .arg(data_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn installed binary");
    {
        let mut stdin = child.stdin.take().unwrap();
        for line in lines {
            writeln!(stdin, "{line}").unwrap();
        }
    }
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "serve exited with {:?}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    // Diagnostics belong on stderr; stdout must be pure JSON-RPC.
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            serde_json::from_str::<Value>(l)
                .unwrap_or_else(|e| panic!("stdout line is not JSON ({e}): {l}"))
        })
        .collect()
}

fn result_of(response: &Value) -> &Value {
    assert!(
        response.get("error").is_none(),
        "MCP error response: {response}"
    );
    response
        .get("result")
        .unwrap_or_else(|| panic!("no result in response: {response}"))
}

fn run_subcommand(bin: &Path, args: &[&str]) -> std::process::Output {
    Command::new(bin)
        .args(args)
        .output()
        .expect("run subcommand")
}

#[test]
fn identity_version_help_and_subcommands() {
    let bin = require_bin!();
    let version = run_subcommand(&bin, &["--version"]);
    assert!(version.status.success());
    let text = String::from_utf8_lossy(&version.stdout).to_string();
    if let Some(expected) = std::env::var_os("CK_QUALIFY_VERSION") {
        let expected = expected.to_string_lossy().to_string();
        assert!(
            text.split_whitespace().any(|t| t == expected),
            "version mismatch: expected {expected}, got {text:?}"
        );
    }

    let help = run_subcommand(&bin, &["--help"]);
    let help_text = String::from_utf8_lossy(&help.stdout).to_string();
    for sub in [
        "serve",
        "migrate",
        "doctor",
        "export-schema",
        "rebuild-index",
    ] {
        assert!(help_text.contains(sub), "help is missing subcommand {sub}");
    }
}

#[test]
fn sqlite_fts5_create_migrate_doctor_and_rebuild() {
    let bin = require_bin!();
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("kernel.db");

    let migrate = run_subcommand(&bin, &["migrate", "--database", db.to_str().unwrap()]);
    assert!(migrate.status.success(), "migrate failed");
    assert!(db.is_file(), "migrate did not create the database");

    let doctor = run_subcommand(&bin, &["doctor", "--database", db.to_str().unwrap()]);
    let doctor_text = String::from_utf8_lossy(&doctor.stdout).to_string();
    assert!(doctor.status.success());
    assert!(
        doctor_text.contains("fts5_enabled=true"),
        "FTS5 not enabled: {doctor_text}"
    );

    let schema = run_subcommand(&bin, &["export-schema"]);
    let schema_text = String::from_utf8_lossy(&schema.stdout).to_string();
    assert!(
        schema_text.contains("session_events_fts") && schema_text.contains("chunks_fts"),
        "schema is missing FTS5 tables"
    );

    let rebuild = run_subcommand(&bin, &["rebuild-index", "--database", db.to_str().unwrap()]);
    assert!(rebuild.status.success(), "rebuild-index failed");
}

#[test]
fn memory_survives_process_restart_and_is_recalled_with_content() {
    let bin = require_bin!();
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("kernel.db");
    let data = dir.path().join("data");

    let store = rpc_tool(
        1,
        "store_context",
        json!({"session_id": "qual-sess", "kind": "decision",
               "text": "Qualification decision: use SQLite FTS5 for lexical recall",
               "metadata": {}}),
    );
    let stored = serve_stdio(&bin, &db, &data, &[store]);
    let stored_id = result_of(&stored[0])["event"]["event_id"]
        .as_str()
        .expect("event_id")
        .to_string();

    // Fresh process, same database file.
    let recall = rpc_tool(
        2,
        "recall_context",
        json!({"session_id": "qual-sess", "query": "lexical recall", "top_k": 5}),
    );
    let recalled = serve_stdio(&bin, &db, &data, &[recall]);
    let results = result_of(&recalled[0])["results"]
        .as_array()
        .expect("results array");
    let hit = results
        .iter()
        .find(|r| r["event_id"] == stored_id)
        .unwrap_or_else(|| panic!("stored event not recalled after restart: {recalled:?}"));
    assert_eq!(hit["kind"], "decision");
    assert!(
        hit["text"]
            .as_str()
            .unwrap_or_default()
            .contains("SQLite FTS5"),
        "recalled event lost its content: {hit}"
    );
}

#[test]
fn ingest_query_cited_retrieval_and_refresh_over_stdio() {
    let bin = require_bin!();
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("kernel.db");
    let data = dir.path().join("data");
    let corpus = dir.path().join("corpus");
    std::fs::create_dir_all(&corpus).unwrap();
    std::fs::write(
        corpus.join("policy.md"),
        "# POL-QUAL-001\nHigh-risk work needs a cited context packet before authority.\n",
    )
    .unwrap();

    let ingest = rpc_tool(
        1,
        "ingest_source",
        json!({"source": {"source_id": "qual_docs", "kind": "filesystem_tree",
                          "path": corpus.display().to_string()}}),
    );
    let query = rpc_tool(
        2,
        "query_reference",
        json!({"query": "cited context packet", "top_k": 3}),
    );
    let refresh = rpc_tool(3, "refresh_source", json!({"source_id": "qual_docs"}));
    let responses = serve_stdio(&bin, &db, &data, &[ingest, query, refresh]);

    assert_eq!(result_of(&responses[0])["job"]["status"], "completed");

    let results = result_of(&responses[1])["results"]
        .as_array()
        .expect("results array");
    let hit = results
        .iter()
        .find(|r| {
            r["citation"]["uri"]
                .as_str()
                .unwrap_or_default()
                .ends_with("policy.md")
        })
        .unwrap_or_else(|| panic!("no citation to the ingested file: {responses:?}"));
    assert!(
        hit["excerpt"]
            .as_str()
            .unwrap_or_default()
            .contains("cited context packet"),
        "excerpt missing cited text: {hit}"
    );
    assert_eq!(hit["citation"]["anchor"], "pol-qual-001");

    assert_eq!(result_of(&responses[2])["job"]["status"], "completed");
}

#[test]
fn fail_closed_unwritable_database_path_creates_nothing() {
    let bin = require_bin!();
    let dir = tempfile::tempdir().unwrap();
    // A regular file used as a parent directory makes the database path unwritable.
    let blocker = dir.path().join("blocker");
    std::fs::write(&blocker, "not a directory").unwrap();
    let db = blocker.join("kernel.db");

    let out = run_subcommand(&bin, &["migrate", "--database", db.to_str().unwrap()]);
    assert!(
        !out.status.success(),
        "migrate must refuse an unwritable path"
    );
    assert!(!db.exists());
}

#[test]
fn fail_closed_corrupt_database_is_not_replaced_with_empty_one() {
    let bin = require_bin!();
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("kernel.db");
    let garbage = b"this is not an sqlite database, do not overwrite me".repeat(64);
    std::fs::write(&db, &garbage).unwrap();

    let out = run_subcommand(&bin, &["migrate", "--database", db.to_str().unwrap()]);
    assert!(!out.status.success(), "corrupt database must be refused");
    assert_eq!(
        std::fs::read(&db).unwrap(),
        garbage,
        "corrupt database content was modified"
    );
}

#[test]
fn fail_closed_invalid_config_is_refused() {
    let bin = require_bin!();
    let dir = tempfile::tempdir().unwrap();
    let cfg = dir.path().join("broken.toml");
    std::fs::write(&cfg, "[storage\ndatabase_path = ").unwrap();
    let out = run_subcommand(&bin, &["migrate", "--config", cfg.to_str().unwrap()]);
    assert!(!out.status.success(), "invalid config must be refused");
}

fn free_loopback_addr() -> String {
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = l.local_addr().unwrap().to_string();
    drop(l);
    addr
}

/// Minimal HTTP/1.1 client over loopback. Returns (status, body).
fn http_request(
    addr: &str,
    method: &str,
    path: &str,
    auth: Option<&str>,
    body: &str,
) -> (u16, String) {
    use std::io::{Read, Write};
    let mut stream = std::net::TcpStream::connect(addr).expect("connect loopback");
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .unwrap();
    let auth_header = auth
        .map(|t| format!("Authorization: Bearer {t}\r\n"))
        .unwrap_or_default();
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\n{auth_header}Connection: close\r\n\r\n{body}",
        body.len()
    )
    .unwrap();
    let mut raw = String::new();
    stream.read_to_string(&mut raw).unwrap();
    let status = raw
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let body = raw.split("\r\n\r\n").nth(1).unwrap_or_default().to_string();
    (status, body)
}

fn wait_for_healthz(addr: &str) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while std::time::Instant::now() < deadline {
        if let Ok(mut s) = std::net::TcpStream::connect(addr) {
            use std::io::{Read, Write};
            let _ = write!(
                s,
                "GET /healthz HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n"
            );
            let mut raw = String::new();
            let _ = s.read_to_string(&mut raw);
            if raw.starts_with("HTTP/1.1 200") {
                return;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    panic!("SSE endpoint at {addr} never became ready");
}

struct Child(std::process::Child);
impl Drop for Child {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn context_required_returns_verified_complete_bundle() {
    let bin = require_bin!();
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("kernel.db");
    let data = dir.path().join("data");
    let corpus_root = dir.path().join("corpus");
    let docs = corpus_root.join("loop-docs");
    std::fs::create_dir_all(&docs).unwrap();
    std::fs::write(
        docs.join("policy.md"),
        "# POL-ACL-001\nHigh-risk work needs a cited context packet before authority.\n",
    )
    .unwrap();

    let digest = "b".repeat(64);
    let args = json!({
        "schema_version": "v1",
        "event_id": "11111111-2222-3333-4444-555555555555",
        "source_agent": "swe-seed",
        "event_type": "ContextRequired",
        "occurred_at": "2026-08-25T00:00:00Z",
        "payload": {
            "domain_model_hash": "a".repeat(64),
            "world_ref": format!("world:qualify@sha256:{digest}"),
            "namespace": "agentic_capability_loop",
            "work_request_id": "wr-qual",
            "context_requirement_id": "cr-qual",
            "corpus_id": "loop-docs",
            "query": "context packet",
            "max_results": 5
        }
    });
    let request = rpc_tool(1, "context_required", args);
    let mut child = Command::new(&bin)
        .args(["serve", "--database"])
        .arg(&db)
        .arg("--data-dir")
        .arg(&data)
        .env("CK_CONTEXT_CORPUS_ROOT", &corpus_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let mut stdin = child.stdin.take().unwrap();
        writeln!(stdin, "{request}").unwrap();
    }
    let out = child.wait_with_output().unwrap();
    let response: Value = serde_json::from_slice(&out.stdout).expect("stdout is JSON");
    let result = result_of(&response);

    assert_eq!(result["access"]["result"], "accessible");
    let bundle = &result["context_bundle"];
    assert_eq!(
        bundle["extensions"]["cep.profile"]["profile_id"],
        "godspeed.context_bundle"
    );
    assert_eq!(
        bundle["extensions"]["cep.profile"]["profile_version"],
        "1.0.0"
    );
    assert_eq!(bundle["integrity"]["verification_status"], "verified");
    assert_eq!(bundle["integrity"]["tamper_status"], "intact");
    assert_eq!(bundle["completeness_status"], "complete_for_context");
    assert_eq!(
        bundle["extensions"]["godspeed.context_bundle"]["retrieval_completeness"],
        "complete"
    );
    assert!(
        bundle.to_string().contains("policy.md"),
        "bundle does not cite the ingested corpus file"
    );
}

#[test]
fn sse_requires_bearer_and_refuses_unauthorized_tools() {
    let bin = require_bin!();
    let dir = tempfile::tempdir().unwrap();
    let addr = free_loopback_addr();
    let cfg = dir.path().join("sse.toml");
    std::fs::write(
        &cfg,
        format!(
            r#"[service]
name = "context-kernal"
[storage]
database_path = "{db}"
[retrieval]
default_mode = "hybrid"
semantic_enabled = false
[mcp]
enable_sse = true
[mcp.sse]
bind = "{addr}"
require_auth = true
[auth]
tokens = ["qual-token"]
[authorization]
allow_tools = ["get_status"]
allow_corpora = ["*"]
"#,
            db = dir.path().join("sse.db").display()
        ),
    )
    .unwrap();

    let child = Command::new(&bin)
        .args(["serve", "--config"])
        .arg(&cfg)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let _guard = Child(child);
    wait_for_healthz(&addr);

    let status_call = rpc_tool(1, "get_status", json!({}));
    let (no_token, _) = http_request(&addr, "POST", "/mcp", None, &status_call);
    assert_eq!(no_token, 401, "request without a token must be refused");
    let (bad_token, _) = http_request(&addr, "POST", "/mcp", Some("wrong"), &status_call);
    assert_eq!(bad_token, 401, "wrong token must be refused");

    let (ok, body) = http_request(&addr, "POST", "/mcp", Some("qual-token"), &status_call);
    assert_eq!(ok, 200);
    let parsed: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(result_of(&parsed)["service"]["healthy"], true);

    let forbidden = rpc_tool(
        2,
        "store_context",
        json!({"session_id": "s", "kind": "decision", "text": "x", "metadata": {}}),
    );
    let (_, body) = http_request(&addr, "POST", "/mcp", Some("qual-token"), &forbidden);
    let refused: Value = serde_json::from_str(&body).unwrap();
    assert!(
        refused.get("error").is_some() && refused.get("result").is_none(),
        "unauthorized tool must be refused, got {refused}"
    );
    assert!(refused["error"]["message"]
        .as_str()
        .unwrap_or_default()
        .contains("unauthorized tool"));
}

#[cfg(unix)]
#[test]
fn sse_stops_cleanly_on_sigterm() {
    let bin = require_bin!();
    let dir = tempfile::tempdir().unwrap();
    let addr = free_loopback_addr();
    let db = dir.path().join("kernel.db");
    let child = Command::new(&bin)
        .args(["serve", "--database"])
        .arg(&db)
        .args(["--sse", &addr])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut guard = Child(child);
    wait_for_healthz(&addr);

    let status = Command::new("kill")
        .args(["-TERM", &guard.0.id().to_string()])
        .status()
        .unwrap();
    assert!(status.success());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        if let Some(exit) = guard.0.try_wait().unwrap() {
            assert!(exit.success(), "SIGTERM should exit cleanly, got {exit:?}");
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "SSE did not stop on SIGTERM"
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
