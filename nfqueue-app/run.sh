#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

cargo build --release
sudo setcap 'cap_net_admin,cap_net_raw+ep' ./target/release/nfqueue-app
RUST_LOG="${RUST_LOG:-info}" ./target/release/nfqueue-app
