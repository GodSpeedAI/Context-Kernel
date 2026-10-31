# scripts

## Overview
Repository helper scripts for Context Kernel. Its single file, `run-exact-test.sh`, is a verification runner: it runs exactly one Context Kernel Rust test by full name through `cargo test`, first discovering the test list for the requested package/target and refusing to run on zero or ambiguous matches. This makes "a zero-test Cargo success is a FAILED gate" an enforced property rather than an assumption.

## Interface
- [run-exact-test.sh:usage] - CLI contract: `run-exact-test.sh <package> <integration-target|-> <full-test-name>`; exactly three positional arguments are required.
- [run-exact-test.sh:list_args] - Discovery command: `cargo test -p <package> --lib -- --list` when the target is `-` (unit tests in the crate's `lib.rs`), otherwise `cargo test -p <package> --test <target> -- --list` for a named cargo integration test target.
- [run-exact-test.sh:matches] - Full-name filter over the `--list` output (`awk -F': test$'` comparing the name before the `: test` suffix for an exact match).
- [run-exact-test.sh:count] - Match-count gate around the discovery result.
- [run-exact-test.sh:run_args] - Run command: `cargo test -p <package> (--lib | --test <target>) <name> -- --exact --nocapture`.

## Invariants
- [run-exact-test.sh:usage] - A wrong argument count prints usage to stderr and exits with code 2.
- [run-exact-test.sh:count] - The full test name must match exactly one discovered test: zero matches exits 1 with `FAIL: zero discovered tests ...`, more than one exits 1 listing the matches; a zero-test Cargo success is a FAILED gate.
- [run-exact-test.sh:list_args] - Target `-` always means the crate's lib unit tests; any other value is passed as a named cargo integration test target.
- [run-exact-test.sh:set -euo pipefail] - Strict shell mode: the first failing command (including a failing `cargo` run of the selected test) aborts the script with a non-zero exit status.

## Side Effects
- [run-exact-test.sh:list_args] / [run-exact-test.sh:run_args] - Invokes `cargo test` twice (discover, then run), which compiles and executes Rust tests and writes Cargo build artifacts under `target/`.
- [run-exact-test.sh:count] - Emits diagnostics (`usage`, `FAIL`/`OK` messages) to stderr and stdout.

## Verification
- The script is itself a test runner. Exercise it from the repository root with:
  `./scripts/run-exact-test.sh <package> <integration-target|-> <full-test-name>`
  where `<integration-target>` is a cargo test target name (e.g. `cli`) or `-` for lib unit tests.
