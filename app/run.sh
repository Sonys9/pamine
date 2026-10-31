#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

# The crate was renamed from `nfqueue-app` to `app`, so the binary name follows
# the package name. Keep this in sync with `[package] name` in Cargo.toml.
BIN=./target/release/app

cargo build --release
sudo setcap 'cap_net_admin,cap_net_raw+ep' "$BIN"
RUST_LOG="${RUST_LOG:-info}" "$BIN"
