#!/usr/bin/env sh
set -eu

cargo fmt --all --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
python3 scripts/check-static-limits.py
