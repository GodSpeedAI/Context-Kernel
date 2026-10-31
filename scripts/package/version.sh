#!/usr/bin/env bash
# Print the single workspace version (Cargo.toml [workspace.package].version).
# Every package and release derives its version from here; nothing is hardcoded.
# With --check-tag vX.Y.Z, exits non-zero unless the tag matches the version.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
version="$(awk '
  /^\[workspace\.package\]/ { in_section = 1; next }
  /^\[/ { in_section = 0 }
  in_section && /^version[[:space:]]*=/ {
    sub(/^[^"]*"/, ""); sub(/".*$/, ""); print; exit
  }
' "$root/Cargo.toml")"

if [[ -z "$version" ]]; then
  echo "error: could not read workspace version from Cargo.toml" >&2
  exit 1
fi

if [[ "${1:-}" == "--check-tag" ]]; then
  tag="${2:?usage: version.sh --check-tag vX.Y.Z}"
  if [[ "$tag" != "v$version" ]]; then
    echo "error: tag $tag does not match workspace version v$version" >&2
    exit 1
  fi
fi

echo "$version"
