# Installation

## Quick install (recommended)

```sh
curl -fsSL https://raw.githubusercontent.com/premananda-cloud/vin/main/install.sh | sh
```

If your default branch is not `main`, change it in the URL. The same script is also attached to every release:

```sh
curl -fsSL https://github.com/premananda-cloud/vin/releases/latest/download/install.sh | sh
```

The installer:

1. Detects your platform (Linux or macOS, x86_64 or arm64).
2. Downloads the matching prebuilt binary from the latest GitHub release and **verifies its SHA-256 checksum** against the release's `SHA256SUMS`. A mismatch aborts the install.
3. Installs it to `~/.local/bin/vin`.
4. **Falls back to building from source** (needs [Rust](https://rustup.rs)) if there is no prebuilt binary for your platform or release.

It then tells you if `~/.local/bin` is missing from your `PATH`, or if another `vin` (for example one from an older `cargo install`) would shadow the new one.

**Upgrading** is the same command: run it again.

### Options

Pass options after `sh -s --` when piping:

```sh
curl -fsSL https://raw.githubusercontent.com/premananda-cloud/vin/main/install.sh | sh -s -- --version v1.2.0
```

| Option | Meaning |
|--------|---------|
| `--version <TAG>` | Install a specific release (e.g. `v1.2.0`) instead of the latest |
| `--bin-dir <DIR>` | Install into `DIR` (default `~/.local/bin`, or `$VIN_BIN_DIR`) |
| `--from-source` | Build with cargo instead of downloading a binary |
| `--no-verify` | Skip the checksum check (not recommended) |
| `--uninstall` | Remove the installed binary |
| `-h`, `--help` | Show help |

Prefer to read the script before running it? Download it first:

```sh
curl -fsSLO https://raw.githubusercontent.com/premananda-cloud/vin/main/install.sh
less install.sh
sh install.sh
```

## Manual download

1. Open the [releases page](https://github.com/premananda-cloud/vin/releases) and download `vin-<target>.tar.gz` for your platform plus `SHA256SUMS`.

   | Platform | `<target>` |
   |----------|-----------|
   | Linux x86_64 | `x86_64-unknown-linux-musl` |
   | Linux arm64 | `aarch64-unknown-linux-musl` |
   | macOS Intel | `x86_64-apple-darwin` |
   | macOS Apple Silicon | `aarch64-apple-darwin` |

2. Verify and install:

   ```sh
   sha256sum --check --ignore-missing SHA256SUMS        # macOS: shasum -a 256 -c SHA256SUMS
   tar -xzf vin-x86_64-unknown-linux-musl.tar.gz
   install -m 755 vin-x86_64-unknown-linux-musl/vin ~/.local/bin/vin
   ```

The Linux binaries are statically linked (musl), so they run on any distribution.

## Build from source

With a clone of the repository:

```sh
git clone https://github.com/premananda-cloud/vin
cd vin
./install.sh --from-source        # builds and installs to ~/.local/bin
```

or directly with cargo:

```sh
cargo install --path .            # installs to ~/.cargo/bin
cargo install --git https://github.com/premananda-cloud/vin --locked
```

Requires a recent stable Rust toolchain ([rustup.rs](https://rustup.rs)).

## Uninstall

```sh
install.sh --uninstall            # or: rm ~/.local/bin/vin
cargo uninstall vin               # if installed with cargo
```

Your settings and custom templates live in the vin [config directory](configuration.md) and are not removed. Delete that directory yourself if you want a clean slate.

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| `vin: command not found` | Add `export PATH="$HOME/.local/bin:$PATH"` to `~/.bashrc` or `~/.zshrc` and open a new terminal |
| `vin --version` shows an old version | Another `vin` is earlier on `PATH`; run `which -a vin` and remove the stale one (`cargo uninstall vin`) |
| `no prebuilt binary found` then a long build | That release has no binary for your platform; the installer builds from source instead (needs cargo) |
| `checksum mismatch` | The download was corrupted or tampered with. Retry; if it persists, build from source |
| `cargo not found` | Install Rust from [rustup.rs](https://rustup.rs), then re-run the installer |
