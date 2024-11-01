#!/bin/sh

set -e # Exit immediately if a command exits with a non-zero status.

if [ -d "wabt" ]; then
    echo "Removing existing wabt directory..."
    rm -rf wabt
fi

git clone --recursive https://github.com/WebAssembly/wabt

cd wabt || exit

git submodule update --init

# CMake and Ninja are required to build wabt, if on macOS, install them with Homebrew
# brew install cmake
# brew install ninja

# Optionally install pthread-stubs for threading support
# brew install pthread-stubs

make clang-debug
make gcc-i686-release
make clang-debug-lsan

chmod +x bin/

sudo cp bin/* /usr/local/bin/

# Verify that wasm-objdump is available
wasm-objdump --help
