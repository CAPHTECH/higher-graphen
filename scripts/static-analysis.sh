#!/usr/bin/env sh
set -eu

cargo fmt --all --check
cargo metadata --locked --format-version 1 --no-deps >/dev/null
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
python3 scripts/check-static-limits.py
python3 scripts/validate-cli-report-contract.py
python3 scripts/validate-json-contracts.py
python3 integrations/cli-skill-bundle/check-bundle.py
