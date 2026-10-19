cargo build --release
sudo setcap 'cap_net_admin,cap_net_raw+ep' ./target/release/nfqueue-app
RUST_LOG=info ./target/release/nfqueue-app
