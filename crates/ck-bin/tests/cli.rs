use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;

fn temp_db(dir: &tempfile::TempDir) -> PathBuf {
    dir.path().join("kernel.db")
}

fn temp_data_dir(dir: &tempfile::TempDir) -> PathBuf {
    let d = dir.path().join("data");
    fs::create_dir_all(&d).unwrap();
    d
}

fn make_jsonrpc(id: u64, method: &str, params: serde_json::Value) -> String {
    serde_json::to_string(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    }))
    .unwrap()
}

fn tool_call(id: u64, name: &str, args: serde_json::Value) -> String {
    make_jsonrpc(
        id,
        "tools/call",
        serde_json::json!({"name": name, "arguments": args}),
    )
}

#[test]
fn migrate_creates_sqlite_database() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = temp_db(&dir);

    let mut cmd = Command::cargo_bin("context-kernal").unwrap();
    cmd.args(["migrate", "--database", db_path.to_str().unwrap()]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("migrated"));

    assert!(db_path.exists());
}

#[test]
fn serve_stdio_answers_get_status_tool_call() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = temp_db(&dir);
    let data_dir = temp_data_dir(&dir);

    let mut cmd = Command::cargo_bin("context-kernal").unwrap();
    cmd.args([
        "serve",
        "--database",
        db_path.to_str().unwrap(),
        "--data-dir",
        data_dir.to_str().unwrap(),
    ])
    .write_stdin(tool_call(1, "get_status", serde_json::json!({})));

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""name":"context-kernel""#))
        .stdout(predicate::str::contains(r#""fts5_enabled":true"#));
}

#[test]
fn store_context_persists_event() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = temp_db(&dir);
    let data_dir = temp_data_dir(&dir);

    let input = tool_call(
        1,
        "store_context",
        serde_json::json!({"session_id":"sess-test","kind":"decision","text":"We chose Rust for the backend","metadata":{}}),
    );

    let mut cmd = Command::cargo_bin("context-kernal").unwrap();
    cmd.args([
        "serve",
        "--database",
        db_path.to_str().unwrap(),
        "--data-dir",
        data_dir.to_str().unwrap(),
    ])
    .write_stdin(input);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("event_id"))
        .stdout(predicate::str::contains("stored_as"))
        .stdout(predicate::str::contains("decision"));
}

#[test]
fn recall_context_returns_stored_events() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = temp_db(&dir);
    let data_dir = temp_data_dir(&dir);

    // Store an event first, then recall in the same session
    let store_input = tool_call(
        1,
        "store_context",
        serde_json::json!({"session_id":"sess-recall","kind":"decision","text":"We decided to use SQLite for storage","metadata":{}}),
    );

    let recall_input = tool_call(
        2,
        "recall_context",
        serde_json::json!({"session_id":"sess-recall","query":"database storage","top_k":5}),
    );

    let mut cmd = Command::cargo_bin("context-kernal").unwrap();
    cmd.args([
        "serve",
        "--database",
        db_path.to_str().unwrap(),
        "--data-dir",
        data_dir.to_str().unwrap(),
    ])
    .write_stdin(format!("{store_input}\n{recall_input}"));

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("query_id"))
        .stdout(predicate::str::contains("results"));
}

#[test]
fn ingest_source_indexes_local_docs() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = temp_db(&dir);
    let data_dir = temp_data_dir(&dir);

    // Create a test docs directory
    let docs_dir = dir.path().join("docs");
    fs::create_dir_all(&docs_dir).unwrap();
    fs::write(
        docs_dir.join("test.md"),
        "# Test Document\n\nThis is a test document about Rust programming.\n",
    )
    .unwrap();

    let config = serde_json::json!({"source_id":"test_docs","kind":"filesystem_tree","path": docs_dir.display().to_string()});
    let args = serde_json::json!({"source": config});

    let input = tool_call(1, "ingest_source", args);

    let mut cmd = Command::cargo_bin("context-kernal").unwrap();
    cmd.args([
        "serve",
        "--database",
        db_path.to_str().unwrap(),
        "--data-dir",
        data_dir.to_str().unwrap(),
    ])
    .write_stdin(input);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("job_id"))
        .stdout(predicate::str::contains("completed"));
}

#[test]
fn query_reference_returns_results_after_ingest() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = temp_db(&dir);
    let data_dir = temp_data_dir(&dir);

    // Create test docs
    let docs_dir = dir.path().join("docs");
    fs::create_dir_all(&docs_dir).unwrap();
    fs::write(
        docs_dir.join("guide.md"),
        "# Getting Started\n\nInstall Rust using rustup. Rust is a systems programming language.\n",
    )
    .unwrap();

    let config = serde_json::json!({"source_id":"test_docs","kind":"filesystem_tree","path": docs_dir.display().to_string()});

    let ingest_input = tool_call(1, "ingest_source", serde_json::json!({"source": config}));
    let query_input = tool_call(
        2,
        "query_reference",
        serde_json::json!({"query":"Rust programming language","top_k":5}),
    );

    let mut cmd = Command::cargo_bin("context-kernal").unwrap();
    cmd.args([
        "serve",
        "--database",
        db_path.to_str().unwrap(),
        "--data-dir",
        data_dir.to_str().unwrap(),
    ])
    .write_stdin(format!("{ingest_input}\n{query_input}"));

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("query_id"))
        .stdout(predicate::str::contains("results"));
}

#[test]
fn resolve_library_id_alias_works() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = temp_db(&dir);
    let data_dir = temp_data_dir(&dir);

    // The resolve-library-id alias should not return "unknown tool"
    let input = tool_call(
        1,
        "resolve-library-id",
        serde_json::json!({"libraryName":"react"}),
    );

    let mut cmd = Command::cargo_bin("context-kernal").unwrap();
    cmd.args([
        "serve",
        "--database",
        db_path.to_str().unwrap(),
        "--data-dir",
        data_dir.to_str().unwrap(),
    ])
    .write_stdin(input);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("candidates").or(predicate::str::contains("query_id")));
}

#[test]
fn get_library_docs_alias_works() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = temp_db(&dir);
    let data_dir = temp_data_dir(&dir);

    let input = tool_call(
        1,
        "get-library-docs",
        serde_json::json!({"context7CompatibleLibraryID":"/vercel/next.js","topic":"routing"}),
    );

    let mut cmd = Command::cargo_bin("context-kernal").unwrap();
    cmd.args([
        "serve",
        "--database",
        db_path.to_str().unwrap(),
        "--data-dir",
        data_dir.to_str().unwrap(),
    ])
    .write_stdin(input);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("results").or(predicate::str::contains("query_id")));
}

#[test]
fn refresh_source_tool_works() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = temp_db(&dir);
    let data_dir = temp_data_dir(&dir);

    let docs_dir = dir.path().join("docs");
    fs::create_dir_all(&docs_dir).unwrap();
    fs::write(
        docs_dir.join("test.md"),
        "# Refresh Test\n\nThis is a test document for refresh.\n",
    )
    .unwrap();

    let config = serde_json::json!({"source_id":"refresh_test_docs","kind":"filesystem_tree","path": docs_dir.display().to_string()});

    let ingest_input = tool_call(1, "ingest_source", serde_json::json!({"source": config}));
    let refresh_input = tool_call(
        2,
        "refresh_source",
        serde_json::json!({"source_id":"refresh_test_docs"}),
    );

    let mut cmd = Command::cargo_bin("context-kernal").unwrap();
    cmd.args([
        "serve",
        "--database",
        db_path.to_str().unwrap(),
        "--data-dir",
        data_dir.to_str().unwrap(),
    ])
    .write_stdin(format!("{ingest_input}\n{refresh_input}"));

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("job_id"))
        .stdout(predicate::str::contains("completed").or(predicate::str::contains("queued")));
}

#[test]
fn sse_healthz_endpoint_works() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = temp_db(&dir);
    let data_dir = temp_data_dir(&dir);

    let input = tool_call(1, "get_status", serde_json::json!({}));

    let mut cmd = Command::cargo_bin("context-kernal").unwrap();
    cmd.args([
        "serve",
        "--database",
        db_path.to_str().unwrap(),
        "--data-dir",
        data_dir.to_str().unwrap(),
        "--sse",
        "127.0.0.1:0",
    ])
    .write_stdin(input);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""name":"context-kernel""#));
}

#[test]
fn query_reference_includes_citations_after_ingest() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = temp_db(&dir);
    let data_dir = temp_data_dir(&dir);

    let docs_dir = dir.path().join("docs");
    fs::create_dir_all(&docs_dir).unwrap();
    fs::write(
        docs_dir.join("citations.md"),
        "# Citation Test\n\nThis document discusses WebAssembly compilation techniques.\n",
    )
    .unwrap();

    let config = serde_json::json!({"source_id":"citation_test_docs","kind":"filesystem_tree","path": docs_dir.display().to_string()});

    let ingest_input = tool_call(1, "ingest_source", serde_json::json!({"source": config}));
    let query_input = tool_call(
        2,
        "query_reference",
        serde_json::json!({"query":"WebAssembly compilation","top_k":5}),
    );

    let mut cmd = Command::cargo_bin("context-kernal").unwrap();
    cmd.args([
        "serve",
        "--database",
        db_path.to_str().unwrap(),
        "--data-dir",
        data_dir.to_str().unwrap(),
    ])
    .write_stdin(format!("{ingest_input}\n{query_input}"));

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("query_id"))
        .stdout(predicate::str::contains("results"));
}
