#!/bin/sh

WASM_MODULE="plugins-local/src/github.com/momo-gg/treblle-wasm-plugin/plugin.wasm"

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

check_file_size() {
    if [ -f "$WASM_MODULE" ]; then
        file_size=$(stat -f%z "$WASM_MODULE")
        if [ "$file_size" -ge 1048576 ]; then
            file_size_mb=$(echo "scale=2; $file_size / 1048576" | bc)
            printf "${GREEN}[INFO] The WASM module size is %s MB.${NC}\n" "$file_size_mb"
        else
            file_size_kb=$(echo "scale=2; $file_size / 1024" | bc)
            printf "${GREEN}[INFO] The WASM module size is %s KB.${NC}\n" "$file_size_kb"
        fi
    else
        printf "${RED}[ERROR] The WASM module file does not exist.${NC}\n"
        exit 1
    fi
}

# Check if the WASM module exports functions required by Traefik
check_export 'handle_request'
check_export 'handle_response'

# Check the size of the generated WASM file
check_file_size
