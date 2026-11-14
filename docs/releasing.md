# Releasing

Publishing a new version is two steps: run the release script, then wait for GitHub Actions.

```sh
scripts/release.sh 1.2.0 --push
```

That's it. A few minutes later the release page has binaries for every platform, `SHA256SUMS` and `install.sh`, and anyone running the [installer](installation.md) gets the new version.

## What the script does

`scripts/release.sh <version> [--push]`:

1. Refuses to run if the working tree has uncommitted changes, the tag already exists, or HEAD is detached.
2. Sets `version` in `Cargo.toml` and refreshes `Cargo.lock`.
3. Runs `cargo fmt --check`, `cargo clippy -D warnings` and `cargo test`. Nothing is tagged if any of them fail.
4. Commits `Release vX.Y.Z` and creates the annotated tag `vX.Y.Z`.
5. With `--push`, pushes the branch and tag to `origin`. Without it, prints the command so you can push when ready.

If a check fails, the script restores `Cargo.toml` and `Cargo.lock` and stops without committing or tagging. Fix the problem, commit it, and run the script again.

## What GitHub Actions does

`.github/workflows/release.yml` runs when a tag matching `v*` is pushed:

1. **Build** (in parallel) for `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `aarch64-apple-darwin` and `x86_64-apple-darwin`. Each job first checks that the tag equals the `Cargo.toml` version, so a mismatched tag fails early.
2. **Package** each binary as `vin-<target>.tar.gz` (binary + `LICENSE` + `README.md`).
3. **Publish**: creates the release with auto-generated notes, attaches the archives, a `SHA256SUMS` file and `install.sh`. If a release for that tag already exists (e.g. you drafted one in the web UI), the files are attached to it instead. Tags containing a `-` (like `v1.3.0-rc1`) are marked as pre-releases.

`.github/workflows/ci.yml` runs formatting, clippy, tests and shellcheck on every push and pull request, so problems show up before release time.

## Versioning

Use [semantic versioning](https://semver.org): bump the patch for fixes, the minor for new features, the major for breaking changes. `vin --version` reports the `Cargo.toml` version, which always equals the tag.

## Adding or changing a template

Templates are compiled into the binary, so a template change only reaches users through a new release:

1. Edit `src/directories_structure/*.yaml`.
2. `cargo test` (validates every built-in template).
3. Commit, then `scripts/release.sh <next version> --push`.

## Manual release (without the script)

```sh
# edit version in Cargo.toml, then:
cargo check && cargo test
git commit -am "Release v1.2.0"
git tag -a v1.2.0 -m "Release v1.2.0"
git push origin main v1.2.0
```

## Adding a platform

Add a line to the `matrix` in `release.yml` (and to the target detection in `install.sh`, in the `detect_target` function), then document it in [installation.md](installation.md).
