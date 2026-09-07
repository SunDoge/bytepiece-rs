#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Keep the CLI version identical to bytepiece-wasm/Cargo.toml.
expected=0.2.128
if [[ "$(wasm-bindgen --version)" != "wasm-bindgen $expected" ]]; then
    echo "Install the matching CLI: cargo install wasm-bindgen-cli --version $expected --locked" >&2
    exit 1
fi
cargo build -p bytepiece-wasm --target wasm32-unknown-unknown --release --locked
for target in web nodejs; do
    wasm-bindgen target/wasm32-unknown-unknown/release/bytepiece_wasm.wasm \
        --target "$target" --out-dir "bytepiece-wasm/pkg-$target" --out-name bytepiece
done
