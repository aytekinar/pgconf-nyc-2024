#!/usr/bin/env bash

set -euo pipefail

if [[ $UID -eq 0 ]]; then
  # Install pgrx requirements
  apt update
  apt install -y clang git libclang-dev
  apt clean

  # Switch to non-root user and run the script again in the user's home directory
  full_script_path=$(realpath "$0")
  user="vscode"
  dir="/home/$user"
  cd "$dir"
  exec su "$user" "$full_script_path" -- "$@"
fi

# Load the environment/profile variables
. .profile

# Install pgrx
if [[ "$PGRX_VERSION" == "latest" ]]; then
  cargo install cargo-pgrx --locked
else
  cargo install cargo-pgrx --version "$PGRX_VERSION" --locked
fi

# Initialize pgrx with available pgenv-managed PostgreSQL versions
for version in $(.pgenv/bin/pgenv versions | sed -E 's/.*\s+(([0-9]+)\.([0-9]+))\s+.*/\1/'); do
  cargo pgrx init --pg"${version%%.*}" "${HOME}/.pgenv/pgsql-${version}/bin/pg_config"
done
