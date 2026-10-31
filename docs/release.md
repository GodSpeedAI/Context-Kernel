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

## crates.io publication

The `context-kernel` CLI and the 15 internal crates it needs are published to crates.io as one lockstep set at the workspace version. Every internal dependency is pinned exactly (`version = "=X.Y.Z"`). Three workspace crates (`ck-scheduler`, `ck-adapter-github`, `ck-adapter-web`) are `publish = false` and never reach crates.io.

- `scripts/package/crates/publish-order.txt` lists the publication order. `scripts/package/crates/publish.sh --check` verifies that it equals the dependency closure of `context-kernel`, that every crate has metadata and an exact pin, and that no unrelated crate is publishable. It then runs `cargo publish --workspace --dry-run`, which packages and verifies each crate in order. `ci.yml` runs this check on every change.
- `publish.sh --publish` (release workflow only) publishes missing versions in order. A version already on crates.io is skipped only when its checksum matches this commit; a mismatch stops the run. Each crate writes a receipt to `dist/crates-receipts/`, uploaded as an artifact.
- Credentials: the `CARGO_REGISTRY_TOKEN` repository secret is exposed only to the publish step. Never print it.
- Runs in parallel with the native GitHub Release, not after it. The `channel-report` job prints each channel's result separately and fails if any channel did not succeed.

Crates cannot be deleted from crates.io. Recover from a bad release with a new version. `cargo yank` only stops new dependency resolution.

## Verify a published release

```bash
gh release download vX.Y.Z --repo GodSpeedAI/Context-Kernel --pattern SHA256SUMS
gh release download vX.Y.Z --repo GodSpeedAI/Context-Kernel --pattern '*arm64*.deb'
sha256sum -c SHA256SUMS --ignore-missing
gh attestation verify context-kernel_X.Y.Z_arm64.deb --repo GodSpeedAI/Context-Kernel
```

## Local packaging (developers)

```bash
cargo build --release --locked -p context-kernel
bash scripts/package/linux/build-linux.sh --binary target/release/context-kernal \
  --arch amd64 --version "$(bash scripts/package/version.sh)" --outdir dist
```

The Linux builder refuses a binary whose glibc floor is above `CK_MAX_GLIBC` (default
2.35) or that links a shared library outside the allowlist.

## Qualify an installed package

The same harness the pipeline uses runs against any installed executable:

```bash
CK_QUALIFY_BIN=/usr/bin/context-kernel CK_QUALIFY_VERSION=X.Y.Z \
  cargo test --release -p context-kernel --test installed_package
```

Without `CK_QUALIFY_BIN` the tests are skipped, so `cargo test --workspace` stays hermetic.
