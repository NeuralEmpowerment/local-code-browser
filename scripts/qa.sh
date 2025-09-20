#!/usr/bin/env bash
set -euo pipefail

echo "==> fmt"
cargo fmt --all -- --check

echo "==> clippy"
cargo clippy --workspace --all-targets -- -D warnings

echo "==> check"
cargo check --workspace --all-targets

echo "==> test"
cargo test --workspace --all-targets -- --nocapture

echo "==> build"
cargo build --workspace --all-targets

echo "QA passed"

