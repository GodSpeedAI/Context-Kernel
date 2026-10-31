#!/usr/bin/env bash
# Build macOS .pkg and .dmg for Context Kernel (per-architecture, no universal binary).
#
# The .pkg and .dmg are UNSIGNED and NOT notarized. Gatekeeper will warn on
# first open; users must approve manually.
#
# Usage:
#   build-macos.sh --arch arm64|x86_64 --binary <path> --version <v> --outdir <dir> [--dry-run]
set -euo pipefail

ARCH="" BINARY="" VERSION="" OUTDIR="" DRY_RUN=0

usage() {
  echo "usage: $0 --arch arm64|x86_64 --binary <path> --version <v> --outdir <dir> [--dry-run]" >&2
  exit 2
}

while [ $# -gt 0 ]; do
  case "$1" in
    --arch) ARCH="${2:-}"; shift 2 ;;
    --binary) BINARY="${2:-}"; shift 2 ;;
    --version) VERSION="${2:-}"; shift 2 ;;
    --outdir) OUTDIR="${2:-}"; shift 2 ;;
    --dry-run) DRY_RUN=1; shift ;;
    *) usage ;;
  esac
done

case "$ARCH" in arm64 | x86_64) ;; *) usage ;; esac
[ -n "$BINARY" ] && [ -n "$VERSION" ] && [ -n "$OUTDIR" ] || usage

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
PKG_ID="com.godspeedai.context-kernel"

SRC_README="$REPO_ROOT/README.md"
SRC_LICENSE="$REPO_ROOT/LICENSE"
SRC_EX1="$REPO_ROOT/config/context-kernal.example.toml"
SRC_EX2="$REPO_ROOT/config/context-kernal.sse.example.toml"
SRC_UNINSTALL="$REPO_ROOT/packaging/macos/UNINSTALL.md"

for f in "$BINARY" "$SRC_README" "$SRC_LICENSE" "$SRC_EX1" "$SRC_EX2" "$SRC_UNINSTALL"; do
  [ -f "$f" ] || { echo "missing input: $f" >&2; exit 1; }
done

PKG_NAME="context-kernel-${VERSION}-macos-${ARCH}.pkg"
DMG_NAME="context-kernel-${VERSION}-macos-${ARCH}.dmg"

# stage_layout <root>: populate the install root (maps to /usr/local).
stage_layout() {
  local root="$1" doc="$1/share/doc/context-kernel"
  mkdir -p "$root/bin" "$doc/examples"
  install -m 0755 "$BINARY" "$root/bin/context-kernal"
  ln -s context-kernal "$root/bin/context-kernel"
  install -m 0644 "$SRC_README" "$doc/README.md"
  install -m 0644 "$SRC_LICENSE" "$doc/LICENSE"
  install -m 0644 "$SRC_EX1" "$doc/examples/context-kernal.example.toml"
  install -m 0644 "$SRC_EX2" "$doc/examples/context-kernal.sse.example.toml"
  install -m 0644 "$SRC_UNINSTALL" "$doc/UNINSTALL.md"
}

if [ "$DRY_RUN" -eq 1 ]; then
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  stage_layout "$tmp/root"
  echo "would build: $OUTDIR/$PKG_NAME (identifier $PKG_ID, install-location /usr/local)"
  echo "would build: $OUTDIR/$DMG_NAME"
  echo "planned files (relative to /usr/local):"
  (cd "$tmp/root" && find . -mindepth 1 | sort | sed 's|^\./||')
  exit 0
fi

command -v pkgbuild >/dev/null || { echo "pkgbuild not found (macOS required)" >&2; exit 1; }
command -v hdiutil >/dev/null || { echo "hdiutil not found (macOS required)" >&2; exit 1; }

mkdir -p "$OUTDIR"
OUTDIR="$(cd "$OUTDIR" && pwd)"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

ROOT="$WORK/root"
stage_layout "$ROOT"

pkgbuild \
  --root "$ROOT" \
  --identifier "$PKG_ID" \
  --version "$VERSION" \
  --install-location /usr/local \
  "$OUTDIR/$PKG_NAME"

# Future signing hook (DISABLED). Needs a Developer ID Installer certificate.
# productsign --sign "Developer ID Installer: <NAME> (<TEAMID>)" \
#   "$OUTDIR/$PKG_NAME" "$OUTDIR/signed-$PKG_NAME"
# xcrun notarytool submit "$OUTDIR/signed-$PKG_NAME" --keychain-profile "<PROFILE>" --wait
# xcrun stapler staple "$OUTDIR/signed-$PKG_NAME"

# DMG contents: the pkg, the uninstall note, and the license.
DMGSRC="$WORK/dmg"
mkdir -p "$DMGSRC"
cp "$OUTDIR/$PKG_NAME" "$DMGSRC/"
cp "$SRC_UNINSTALL" "$DMGSRC/UNINSTALL.md"
cp "$SRC_LICENSE" "$DMGSRC/LICENSE"

rm -f "$OUTDIR/$DMG_NAME"
hdiutil create -volname "Context Kernel $VERSION" -srcfolder "$DMGSRC" -format UDZO -ov "$OUTDIR/$DMG_NAME"

# Append checksums (replace any stale entries for these files).
SUMS="$OUTDIR/SHA256SUMS"
touch "$SUMS"
grep -v -F -e "  $PKG_NAME" -e "  $DMG_NAME" "$SUMS" > "$WORK/sums" || true
(cd "$OUTDIR" && shasum -a 256 "$PKG_NAME" "$DMG_NAME") >> "$WORK/sums"
cp "$WORK/sums" "$SUMS"

echo "Wrote $OUTDIR/$PKG_NAME"
echo "Wrote $OUTDIR/$DMG_NAME"
echo "Updated $SUMS"
