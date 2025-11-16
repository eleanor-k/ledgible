#!/bin/sh
set -e
cargo fmt --check
cargo clippy
cargo check
cargo test
cargo msrv verify
markdownlint *.md
git commit "$@"
