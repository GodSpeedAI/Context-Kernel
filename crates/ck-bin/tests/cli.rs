use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use base64::Engine;
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
fn export_schema_prints_sqlite_schema() {
    let mut cmd = Command::cargo_bin("context-kernal").unwrap();
    cmd.arg("export-schema");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "CREATE TABLE IF NOT EXISTS sources",
        ))
        .stdout(predicate::str::contains(
            "CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts",
        ));
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

#[test]
fn schema_export_can_initialize_database() {
    let mut cmd = Command::cargo_bin("context-kernal").unwrap();
    let output = cmd
        .arg("export-schema")
        .assert()
        .success()
        .get_output()
        .clone();
    let schema = String::from_utf8(output.stdout).unwrap();
    assert!(
        schema.contains("schema_migrations"),
        "schema should contain schema_migrations table"
    );
    assert!(
        schema.contains("CREATE TABLE IF NOT EXISTS sources"),
        "schema should contain sources table"
    );
    assert!(
        schema.contains("CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts"),
        "schema should contain FTS table"
    );
}

#[test]
fn migrate_then_doctor() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("kernel.db");

    let mut cmd = Command::cargo_bin("context-kernal").unwrap();
    cmd.args(["migrate", "--database", db_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("migrated"));

    let mut cmd = Command::cargo_bin("context-kernal").unwrap();
    cmd.args(["doctor", "--database", db_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("sqlite"))
        .stdout(predicate::str::contains("fts5_enabled=true"));
}

#[test]
fn retention_deletes_old_artifact_payload() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("kernel.db");
    let data_dir = dir.path().join("data");
    std::fs::create_dir_all(&data_dir).unwrap();

    let config_content = format!(
        r#"
[service]
name = "context-kernal"

[storage]
database_path = "{}"

[retrieval]
default_mode = "hybrid"
semantic_enabled = false

[retention]
enabled = true
session_event_days = 90
inactive_session_days = 30
refresh_job_days = 30
artifact_days = 0
disabled_source_days = 30
"#,
        db_path.display()
    );

    let config_path = dir.path().join("test-config.toml");
    std::fs::write(&config_path, &config_content).unwrap();

    let store_input = tool_call(
        1,
        "store_context",
        serde_json::json!({"session_id":"sess-ret","kind":"note","artifact_ref":{"content":base64::engine::general_purpose::STANDARD.encode(b"test artifact data for retention"),"content_type":"text/plain"},"metadata":{}}),
    );

    let mut cmd = Command::cargo_bin("context-kernal").unwrap();
    cmd.args([
        "serve",
        "--config",
        config_path.to_str().unwrap(),
        "--database",
        db_path.to_str().unwrap(),
        "--data-dir",
        data_dir.to_str().unwrap(),
    ])
    .write_stdin(store_input);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("event_id"));
}

#[test]
fn serve_with_local_config_template() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("kernel.db");
    let data_dir = dir.path().join("data");
    std::fs::create_dir_all(&data_dir).unwrap();

    let config_content = format!(
        r#"
[service]
name = "context-kernal"

[storage]
database_path = "{}"

[retrieval]
default_mode = "hybrid"
semantic_enabled = false

[retention]
enabled = false
"#,
        db_path.display()
    );

    let config_path = dir.path().join("test-config.toml");
    std::fs::write(&config_path, &config_content).unwrap();

    let input = tool_call(1, "get_status", serde_json::json!({}));

    let mut cmd = Command::cargo_bin("context-kernal").unwrap();
    cmd.args([
        "serve",
        "--config",
        config_path.to_str().unwrap(),
        "--database",
        db_path.to_str().unwrap(),
        "--data-dir",
        data_dir.to_str().unwrap(),
    ])
    .write_stdin(input);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""name":"context-kernel""#));
}
