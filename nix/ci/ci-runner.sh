#!/bin/sh
set -e

echo ">>> Running format check..."
cargo fmt --check

echo ">>> Running clippy..."
cargo clippy -- -D warnings

echo ">>> Running tests..."
cargo test

echo "ALL OK"
