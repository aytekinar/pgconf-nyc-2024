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

# Load the environment/profile variables
. .profile

# Install cargo-deny
if [[ "${CARGO_DENY_VERSION}" == "latest" ]]; then
  cargo install cargo-deny --locked
else
  cargo install cargo-deny --version "${CARGO_DENY_VERSION}" --locked
fi

# Install cargo-udeps
if [[ "${CARGO_UDEPS_VERSION}" == "latest" ]]; then
  cargo install cargo-udeps --locked
else
  cargo install cargo-udeps --version "${CARGO_UDEPS_VERSION}" --locked
fi
