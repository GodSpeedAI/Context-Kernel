# Install Context Kernel

Context Kernel ships as a single native executable. Installed usage needs **no**
Rust toolchain, no Python, no Docker, no database server, no model download, and no
API key. Installation starts no service and no daemon.

## Executable names

| Name | Status |
|---|---|
| `context-kernel` | Preferred operator command. |
| `context-kernal` | Compatibility name. Existing MCP client configurations keep working. |

Both names run the same program. Linux and macOS packages link `context-kernel`
to `context-kernal`. The Windows packages contain an identical copy of the binary under each name.

## Choose a package

| Platform | Artifact | Install |
|---|---|---|
| Debian/Ubuntu x86_64 | `context-kernel_<v>_amd64.deb` | `sudo apt install ./context-kernel_<v>_amd64.deb` |
| Debian/Ubuntu ARM64 (Jetson 22.04) | `context-kernel_<v>_arm64.deb` | `sudo apt install ./context-kernel_<v>_arm64.deb` |
| Linux (any) | `context-kernel-<v>-linux-<arch>.tar.gz` | Extract and put `bin/` on `PATH`. |
| Windows x86_64 | `context-kernel-<v>-windows-x86_64-setup.exe` | Per-user installer, no administrator rights. |
| Windows x86_64 | `context-kernel-<v>-windows-x86_64.zip` | Extract and run `bin\context-kernel.exe`. |
| macOS Apple silicon | `context-kernel-<v>-macos-arm64.pkg` / `.dmg` | Installs to `/usr/local/bin`. |
| macOS Intel | `context-kernel-<v>-macos-x86_64.pkg` / `.dmg` | Installs to `/usr/local/bin`. |

Every release includes `SHA256SUMS`. Verify before installing:

```bash
sha256sum -c SHA256SUMS --ignore-missing      # Linux
shasum -a 256 -c SHA256SUMS --ignore-missing  # macOS
```

Signing status: the Windows binaries and installer are **unsigned**. The macOS
packages are **unsigned and not notarized**. Expect Gatekeeper or SmartScreen
warnings until signing is added.

Linux packages state their glibc floor in the `Depends` field (`dpkg-deb -f <deb> Depends`). The ARM64 package is built on Ubuntu
22.04. The build fails if any required glibc symbol is newer than 2.35, the NVIDIA Jetson AGX Orin
Ubuntu 22.04 image. The Jetson runtime itself is not yet verified in CI.

## Where data lives

Without `--config` or `--database`, the executable uses one database per user,
independent of the working directory:

| Platform | Default database |
|---|---|
| Linux | `$XDG_DATA_HOME/context-kernel/kernel.db` (default `~/.local/share/context-kernel/kernel.db`) |
| macOS | `~/Library/Application Support/Context Kernel/kernel.db` |
| Windows | `%LOCALAPPDATA%\Context Kernel\kernel.db` |

Artifacts are stored in the same directory as the database.
Set `CK_DATA_DIR` to choose a different directory. An explicit `--database` or
`--config` always wins over the default.

Precedence: `--database` > `--config` (`storage.database_path`) > `CK_DATA_DIR` >
platform default. Relative paths in a config file resolve against the working
directory, so use absolute paths for installed deployments.

Uninstalling a package never removes these data directories.

## Verify an installation

```bash
context-kernel --version
context-kernel migrate --database /tmp/ck-check.db
context-kernel doctor --database /tmp/ck-check.db   # prints fts5_enabled=true
```

`doctor` opens and migrates the database. It is not read-only.

## Run

- **Stdio MCP (default):** `context-kernel serve` reads JSON-RPC on stdin and writes responses on stdout. It exits when stdin closes. Diagnostics go to stderr.
- **SSE MCP (opt-in):** Without `require_auth = true`, SSE is accepted only on a loopback address; any other bind is refused at startup. `context-kernel serve --config <file> --sse 127.0.0.1:8765` (or `enable_sse = true` in config). The listener stays up after stdin closes and stops on SIGINT or SIGTERM. Bearer authentication is required for configured deployments; see [security](security.md).
- **Service manager (opt-in):** `packaging/linux/context-kernel.service.example` is shipped in `/usr/share/doc/context-kernel/examples/`. It is not installed or enabled by the package.

Example configuration files are in `/usr/share/doc/context-kernel/examples/` (Linux) or the `share/doc/context-kernel/examples/` folder of the archives.

## Uninstall

- Debian/Ubuntu: `sudo apt remove context-kernel`
- macOS: `sudo rm /usr/local/bin/context-kernel /usr/local/bin/context-kernal` and remove `/usr/local/share/doc/context-kernel`. See `UNINSTALL.md` in the package.
- Windows: Apps and Features, or the uninstaller in the install folder.
- Archives: delete the extracted folder.

User data (`kernel.db`, artifacts) is kept in every case. Remove it yourself only if you
intend to discard the memory.

## Build from source (developers)

```bash
cargo build --release --locked -p ck-bin
./target/release/context-kernal --version
```

Development builds keep the same defaults, so running from a checkout without flags
now uses the per-user database rather than `./kernel.db`. Pass `--database ./kernel.db`
to keep the old location.
