# CI

Recommended checks:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets
cargo test --workspace
cargo build --release -p ck-bin
```

The repository includes a GitHub Actions example at `.github/workflows/ci.yml`.

Release packaging is single-binary: upload `target/release/context-kernal` from the release build job.
