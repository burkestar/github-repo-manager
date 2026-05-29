# Developer Guide

How to build and run github-repo-manager from source.

## Prerequisites

- Rust (1.74+)
- `libgit2` — on macOS: `brew install libgit2`

## Build & run from source

```bash
git clone <this-repo>
cd github-repo-manager
cargo run                 # build and run in one step
cargo build --release     # release binary at ./target/release/github-repo-manager
```
