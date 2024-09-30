#!/usr/bin/env bash

set -euo pipefail

if [[ $UID -eq 0 ]]; then
  # Switch to non-root user and run the script again in the user's home directory
  full_script_path=$(realpath "$0")
  user="vscode"
  dir="/home/$user"
  cd "$dir"
  exec su "$user" "$full_script_path" -- "$@"
fi

# Install the Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --component "$RUST_COMPONENTS"

# Load the environment/profile variables
. .profile

# Install the nightly toolchain
rustup toolchain install nightly --component "$RUST_COMPONENTS"
