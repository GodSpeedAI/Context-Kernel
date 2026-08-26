#!/usr/bin/env bash
# Run a single, exactly-matched Context Kernel Rust test by full name.
#
# Usage:
#   ./scripts/run-exact-test.sh <package> <integration-target|-> <full-test-name>
#
# The integration target is the cargo test target name (e.g. `cli`) or `-` for
# unit tests in the crate's lib.rs. The full test name must match exactly one
# discovered test; the script asserts this and exits non-zero on zero or many
# matches, then runs that one test with `--exact`.
#
# A zero-test Cargo success is a FAILED gate.
set -euo pipefail

if [ "$#" -ne 3 ]; then
  echo "usage: $0 <package> <integration-target|-> <full-test-name>" >&2
  exit 2
fi

pkg="$1"
target="$2"
name="$3"

# Build the test list. For unit tests we list the lib; for integration tests we
# list the named target. Both use `-- --list` to print `<name>: test` lines.
if [ "$target" = "-" ]; then
  list_args=(test -p "$pkg" --lib -- --list)
  run_args=(test -p "$pkg" --lib "$name" -- --exact --nocapture)
else
  list_args=(test -p "$pkg" --test "$target" -- --list)
  run_args=(test -p "$pkg" --test "$target" "$name" -- --exact --nocapture)
fi

# Discover matching tests. We require exactly one full name match.
matches=$(cargo "${list_args[@]}" 2>/dev/null | awk -F': test$' -v n="$name" '$1==n {print $1}')

count=$(printf '%s\n' "$matches" | grep -c . || true)

if [ "$count" -eq 0 ]; then
  echo "FAIL: zero discovered tests named '$name' in $pkg/$target" >&2
  echo "A zero-test Cargo success is a failed gate." >&2
  exit 1
fi

if [ "$count" -gt 1 ]; then
  echo "FAIL: $count discovered tests named '$name' in $pkg/$target — need a unique name" >&2
  printf '%s\n' "$matches" >&2
  exit 1
fi

echo "OK: exactly one test named '$name'; running --exact"
cargo "${run_args[@]}"
