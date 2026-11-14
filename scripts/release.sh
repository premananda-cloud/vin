#!/bin/sh
# Cut a vin release: bump the version, run checks, commit, tag, and (optionally) push.
#
#   scripts/release.sh 1.2.0          bump + test + commit + tag (local only)
#   scripts/release.sh 1.2.0 --push   ...and push the commit and tag to origin
#
# Pushing a `v*` tag triggers .github/workflows/release.yml, which builds the
# binaries and publishes the GitHub release.

set -eu

die() { printf 'error: %s\n' "$*" >&2; exit 1; }

usage() {
    cat <<'EOF'
Usage: scripts/release.sh <version> [--push]

  <version>   semantic version without the leading "v" (e.g. 1.2.0)
  --push      push the release commit and tag to origin (default: print the command)
EOF
}

VERSION=""
PUSH=0
for arg in "$@"; do
    case "$arg" in
        --push) PUSH=1 ;;
        -h|--help) usage; exit 0 ;;
        -*) die "unknown option: $arg" ;;
        *) [ -z "$VERSION" ] || die "only one version may be given"; VERSION="$arg" ;;
    esac
done
[ -n "$VERSION" ] || { usage >&2; exit 1; }
VERSION="${VERSION#v}"
printf '%s' "$VERSION" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.]+)?$' ||
    die "'$VERSION' is not a valid version (expected e.g. 1.2.0)"
TAG="v$VERSION"

cd "$(dirname "$0")/.."
git rev-parse --git-dir >/dev/null 2>&1 || die "not a git repository"
[ -z "$(git status --porcelain)" ] || die "working tree is not clean; commit or stash your changes first"
if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
    die "tag $TAG already exists"
fi
BRANCH=$(git rev-parse --abbrev-ref HEAD)
[ "$BRANCH" != "HEAD" ] || die "detached HEAD; check out a branch first"

CURRENT=$(awk -F'"' '/^version = /{ print $2; exit }' Cargo.toml)
[ -n "$CURRENT" ] || die "could not read the version from Cargo.toml"

echo "==> Releasing $TAG (Cargo.toml currently $CURRENT, branch $BRANCH)"

if [ "$CURRENT" != "$VERSION" ]; then
    # If anything below fails before we commit, put the version files back.
    trap 'git checkout -q -- Cargo.toml Cargo.lock; rm -f Cargo.toml.new' EXIT
    awk -v v="$VERSION" '
        !done && /^version = "/ { print "version = \"" v "\""; done = 1; next }
        { print }
    ' Cargo.toml > Cargo.toml.new && mv Cargo.toml.new Cargo.toml
    cargo check --quiet   # refreshes Cargo.lock with the new version
fi

echo "==> Running checks"
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked

if [ -n "$(git status --porcelain)" ]; then
    git add Cargo.toml Cargo.lock
    git commit -q -m "Release $TAG"
    trap - EXIT
    echo "==> Committed version bump"
fi
git tag -a "$TAG" -m "Release $TAG"
echo "==> Tagged $TAG"

if [ "$PUSH" = 1 ]; then
    git push origin "$BRANCH" "$TAG"
    echo "==> Pushed. Watch the build under the repository's Actions tab."
else
    echo "==> Not pushed yet. When ready, run:"
    echo "    git push origin $BRANCH $TAG"
fi
