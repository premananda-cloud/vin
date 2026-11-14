#!/bin/sh
# vin installer
#
#   curl -fsSL https://raw.githubusercontent.com/premananda-cloud/vin/main/install.sh | sh
#   curl -fsSL https://raw.githubusercontent.com/premananda-cloud/vin/main/install.sh | sh -s -- --version v1.2.0
#
# Downloads a prebuilt binary from GitHub Releases (verified against SHA256SUMS).
# If no binary exists for this platform or release, it builds from source with cargo.
# Run `install.sh --help` for options. Re-running the script upgrades vin.

set -eu

REPO="${VIN_REPO:-premananda-cloud/vin}"
RELEASES_URL="${VIN_RELEASES_URL:-https://github.com/$REPO/releases}"
GIT_URL="${VIN_GIT_URL:-https://github.com/$REPO}"
BIN_NAME="vin"

BIN_DIR="${VIN_BIN_DIR:-}"
VERSION=""
FROM_SOURCE=0
UNINSTALL=0
VERIFY=1

# ---------------------------------------------------------------- output ----
if [ -t 2 ] && [ -z "${NO_COLOR:-}" ]; then
    BOLD=$(printf '\033[1m'); YELLOW=$(printf '\033[33m'); RED=$(printf '\033[31m'); RESET=$(printf '\033[0m')
else
    BOLD=""; YELLOW=""; RED=""; RESET=""
fi
say()  { printf '%s\n' "$*" >&2; }
step() { printf '%s==>%s %s\n' "$BOLD" "$RESET" "$*" >&2; }
warn() { printf '%swarning:%s %s\n' "$YELLOW" "$RESET" "$*" >&2; }
die()  { printf '%serror:%s %s\n' "$RED" "$RESET" "$*" >&2; exit 1; }
have() { command -v "$1" >/dev/null 2>&1; }

usage() {
    cat <<EOF
Install vin (research paper workspace initialiser).

Usage: install.sh [OPTIONS]

Options:
  --version <TAG>    Install a specific release (e.g. v1.2.0). Default: latest.
  --bin-dir <DIR>    Install into DIR. Default: ~/.local/bin (or \$VIN_BIN_DIR).
  --from-source      Skip the prebuilt binary and build with cargo.
  --no-verify        Skip the SHA256 checksum check (not recommended).
  --uninstall        Remove the installed binary.
  -h, --help         Show this help.

By default a prebuilt binary is downloaded and checksum-verified. If none is
available for your platform, vin is built from source (requires Rust/cargo,
see https://rustup.rs). Running this script from a clone of the repository
builds that checkout.

Environment:
  VIN_BIN_DIR        Same as --bin-dir.
  VIN_REPO           GitHub "owner/name" to install from (default: $REPO).
  VIN_RELEASES_URL   Override the releases base URL (for mirrors and testing).
EOF
}

# ------------------------------------------------------------- arguments ----
while [ $# -gt 0 ]; do
    case "$1" in
        --version)     [ $# -ge 2 ] || die "--version needs a value"; VERSION="$2"; shift 2 ;;
        --version=*)   VERSION="${1#*=}"; shift ;;
        --bin-dir)     [ $# -ge 2 ] || die "--bin-dir needs a value"; BIN_DIR="$2"; shift 2 ;;
        --bin-dir=*)   BIN_DIR="${1#*=}"; shift ;;
        --from-source) FROM_SOURCE=1; shift ;;
        --no-verify)   VERIFY=0; shift ;;
        --uninstall)   UNINSTALL=1; shift ;;
        -h|--help)     usage; exit 0 ;;
        *)             die "unknown option: $1 (try --help)" ;;
    esac
done

case "$VERSION" in
    ""|v*) ;;
    *) VERSION="v$VERSION" ;;
esac
if [ -z "$BIN_DIR" ]; then
    [ -n "${HOME:-}" ] || die "cannot determine the install directory (HOME is not set); use --bin-dir"
    BIN_DIR="$HOME/.local/bin"
fi

DEST="$BIN_DIR/$BIN_NAME"

# ------------------------------------------------------------- uninstall ----
if [ "$UNINSTALL" = 1 ]; then
    if [ -f "$DEST" ]; then
        rm -f "$DEST" || die "could not remove $DEST"
        step "Removed $DEST"
    else
        step "Nothing to remove: $DEST does not exist"
    fi
    say "Your vin settings and custom templates (the vin config directory) were left in place."
    exit 0
fi

# ------------------------------------------------------------ workspace -----
TMP=$(mktemp -d 2>/dev/null || mktemp -d -t vin-install) || die "could not create a temporary directory"
trap 'rm -rf "$TMP"' EXIT
trap 'exit 1' INT TERM HUP

SRC_BIN=""

# ---------------------------------------------------------------- prebuilt --
detect_target() {
    os=$(uname -s); arch=$(uname -m)
    case "$os" in
        Linux)  os_part="unknown-linux-musl" ;;
        Darwin) os_part="apple-darwin" ;;
        *) return 1 ;;
    esac
    case "$arch" in
        x86_64|amd64)  arch_part="x86_64" ;;
        aarch64|arm64) arch_part="aarch64" ;;
        *) return 1 ;;
    esac
    TARGET="$arch_part-$os_part"
}

download() { # url dest
    if have curl; then
        curl -fsL --retry 2 -o "$2" "$1"
    elif have wget; then
        wget -q -O "$2" "$1"
    else
        return 2
    fi
}

sha256_of() {
    if have sha256sum; then
        sha256sum "$1" | cut -d ' ' -f 1
    elif have shasum; then
        shasum -a 256 "$1" | cut -d ' ' -f 1
    elif have openssl; then
        openssl dgst -sha256 "$1" | sed 's/^.*= //'
    else
        return 1
    fi
}

# Returns 0 and sets SRC_BIN on success; returns 1 when no prebuilt binary is
# available (caller falls back to source). Integrity failures abort outright.
install_prebuilt() {
    if ! detect_target; then
        warn "no prebuilt binary for $(uname -s)/$(uname -m)"
        return 1
    fi
    if ! have curl && ! have wget; then
        warn "neither curl nor wget found, cannot download a prebuilt binary"
        return 1
    fi

    file="$BIN_NAME-$TARGET.tar.gz"
    if [ -n "$VERSION" ]; then
        base="$RELEASES_URL/download/$VERSION"
    else
        base="$RELEASES_URL/latest/download"
    fi

    step "Downloading $file (${VERSION:-latest release})"
    if ! download "$base/$file" "$TMP/$file"; then
        warn "no prebuilt binary found at $base/$file"
        return 1
    fi

    if [ "$VERIFY" = 1 ]; then
        download "$base/SHA256SUMS" "$TMP/SHA256SUMS" ||
            die "could not download SHA256SUMS to verify the download (use --no-verify to skip)"
        expected=$(awk -v f="$file" '{ n = $2; sub(/^\*/, "", n); if (n == f) { print $1; exit } }' "$TMP/SHA256SUMS")
        [ -n "$expected" ] || die "$file is not listed in SHA256SUMS"
        actual=$(sha256_of "$TMP/$file") ||
            die "no sha256 tool found (install sha256sum, shasum or openssl, or use --no-verify)"
        [ "$expected" = "$actual" ] ||
            die "checksum mismatch for $file (expected $expected, got $actual); refusing to install"
        step "Checksum OK"
    else
        warn "skipping checksum verification"
    fi

    mkdir "$TMP/extract" || die "could not create a temporary directory"
    tar -xzf "$TMP/$file" -C "$TMP/extract" || die "could not extract $file"
    SRC_BIN=$(find "$TMP/extract" -type f -name "$BIN_NAME" | head -n 1)
    [ -n "$SRC_BIN" ] || die "the archive did not contain a '$BIN_NAME' binary"
    return 0
}

# ------------------------------------------------------------------ source --
build_from_source() {
    have cargo || die "cargo not found. Install Rust from https://rustup.rs and re-run this script, or download a prebuilt binary from $RELEASES_URL"

    # Build the local checkout only when no specific version was requested;
    # a pinned --version always builds that tag from GitHub.
    here=""
    if [ -z "$VERSION" ] && [ -f "$0" ]; then
        here=$(CDPATH='' cd -- "$(dirname -- "$0")" 2>/dev/null && pwd) || here=""
    fi
    if [ -n "$here" ] && [ -f "$here/Cargo.toml" ] && grep -q "^name = \"$BIN_NAME\"" "$here/Cargo.toml"; then
        step "Building from the local checkout at $here (this takes a minute)"
        cargo build --release --locked --manifest-path "$here/Cargo.toml" --target-dir "$TMP/target" ||
            die "build failed"
        SRC_BIN="$TMP/target/release/$BIN_NAME"
    else
        step "Building from $GIT_URL (${VERSION:-default branch}); this takes a minute"
        if [ -n "$VERSION" ]; then
            cargo install --git "$GIT_URL" --tag "$VERSION" --locked --root "$TMP/cargo-root" || die "build failed"
        else
            cargo install --git "$GIT_URL" --locked --root "$TMP/cargo-root" || die "build failed"
        fi
        SRC_BIN="$TMP/cargo-root/bin/$BIN_NAME"
    fi
    [ -f "$SRC_BIN" ] || die "build finished but no '$BIN_NAME' binary was produced"
}

# ------------------------------------------------------------------- main ---
if [ "$FROM_SOURCE" = 1 ]; then
    build_from_source
elif ! install_prebuilt; then
    warn "falling back to building from source"
    build_from_source
fi

mkdir -p "$BIN_DIR" || die "cannot create $BIN_DIR"
tmpdest="$BIN_DIR/.$BIN_NAME.tmp.$$"
if ! { cp "$SRC_BIN" "$tmpdest" && chmod 755 "$tmpdest" && mv -f "$tmpdest" "$DEST"; }; then
    rm -f "$tmpdest"
    die "could not write to $BIN_DIR (choose another folder with --bin-dir)"
fi

installed=$("$DEST" --version 2>/dev/null) || die "installed $DEST but it does not run on this system"
step "Installed $installed -> $DEST"

# A different `vin` earlier on PATH would silently win; tell the user.
other=$(command -v "$BIN_NAME" 2>/dev/null || true)
case ":$PATH:" in
    *":$BIN_DIR:"*) on_path=1 ;;
    *) on_path=0 ;;
esac
if [ "$on_path" = 0 ]; then
    warn "$BIN_DIR is not on your PATH. Add this line to your shell profile (~/.bashrc, ~/.zshrc):"
    say "    export PATH=\"$BIN_DIR:\$PATH\""
fi
if [ -n "$other" ] && [ "$other" != "$DEST" ]; then
    warn "'$BIN_NAME' currently resolves to $other, which takes priority over the new install."
    say "    Remove it (for a cargo install: cargo uninstall $BIN_NAME) or put $BIN_DIR first on your PATH."
fi

say ""
say "Try: vin list"
