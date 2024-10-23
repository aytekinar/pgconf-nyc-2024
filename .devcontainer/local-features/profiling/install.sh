#!/usr/bin/env bash

set -euo pipefail

if [[ $UID -eq 0 ]]; then
  # Install profiling requirements
  apt update
  apt install -y bpfcc-tools gnuplot linux-perf sysstat
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

# Install cargo-criterion
if [[ "${CARGO_CRITERION_VERSION}" == "latest" ]]; then
  cargo install cargo-criterion --locked
else
  cargo install cargo-criterion --version "${CARGO_CRITERION_VERSION}" --locked
fi

# Install flamegraph
if [[ "${CARGO_FLAMEGRAPH_VERSION}" == "latest" ]]; then
  cargo install flamegraph --locked
else
  cargo install flamegraph --version "${CARGO_FLAMEGRAPH_VERSION}" --locked
fi

# Install samply
if [[ "${CARGO_SAMPLY_VERSION}" == "latest" ]]; then
  cargo install samply --locked
else
  cargo install samply --version "${CARGO_SAMPLY_VERSION}" --locked
fi
