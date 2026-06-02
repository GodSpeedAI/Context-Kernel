# Release Process

## Prerequisites

- Rust stable toolchain installed
- Git repository clean

## Build Release Binary

```bash
cargo build --release -p ck-bin
```

The binary is at `target/release/context-kernal`.

## Generate Checksum

```bash
sha256sum target/release/context-kernal > context-kernal-linux-x86_64.sha256
```

## Verify Release

```bash
# Run all checks
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Build and smoke test
cargo build --release -p ck-bin

tmpdir="$(mktemp -d)"
target/release/context-kernal migrate --database "$tmpdir/kernel.db"
target/release/context-kernal doctor --database "$tmpdir/kernel.db"
target/release/context-kernal export-schema | head -5
```

## Tag and Release

```bash
git tag v0.1.0
git push origin v0.1.0
```

The GitHub Actions release workflow will automatically:
1. Run fmt, clippy, and tests
2. Build the Linux release binary
3. Generate a SHA-256 checksum
4. Create a GitHub Release with the binary and checksum

## Install

```bash
# Download
curl -LO https://github.com/USER/Context_Kernel_Service_MCP/releases/latest/download/context-kernal
chmod +x context-kernal

# Verify checksum
sha256sum -c context-kernal-linux-x86_64.sha256

# Run
./context-kernal migrate --database ./data/kernel.db
./context-kernal doctor --database ./data/kernel.db
./context-kernal serve --config context-kernal.toml
```
