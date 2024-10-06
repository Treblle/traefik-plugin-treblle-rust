#!/bin/sh

set -e # Exit immediately if a command exits with a non-zero status.

echo "Starting WasmEdge installation..."

# Check if curl is installed
if ! command -v curl >/dev/null 2>&1; then
    echo "Error: curl is not installed. Please install curl and try again."
    exit 1
fi

# Download and run the WasmEdge installation script
if curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install.sh | bash; then
    echo "WasmEdge installation completed successfully."
else
    echo "Error: WasmEdge installation failed."
    exit 1
fi

# Remind user to update their PATH
echo "Please ensure that WasmEdge is in your PATH by adding the following line to your shell configuration file (.bashrc, .zshrc, etc.):"
# shellcheck disable=SC2016
echo 'export PATH=$PATH:$HOME/.wasmedge/bin'

echo "You may need to restart your terminal or run 'source ~/.bashrc' (or equivalent) for the changes to take effect."

# Optionally, we can try to add this to the user's shell configuration automatically
# shellcheck disable=SC2162
# shellcheck disable=SC3045
read -p "Would you like to attempt to add WasmEdge to your PATH automatically? (y/n) " answer
case "$answer" in
y* | Y*)
    if [ -f "$HOME/.bashrc" ]; then
        # shellcheck disable=SC2016
        echo 'export PATH=$PATH:$HOME/.wasmedge/bin' >>"$HOME/.bashrc"
        echo "Added to .bashrc"
    elif [ -f "$HOME/.zshrc" ]; then
        # shellcheck disable=SC2016
        echo 'export PATH=$PATH:$HOME/.wasmedge/bin' >>"$HOME/.zshrc"
        echo "Added to .zshrc"
    else
        echo "Could not find .bashrc or .zshrc. Please add the PATH manually."
    fi
    ;;
*)
    echo "Skipped automatic PATH addition. Please add it manually."
    ;;
esac

echo "WasmEdge installation script completed."
