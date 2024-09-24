#!/bin/sh
set -e

# Define the plugin directory
PLUGIN_DIR="../plugins-local/src/github.com/momo-gg/rust-http-wasm"

# Ensure the target is installed
rustup target add wasm32-wasip1

# Build the WASM plugin
cargo build --target wasm32-wasip1 --release --features wasm

# Remove the existing plugin directory if it exists
if [ -d "$PLUGIN_DIR" ]; then
    echo "Removing existing plugin directory..."
    rm -rf "$PLUGIN_DIR"
fi

# Create the plugin directory structure
echo "Creating plugin directory..."
mkdir -p "$PLUGIN_DIR"

# Copy the compiled WASM file
echo "Copying WASM plugin..."
cp target/wasm32-wasip1/release/rust_http_wasm.wasm "$PLUGIN_DIR/plugin.wasm"

# Copy the .traefik.yml file
echo "Copying .traefik.yml..."
cp .traefik.yml "$PLUGIN_DIR/.traefik.yml"

echo "Build complete. Plugin files have been copied to $PLUGIN_DIR"
