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
