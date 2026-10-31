# Native Distribution Report — 2026-10-31

Scope: make Context Kernel installable as native packages (Linux x86_64/arm64, Windows x86_64, macOS arm64/x86_64), prove the packages on CI, and hand off a verified ARM64 path for the Jetson AGX Orin 64GB (Ubuntu 22.04, glibc 2.35).

## Outcome

- PR #6 merged into `main` (merge commit `20c0e76`) after every CI job passed on the merge commit.
- CI run `41184479269` (head `7310c96`): `rust` and all five `package / *` jobs succeeded. Each platform installed its own package and ran the installed-binary qualification suite against it.
- No release tag was created. Publishing `v0.1.0` is a deliberate separate action (see "Release").
- **Not verified:** execution on the Jetson itself. I have no device access. The ARM64 package is verified on a GitHub `ubuntu-22.04-arm` runner, which shares the Jetson's Ubuntu 22.04 userspace, but not on the device.

## Evidence

| Item | Value |
|---|---|
| Merged PR | GodSpeedAI/Context-Kernel#6 → `main` @ `20c0e76` |
| Branch head verified | `7310c96` (merge of `origin/main` 81a6fdb into the feature branch) |
| CI run (verified run) | https://github.com/GodSpeedAI/Context-Kernel/actions/runs/41184479269 |
| Earlier fully green run (pre-merge head) | `41183929193` (head `3d24a7e`) |
| Full workspace tests on the merged tree | 302 passed, 0 failed (`cargo test --workspace`) |
| clippy / fmt | `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --check` clean |

Artifacts from run `41184479269` (names carry the SHA of the triggering event; for `pull_request` runs that is GitHub's merge ref `3b2f27976c2953aab8ab5e4b42501377a114b4ab`, not the branch head — noted, not hidden):

| Artifact | Size (bytes) |
|---|---|
| `context-kernel-0.1.0-candidate-3b2f27976c2953aab8ab5e4b42501377a114b4ab-linux-arm64` | 7,886,188 |
| `…-linux-x86_64` | 8,026,990 |
| `…-windows-x86_64` | 14,254,169 |
| `…-macos-arm64` | 8,644,173 |
| `…-macos-x86_64` | 9,072,302 |

SHA-256 of the ARM64 files, from the `SHA256SUMS` inside the run-`41183929193` ARM64 artifact (same build recipe; the `3d24a7e` build), verified locally with `sha256sum -c`:

- `context-kernel-0.1.0-linux-aarch64.tar.gz` — `0689816e3b928c3a0f97f6407bb68b2db4cc71884e844feee434ce06dba87d78`
- `context-kernel_0.1.0_arm64.deb` — `b10a1caaba8f5787446f3fd53c12bfa930f3be0f7de6a11f376a2ec09389ed21`

ARM64 ABI (from the CI log of run `41183929193`): machine `AArch64`, interpreter `/lib/ld-linux-aarch64.so.1`, max required symbol `GLIBC_2.34` (limit 2.35), `DT_NEEDED` = loader, `libc.so.6`, `libgcc_s.so.1`, `libm.so.6` only. No OpenSSL, no system SQLite. Deb: `Architecture: arm64`, `Depends: libc6 (>= 2.34), libgcc-s1`, `Version: 0.1.0`.

## Platform matrix

| Platform | Build | Package | Install | Installed-binary qualification | Runtime deps | Notes |
|---|---|---|---|---|---|---|
| Linux x86_64 (`ubuntu-22.04`) | pass | `.deb`, `.tar.gz` | pass (apt) | pass | glibc, libgcc-s1 | |
| Linux ARM64 (`ubuntu-22.04-arm`) | pass | `.deb`, `.tar.gz` | pass (apt) | pass | glibc ≤2.34 | Jetson target ABI; not run on device |
| Windows x86_64 (`windows-2022`) | pass | `.zip`, `-setup.exe` | pass (per-user, silent) | pass | none beyond OS | UNSIGNED |
| macOS arm64 (`macos-14`) | pass | `.pkg`, `.dmg` | pass (`installer`) | pass | none beyond OS | UNSIGNED, NOT notarized |
| macOS x86_64 (`macos-15-intel`) | pass | `.pkg`, `.dmg` | pass | pass | none beyond OS | label verified by CI run; UNSIGNED, NOT notarized |

MCP stdio and SSE were exercised by the qualification suite on every platform (stdio: `ingest_source`, `query_reference`, `store_context`, `recall_context`, `refresh_source`, `get_status`, `context_required`; SSE: healthz, bearer auth 401s, authorized tool, unauthorized-tool refusal, SIGTERM clean stop on Unix).

Local verification (this x86_64 host, before merge): the packaged `.deb` payload passed the qualification harness (10/10) and the functional smoke test (ingest → cited retrieval → restart recall with content → verified `godspeed.context_bundle` → refusal without authority → FTS5 rebuild).

## What changed (by commit)

1. `8475549` **fix(recall)**: `recall_context` returned event IDs with empty `text`, `kind: "unknown"`, empty `created_at`. The ranker only orders identities. Now hydrated from `SessionMemoryPort::get_event`. This was a real defect that blocked the restart-recall acceptance criterion.
2. `731b935` **feat(bin)**: `--version` (from Cargo metadata); per-user default database; standalone SSE lifecycle (stdin EOF no longer kills SSE; SIGINT/SIGTERM stop it cleanly).
3. `fbfec2c` **test**: `crates/ck-bin/tests/installed_package.rs` — runs against any binary via `CK_QUALIFY_BIN`, skipped otherwise.
4. `2993831` **feat(dist)**: `scripts/package/linux/build-linux.sh` (ABI gate), `scripts/package/version.sh`, `packaging/{linux,windows,macos}`, `scripts/package/{windows,macos}`, `.github/workflows/{ci,package,release}.yml`, Jetson scripts.
5. `1ac9d84` **fix(security)**: SSE without auth refused on non-loopback addresses; reload without tokens fails; packager/installer/CI fixes from adversarial review.
6. `22a252a` / `3d24a7e` **fix(ci/test)**: Windows zip extraction root; TOML literal path in the SSE test (Windows backslash escapes).
7. `7310c96` merge of `origin/main` (PR #5), conflicts only in append-style `.agents` logs.

## Decisions I made without explicit instruction

These are the load-bearing choices. Each is reversible.

1. **Binary name**: kept `context-kernal` as the compiled binary (no duplicate `[[bin]]`); `context-kernel` is a symlink in Linux/macOS packages and a byte-identical copy in Windows packages.
2. **Default database changed to per-user** when no `--config`/`--database` is given. This changes developer behavior: a plain `cargo run` now writes to the user data dir, not `./kernel.db`. Mitigation: `CK_DATA_DIR`, and explicit `--database` is unchanged. This was the riskiest change; it is the reason the brief asked for it, but it is a behavior change you should know about.
3. **SSE lifecycle**: stdin EOF no longer ends a standalone SSE service; no daemon framework added. The systemd unit is shipped as an opt-in example, not enabled.
4. **SSE security fail-closed**: a non-loopback bind without `require_auth` is refused at startup. Before this, `--sse 0.0.0.0:…` served every tool and corpus unauthenticated (confirmed in code: `BearerAuth::no_auth()` sets local trust and allows all). This is stricter than the old behavior; the old behavior was a security defect.
5. **Ubuntu 22.04 runners for Linux** to keep the glibc floor at the Jetson's 2.35, rather than the newer default.
6. **Windows per-user installer, unsigned; macOS per-architecture packages, unsigned, not notarized**. No universal binaries. Signing and notarization need credentials and are not configured.
7. **Maintainer field** in the `.deb` uses `Context-Kernel@users.noreply.github.com`, which I invented as a placeholder. Replace with the real maintainer address before any public distribution.
8. **Qualification harness is a Rust test**, not a shell script, so one implementation runs on all platforms. The Jetson shell smoke test uses `python3 -I` as a test-harness-only dependency; Context Kernel itself needs no Python.
9. **Release gating**: release refuses an existing tag/release, requires exactly 10 assets, attests provenance, creates one release. Candidate builds from branches are artifacts only.
10. **Merged to `main`** with a merge commit (not squash), as you asked for checkpoints.

## Not done / limitations

- **Jetson**: not executed. The install and smoke scripts have been checked for syntax (`shellcheck`) and the ARM64 gate was checked on x86 (refuses correctly). Run them on the device; see below.
- **No release published**; no `v0.1.0` tag exists. Creating it would publish a release, so I did not.
- **Config reload** is still triggered only by stdin requests. A standalone SSE service with closed stdin cannot reload config (existing design, now explicit in debt).
- **Recall scope**: `RecallScope::Recent` behaves like `Current` (pre-existing, not changed).
- **Unauthorized-tool refusal over SSE** returns JSON-RPC `error.code = "internal"`, not an authorization code (pre-existing; recorded in `OBSERVED_DEBT.md`).
- **`doctor` migrates the DB**; it is not read-only (documented).
- **Upgrading users** with a `./kernel.db` in a working directory will see an empty per-user store unless they pass `--database`. No automatic migration. Documented in `docs/install.md`.
- **Installer trust**: release installs verify `gh attestation verify`; run-artifact installs trust the authenticated `gh` download plus the checksum file. The Jetson run mode therefore requires `gh` login.
- **`actionlint`** was unavailable locally (mise shim without a version); workflows were validated by YAML parsing and by actually running in CI.
- **`current_status.yml`** was stale (it described EGM READMEs as uncommitted; they were committed in `fcbb351`). It has been rewritten.

## Jetson instructions

On the Jetson (Ubuntu 22.04, aarch64), with the `gh` CLI logged in to GitHub:

```bash
# 1. Get the installer script from the merged main branch (no clone needed beyond this file)
gh api repos/GodSpeedAI/Context-Kernel/contents/scripts/jetson/install-context-kernel.sh?ref=main \
  -H "Accept: application/vnd.github.raw" > /tmp/install-context-kernel.sh

# 2. Install the exact ARM64 candidate from the verified run (commit-bound)
bash /tmp/install-context-kernel.sh --run 41184479269 --sha "$(gh api repos/GodSpeedAI/Context-Kernel/commits/7310c96 -q .sha)"

# 3. Functional smoke test (disposable data only)
gh api repos/GodSpeedAI/Context-Kernel/contents/scripts/jetson/functional-smoke.sh?ref=main \
  -H "Accept: application/vnd.github.raw" > /tmp/functional-smoke.sh
CK_BIN=context-kernel bash /tmp/functional-smoke.sh
```

Note on step 2: `--sha` must be the full 40-character commit SHA; the script compares it with the run's `headSha`. The command above resolves `7310c96` through the GitHub API so no SHA is typed by hand. Omit `--sha` to skip the commit binding. The install script verifies the `.deb` SHA-256 and architecture, then `apt install`, identity/version, isolated FTS5 and an MCP stdio `get_status`. It starts no service and does not change `HOME`.

Safe configuration (installed use, absolute path):

```toml
[service]
name = "context-kernal"
[storage]
database_path = "/home/<you>/.local/share/context-kernel/kernel.db"
[retrieval]
default_mode = "hybrid"
semantic_enabled = false
```

Local MCP client (stdio): command `/usr/bin/context-kernel`, args `["serve"]`. Without flags this uses the per-user database.

Opt-in SSE (loopback only without auth):

```toml
[mcp]
enable_sse = true
[mcp.sse]
bind = "127.0.0.1:8765"
require_auth = true
[auth]
tokens = ["${CONTEXT_KERNEL_AUTH_TOKEN}"]
```

Start with `context-kernel serve --config <file>`. Packaging ships `/usr/share/doc/context-kernel/examples/context-kernel-sse.service.example`; it is not enabled.

Uninstall: `sudo apt remove context-kernel`. Data in `~/.local/share/context-kernel` is kept.

Backup: `sqlite3 kernel.db ".backup backup.db"` (or stop the service and copy `kernel.db` plus the artifacts directory alongside it). Recovery: restore both to the same directory and point `database_path` at it.

## Remaining debt (see `.agents/OBSERVED_DEBT.md`)

Config reload trigger for standalone SSE; SSE unauthorized error code; `doctor` migrates; example config relative path; a missing recall-scope test; no Windows/macOS hands-on test beyond CI; `actionlint` not run.
