#!/bin/sh

set -e # Exit immediately if a command exits with a non-zero status.

# Check for required commands
check_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "Error: $1 is not installed. Please install it and try again."
        exit 1
    fi
}

check_command curl
check_command tar

# Set WASI version
export WASI_VERSION=22
export WASI_VERSION_FULL=${WASI_VERSION}.0

# Download wasi-sdk
echo "Downloading wasi-sdk version ${WASI_VERSION_FULL}..."
if ! curl -L -o wasi-sdk-${WASI_VERSION_FULL}-macos.tar.gz https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-${WASI_VERSION}/wasi-sdk-${WASI_VERSION_FULL}-macos.tar.gz; then
    echo "Error: Failed to download wasi-sdk. Please check your internet connection and try again."
    exit 1
fi

# Extract the archive
echo "Extracting wasi-sdk..."
if ! tar xvf wasi-sdk-${WASI_VERSION_FULL}-macos.tar.gz; then
    echo "Error: Failed to extract wasi-sdk archive."
    exit 1
fi

# Set up environment variables
WASI_SDK_PATH=$(pwd)/wasi-sdk-${WASI_VERSION_FULL}

# Determine the correct profile file
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

# Add environment variables to the profile file
echo "Adding environment variables to $PROFILE_FILE"
echo "export WASI_SDK_PATH=${WASI_SDK_PATH}" >>"$PROFILE_FILE"
# shellcheck disable=SC2016
echo 'export CC="${WASI_SDK_PATH}/bin/clang --sysroot=${WASI_SDK_PATH}/share/wasi-sysroot"' >>"$PROFILE_FILE"

# Clean up the downloaded archive
rm wasi-sdk-${WASI_VERSION_FULL}-macos.tar.gz

echo "wasi-sdk installation complete!"
echo "Please run 'source $PROFILE_FILE' or restart your terminal to apply the changes."
