set -e
cargo fmt --check
cargo clippy
cargo check
cargo test
cargo msrv verify
git commit
