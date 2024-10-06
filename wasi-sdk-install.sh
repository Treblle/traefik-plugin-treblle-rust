#!/bin/sh

set -e # Exit immediately if a command exits with a non-zero status.

check_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "Error: $1 is not installed. Please install it and try again."
        exit 1
    fi
}

check_command curl
check_command tar

export WASI_VERSION=24
export WASI_VERSION_FULL=${WASI_VERSION}.0

if [ "$(uname)" = "Darwin" ]; then
    OS="macos"
elif [ "$(uname)" = "Linux" ]; then
    OS="linux"
else
    echo "Unsupported operating system. This script supports macOS and Linux."
    exit 1
fi

echo "Downloading wasi-sdk version ${WASI_VERSION_FULL} for ${OS}..."
DOWNLOAD_URL="https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-${WASI_VERSION}/wasi-sdk-${WASI_VERSION_FULL}-${OS}.tar.gz"
curl -L -o wasi-sdk-${WASI_VERSION_FULL}-${OS}.tar.gz "$DOWNLOAD_URL"

echo "Extracting wasi-sdk..."
tar xf wasi-sdk-${WASI_VERSION_FULL}-${OS}.tar.gz
rm wasi-sdk-${WASI_VERSION_FULL}-${OS}.tar.gz

WASI_SDK_PATH=$(pwd)/wasi-sdk-${WASI_VERSION_FULL}

if [ -n "$ZSH_VERSION" ]; then
    PROFILE_FILE="$HOME/.zshrc"
elif [ -n "$BASH_VERSION" ]; then
    PROFILE_FILE="$HOME/.bash_profile"
else
    echo "Unsupported shell. Please manually add the following lines to your shell profile:"
    echo "export WASI_SDK_PATH=${WASI_SDK_PATH}"
    # shellcheck disable=SC2016
    echo 'export CC="${WASI_SDK_PATH}/bin/clang --sysroot=${WASI_SDK_PATH}/share/wasi-sysroot"'
    exit 1
fi

echo "Adding environment variables to $PROFILE_FILE"
echo "export WASI_SDK_PATH=${WASI_SDK_PATH}" >>"$PROFILE_FILE"
# shellcheck disable=SC2016
echo 'export CC="${WASI_SDK_PATH}/bin/clang --sysroot=${WASI_SDK_PATH}/share/wasi-sysroot"' >>"$PROFILE_FILE"

echo "wasi-sdk installation complete!"
echo "Please run 'source $PROFILE_FILE' or restart your terminal to apply the changes."

rustup target add wasm32-wasi

echo "Setup complete. You may need to restart your terminal or run 'source $PROFILE_FILE' to apply the changes."
