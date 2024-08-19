#!/bin/sh

WASM_MODULE="plugins-local/src/github.com/momo-gg/rust-http-wasm/plugin.wasm"

# ANSI color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

check_export() {
    function_name=$1
    if wasm-objdump -x "$WASM_MODULE" | grep -q "$function_name"; then
        printf "${GREEN}[OK] The WASM module exports the %s function.${NC}\n" "$function_name"
    else
        printf "${RED}[ERROR] The WASM module does not export the %s function.${NC}\n" "$function_name"
        exit 1
    fi
}

# Check if the WASM module exports functions required by Traefik
check_export 'handle_request'
check_export 'handle_response'
