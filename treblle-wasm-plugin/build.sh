#!/bin/sh
set -e

# Default target
TARGET=${TARGET:-wasm32-wasip1}

# Define the plugin directory
PLUGIN_DIR="../plugins-local/src/github.com/momo-gg/treblle-wasm-plugin"
# Ensure the target is installed
rustup target add "$TARGET"

# Build the WASM plugin
export CC="/opt/homebrew/opt/llvm/bin/clang"
export TARGET_CC="/opt/homebrew/opt/llvm/bin/clang"
cargo build --target "$TARGET" --release --features wasm

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
cp target/"$TARGET"/release/treblle_wasm_plugin.wasm "$PLUGIN_DIR/plugin.wasm"

# Copy the .traefik.yml file
echo "Copying .traefik.yml..."
cp .traefik.yml "$PLUGIN_DIR/.traefik.yml"

echo "Build complete. Plugin files have been copied to $PLUGIN_DIR"
