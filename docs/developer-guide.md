# Developer Guide

How to build and run github-repo-manager from source.

## Prerequisites

- Rust (1.74+)
- A C compiler and `perl` — needed to build the vendored `libgit2` and OpenSSL
  (`git2` is configured with the `vendored-libgit2` and `vendored-openssl` features, so
  no system `libgit2`/OpenSSL is required). On macOS the Xcode Command Line Tools
  (`xcode-select --install`) provide these.

## Build & run from source

```bash
git clone https://github.com/burkestar/github-repo-manager
cd github-repo-manager
cargo run                 # build and run in one step
cargo build --release     # release binary at ./target/release/github-repo-manager
```

## Continuous integration

`.github/workflows/ci.yml` runs on every push to `main` and on pull requests:
`cargo fmt --check` + `cargo clippy -D warnings` (Linux), and `cargo build` + `cargo test`
on Linux, macOS, and Windows. Run the same checks locally before pushing:

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Releasing

Releases are automated with [dist](https://github.com/axodotdev/cargo-dist) (`dist-workspace.toml`
holds the config; `.github/workflows/release.yml` is generated — don't hand-edit it, run
`dist generate` after config changes). Pushing a version tag builds binaries for macOS, Linux, and
Windows, creates a GitHub Release with those artifacts + installers, and publishes the crate to
crates.io.

**One-time setup:** create a [crates.io API token](https://crates.io/settings/tokens) and add it as
the GitHub Actions repository secret **`CARGO_REGISTRY_TOKEN`** (Settings → Secrets and variables →
Actions). `GITHUB_TOKEN` is provided automatically.

**Cut a release:**

```bash
# 1. Bump the version in Cargo.toml (e.g. 0.1.0 -> 0.1.1), commit it.
# 2. Verify locally before tagging:
cargo publish --dry-run         # checks crates.io packaging
dist plan                       # lists the artifacts that will be built

# 3. Tag and push — this triggers the Release workflow:
git tag v0.1.1
git push origin v0.1.1
```

Prerelease tags (e.g. `v0.1.1-rc.1`) build and publish the GitHub Release but **skip** the crates.io
publish, so they're a safe way to exercise the full pipeline. `cargo publish` is irreversible — a
version can never be re-published — so always dry-run first.

