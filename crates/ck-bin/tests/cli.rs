use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn migrate_creates_sqlite_database() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("kernel.db");

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
    let db_path = dir.path().join("kernel.db");

    let mut cmd = Command::cargo_bin("context-kernal").unwrap();
    cmd.args(["serve", "--database", db_path.to_str().unwrap()])
        .write_stdin(
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_status","arguments":{}}}"#,
        );

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""name":"context-kernal""#))
        .stdout(predicate::str::contains(r#""fts5_enabled":true"#));

    assert!(db_path.exists());
}
