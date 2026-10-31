#!/usr/bin/env bash
# Build the Linux packages for one architecture from an already-built binary:
#   context-kernel-<version>-linux-<arch>.tar.gz
#   context-kernel_<version>_<deb-arch>.deb
# and append SHA-256 lines to SHA256SUMS in the output directory.
#
# Compatibility gate (fails the build, does not warn):
#   * ELF machine type must match --arch.
#   * Highest GLIBC_x.y symbol version must be <= CK_MAX_GLIBC (default 2.35,
#     Ubuntu 22.04 / the NVIDIA Jetson target). Override only deliberately.
#   * DT_NEEDED may list only libc.so.6, libm.so.6, libgcc_s.so.1 and the
#     dynamic loader. Anything else (e.g. libssl, libsqlite3) fails the build.
set -euo pipefail

usage() {
  echo "usage: $0 --binary <context-kernal> --arch amd64|arm64 --version <v> --outdir <dir>" >&2
  exit 2
}

binary="" arch="" version="" outdir=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --binary) binary="$2"; shift 2 ;;
    --arch) arch="$2"; shift 2 ;;
    --version) version="$2"; shift 2 ;;
    --outdir) outdir="$2"; shift 2 ;;
    *) usage ;;
  esac
done
[[ -n "$binary" && -n "$arch" && -n "$version" && -n "$outdir" ]] || usage
[[ -f "$binary" ]] || { echo "error: binary not found: $binary" >&2; exit 1; }

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
max_glibc="${CK_MAX_GLIBC:-2.35}"

case "$arch" in
  amd64) elf_machine="Advanced Micro Devices X86-64"; tar_arch="x86_64" ;;
  arm64) elf_machine="AArch64"; tar_arch="aarch64" ;;
  *) echo "error: unsupported --arch $arch (amd64|arm64)" >&2; exit 2 ;;
esac

# ---- ABI verification -------------------------------------------------------
machine="$(readelf -h "$binary" | awk -F: '/Machine:/ {sub(/^ +/, "", $2); print $2}')"
if [[ "$machine" != "$elf_machine" ]]; then
  echo "error: ELF machine '$machine' does not match arch $arch ('$elf_machine')" >&2
  exit 1
fi

interp="$(readelf -l "$binary" | awk '/Requesting program interpreter/ {gsub(/[\[\]]/, ""); print $NF}')"
max_sym="$(objdump -T "$binary" | { grep -o 'GLIBC_[0-9][0-9.]*' || true; } | sort -uV | tail -1 | sed 's/^GLIBC_//')"
if [[ -z "$max_sym" ]]; then
  echo "error: no GLIBC symbol versions found; binary is not a dynamically linked glibc build" >&2
  exit 1
fi
if [[ "$(printf '%s\n%s\n' "$max_sym" "$max_glibc" | sort -V | tail -1)" != "$max_glibc" ]]; then
  echo "error: binary requires GLIBC_$max_sym, above the allowed maximum $max_glibc" >&2
  exit 1
fi

needed="$(readelf -d "$binary" | awk '/NEEDED/ {gsub(/[\[\]]/, ""); print $NF}' | sort -u)"
allowed_re='^(libc\.so\.6|libm\.so\.6|libgcc_s\.so\.1|ld-linux-(x86-64|aarch64)\.so\.[0-9]+)$'
while IFS= read -r lib; do
  [[ -z "$lib" ]] && continue
  if [[ ! "$lib" =~ $allowed_re ]]; then
    echo "error: unexpected shared library dependency: $lib" >&2
    exit 1
  fi
done <<< "$needed"

# Debian dependency on libc is the verified maximum symbol version.
debian_libc_dep="libc6 (>= $max_sym)"

# ---- Stage ------------------------------------------------------------------
name="context-kernel-$version-linux-$tar_arch"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
stage="$work/$name"

install -d "$stage/bin" "$stage/share/doc/context-kernel/examples"
install -m 0755 "$binary" "$stage/bin/context-kernal"
ln -s context-kernal "$stage/bin/context-kernel"
install -m 0644 "$root/README.md" "$stage/share/doc/context-kernel/README.md"
install -m 0644 "$root/LICENSE" "$stage/share/doc/context-kernel/LICENSE"
install -m 0644 "$root/docs/install.md" "$stage/share/doc/context-kernel/INSTALL.md"
install -m 0644 "$root/config/context-kernal.example.toml" \
  "$stage/share/doc/context-kernel/examples/context-kernal.example.toml"
install -m 0644 "$root/config/context-kernal.sse.example.toml" \
  "$stage/share/doc/context-kernel/examples/context-kernal.sse.example.toml"
install -m 0644 "$root/packaging/linux/context-kernel.service.example" \
  "$stage/share/doc/context-kernel/examples/context-kernel-sse.service.example"

mkdir -p "$outdir"
outdir="$(cd "$outdir" && pwd)"

# ---- Tar archive (reproducible ordering, root ownership) --------------------
tarball="$outdir/$name.tar.gz"
(cd "$work" && tar --sort=name --owner=0 --group=0 --numeric-owner \
  --mtime='@0' -czf "$tarball" "$name")

# ---- Debian package ---------------------------------------------------------
deb_root="$work/deb"
deb_name="context-kernel_${version}_${arch}.deb"
install -d "$deb_root/DEBIAN" "$deb_root/usr/bin" \
  "$deb_root/usr/share/doc/context-kernel/examples"
install -m 0755 "$binary" "$deb_root/usr/bin/context-kernal"
ln -s context-kernal "$deb_root/usr/bin/context-kernel"
for f in README.md LICENSE; do
  install -m 0644 "$stage/share/doc/context-kernel/$f" "$deb_root/usr/share/doc/context-kernel/$f"
done
install -m 0644 "$stage/share/doc/context-kernel/INSTALL.md" "$deb_root/usr/share/doc/context-kernel/INSTALL.md"
cp "$stage/share/doc/context-kernel/examples/"* "$deb_root/usr/share/doc/context-kernel/examples/"
install -m 0644 "$root/packaging/linux/copyright" "$deb_root/usr/share/doc/context-kernel/copyright"

installed_kib="$(du -sk "$deb_root/usr" | cut -f1)"
cat > "$deb_root/DEBIAN/control" <<EOF
Package: context-kernel
Version: $version
Section: utils
Priority: optional
Architecture: $arch
Maintainer: GodSpeedAI Context Kernel <Context-Kernel@users.noreply.github.com>
Installed-Size: $installed_kib
Depends: $debian_libc_dep, libgcc-s1
Homepage: https://github.com/GodSpeedAI/Context-Kernel
Description: Context Kernel MCP context assembly and memory service
 Single native executable providing MCP stdio and optional SSE transports,
 SQLite/FTS5 lexical search, durable session memory, filesystem artifacts,
 and canonical context bundles. Installs no service and starts no daemon.
 The binary is installed as context-kernel and as the compatibility name
 context-kernal. User data is kept under the per-user data directory on
 removal.
EOF

dpkg-deb --build --root-owner-group "$deb_root" "$outdir/$deb_name" >/dev/null

# ---- Checksums and report ---------------------------------------------------
(cd "$outdir" && sed -i "\|  $name.tar.gz\$|d;\|  $deb_name\$|d" SHA256SUMS 2>/dev/null || true; sha256sum "$name.tar.gz" "$deb_name" >> SHA256SUMS)
echo "built: $tarball"
echo "built: $outdir/$deb_name"
echo "abi: machine='$machine' interpreter='$interp' max_glibc=GLIBC_$max_sym"
echo "needed: $(echo "$needed" | tr '\n' ' ')"
