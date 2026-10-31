#!/usr/bin/env bash
# Install and verify the Context Kernel ARM64 Debian package on an NVIDIA Jetson
# (Ubuntu 22.04, aarch64). Runs in a subprocess: the EXIT trap only affects this
# script's temporary directory. HOME is not changed and no service is started.
#
# Usage (choose one source):
#   bash install-context-kernel.sh --run <GitHub Actions run id> [--repo OWNER/REPO]
#       Downloads the exact candidate artifact from that run. Needs `gh` logged in.
#   bash install-context-kernel.sh --release <vX.Y.Z> [--repo OWNER/REPO]
#       Downloads the published release asset. Needs `gh`.
# Optional: --sha <full commit sha>  refuses an artifact from any other commit.
set -euo pipefail

REPO="GodSpeedAI/Context-Kernel"
RUN="" RELEASE="" EXPECT_SHA=""

die() { echo "error: $*" >&2; exit 1; }

while [[ $# -gt 0 ]]; do
  case "$1" in
    --run) RUN="$2"; shift 2 ;;
    --release) RELEASE="$2"; shift 2 ;;
    --repo) REPO="$2"; shift 2 ;;
    --sha) EXPECT_SHA="$2"; shift 2 ;;
    *) die "unknown argument: $1" ;;
  esac
done
[[ -n "$RUN" || -n "$RELEASE" ]] || die "pass --run <id> or --release <tag>"
[[ -z "$RUN" || -z "$RELEASE" ]] || die "pass only one of --run or --release"

# 1. Architecture gate.
machine="$(uname -m)"
[[ "$machine" == "aarch64" ]] || die "this package is ARM64 only; uname -m reports '$machine'"
command -v gh >/dev/null || die "the GitHub CLI (gh) is required to download the package"
command -v sha256sum >/dev/null || die "sha256sum is required"
command -v dpkg-deb >/dev/null || die "dpkg-deb is required (Debian/Ubuntu host)"
echo "glibc: $(ldd --version 2>&1 | head -1)"

# 2. Subprocess-scoped temporary directory, removed on any exit.
work="$(mktemp -d /tmp/context-kernel-install.XXXXXX)"
trap 'rm -rf "$work"' EXIT
dl="$work/download"
mkdir -p "$dl"

# 3. Download the exact artifact.
if [[ -n "$RUN" ]]; then
  gh run download "$RUN" --repo "$REPO" --pattern "context-kernel-*-candidate-*-linux-arm64" --dir "$dl"
else
  gh release download "$RELEASE" --repo "$REPO" --pattern "context-kernel_*_arm64.deb" --pattern SHA256SUMS --dir "$dl"
fi
deb="$(find "$dl" -name 'context-kernel_*_arm64.deb' | head -1)"
sums="$(find "$dl" -name 'SHA256SUMS' | head -1)"
[[ -n "$deb" && -n "$sums" ]] || die "download did not contain the .deb and SHA256SUMS"

# 4. Verify SHA-256 against the published checksum list.
(cd "$(dirname "$deb")" && grep " $(basename "$deb")\$" "$sums" | sha256sum -c -)
echo "sha256: verified $(basename "$deb")"

# 5. Verify the Debian architecture and version.
deb_arch="$(dpkg-deb -f "$deb" Architecture)"
deb_version="$(dpkg-deb -f "$deb" Version)"
[[ "$deb_arch" == "arm64" ]] || die "package architecture is '$deb_arch', expected arm64"
echo "package: context-kernel $deb_version ($deb_arch)"

# 6. Optional commit binding (artifact names carry the commit SHA).
if [[ -n "$EXPECT_SHA" && -n "$RUN" ]]; then
  head_sha="$(gh run view "$RUN" --repo "$REPO" --json headSha -q .headSha)"
  [[ "$head_sha" == "$EXPECT_SHA" ]] || die "run $RUN built $head_sha, not $EXPECT_SHA"
fi

# 7. Install with apt so dependencies (libc6, libgcc-s1) are resolved and checked.
sudo apt-get install -y "$deb"
installed="$(command -v context-kernel)"
[[ "$installed" == "/usr/bin/context-kernel" ]] || die "unexpected executable location: $installed"
test -x /usr/bin/context-kernal || die "compatibility name /usr/bin/context-kernal is missing"

# 8. Executable identity and version.
got="$(context-kernel --version)"
echo "identity: $got"
[[ "$got" == *"$deb_version"* ]] || die "--version '$got' does not match package version $deb_version"

# 9. SQLite/FTS5 in an isolated database (never the user's data).
db="$work/isolated/kernel.db"
mkdir -p "$work/isolated"
context-kernel migrate --database "$db"
doctor="$(context-kernel doctor --database "$db")"
echo "doctor: $doctor"
[[ "$doctor" == *"fts5_enabled=true"* ]] || die "FTS5 is not enabled in the packaged binary"

# 10. One MCP stdio request through the installed binary.
req='{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_status","arguments":{}}}'
resp="$(printf '%s\n' "$req" | context-kernel serve --database "$db" --data-dir "$work/isolated")"
echo "stdio: $resp"
[[ "$resp" == *'"id":1'* && "$resp" == *'"healthy":true'* ]] \
  || die "MCP stdio response did not report a healthy service"

# 11. No background service was started by installation.
if systemctl list-units --all --plain --no-legend 2>/dev/null | grep -qi 'context-kernel'; then
  die "a context-kernel systemd unit exists; installation must not start one"
fi

echo "OK: Context Kernel $deb_version installed and verified on $(hostname)."
echo "Next: bash functional-smoke.sh (ingestion, cited retrieval, memory across restart)."
