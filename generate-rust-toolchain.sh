#!/bin/sh

# shellcheck disable=SC1091
. .env

cat >rust-toolchain.toml <<EOF
[toolchain]
    channel    = "${RUST_VERSION}"
    components = ["rustfmt", "clippy"]
    targets    = ["wasm32-wasi", "wasm32-wasip1"]
EOF

echo "Generated rust-toolchain.toml with Rust version ${RUST_VERSION}"
