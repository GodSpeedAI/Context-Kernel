# Release Process

Release artifacts are produced by GitHub Actions, not by hand. The pipeline is in
`.github/workflows/`:

| Workflow | Trigger | Produces |
|---|---|---|
| `ci.yml` | every push and pull request | Rust gates, then commit-bound candidate packages as **workflow artifacts** (30-day retention). No release. |
| `package.yml` | called by `ci.yml` and `release.yml` | Native packages per platform, installed and qualified on that platform. |
| `release.yml` | push of a `v*` tag | Verified tag, gates, qualified packages, `SHA256SUMS`, build provenance attestations, and **one** GitHub Release. |

## Platform matrix

| Artifact | Runner | Notes |
|---|---|---|
| Linux x86_64 `.deb`, `.tar.gz` | `ubuntu-22.04` | glibc floor from the build; checked by `build-linux.sh`. |
| Linux ARM64 `.deb`, `.tar.gz` | `ubuntu-22.04-arm` | Built on the Jetson's Ubuntu 22.04 ABI (glibc 2.35 maximum). |
| Windows x86_64 `.zip`, `-setup.exe` | `windows-2022` | Per-user installer. Unsigned. |
| macOS arm64 `.pkg`, `.dmg` | `macos-14` | Unsigned, not notarized. |
| macOS x86_64 `.pkg`, `.dmg` | `macos-15-intel` | Unsigned, not notarized. Confirm the label is still served. |

Each architecture is built on its own native runner. There are no universal macOS
binaries and no cross-compiled artifacts.

## Cut a release

1. Set `[workspace.package].version` in `Cargo.toml` (and `Cargo.lock`) to the release version.
2. Merge to `main` with CI green.
3. Create an annotated tag `vX.Y.Z` on that commit and push it. Do not move or re-push a tag.
4. `release.yml` will:
   - fail if the tag does not equal `v` + the workspace version (`scripts/package/version.sh --check-tag`);
   - fail if a release with that tag already exists;
   - run `fmt`, `clippy -D warnings`, and `cargo test --workspace`;
   - build, package, install, and qualify every platform;
   - merge the per-platform `SHA256SUMS`, verify them, and require exactly 10 assets;
   - attest build provenance for the artifacts;
   - create one release with all assets.

If any platform fails, nothing is published. Fix forward with a new version tag; do not
delete or move a published tag.

## Verify a published release

```bash
gh release download vX.Y.Z --repo GodSpeedAI/Context-Kernel --pattern SHA256SUMS
gh release download vX.Y.Z --repo GodSpeedAI/Context-Kernel --pattern '*arm64*.deb'
sha256sum -c SHA256SUMS --ignore-missing
gh attestation verify context-kernel_X.Y.Z_arm64.deb --repo GodSpeedAI/Context-Kernel
```

## Local packaging (developers)

```bash
cargo build --release --locked -p ck-bin
bash scripts/package/linux/build-linux.sh --binary target/release/context-kernal \
  --arch amd64 --version "$(bash scripts/package/version.sh)" --outdir dist
```

The Linux builder refuses a binary whose glibc floor is above `CK_MAX_GLIBC` (default
2.35) or that links a shared library outside the allowlist.

## Qualify an installed package

The same harness the pipeline uses runs against any installed executable:

```bash
CK_QUALIFY_BIN=/usr/bin/context-kernel CK_QUALIFY_VERSION=X.Y.Z \
  cargo test --release -p ck-bin --test installed_package
```

Without `CK_QUALIFY_BIN` the tests are skipped, so `cargo test --workspace` stays hermetic.
