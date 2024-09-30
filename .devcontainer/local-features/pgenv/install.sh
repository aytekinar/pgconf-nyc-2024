#!/usr/bin/env bash

set -euo pipefail

if [[ $UID -eq 0 ]]; then
  # Install pgenv and PostgreSQL requirements
  apt-get update
  apt-get install -y \
    bison \
    build-essential \
    ca-certificates \
    ccache \
    clang \
    curl \
    flex \
    git \
    libclang-dev \
    liblz4-dev \
    libreadline-dev \
    libssl-dev \
    libxml2-dev \
    libxml2-utils \
    libxslt-dev \
    pkg-config \
    uuid-dev \
    xsltproc \
    zlib1g-dev
  apt-get clean

  # Switch to non-root user and run the script again in the user's home directory
  full_script_path=$(realpath "$0")
  user="vscode"
  dir="/home/$user"
  cd "$dir"
  exec su "$user" "$full_script_path" -- "$@"
fi

# Install pgenv
if [[ ! -d ".pgenv" ]]; then
  git clone https://github.com/theory/pgenv.git .pgenv
  if [[ "$PGENV_VERSION" != "latest" ]]; then
    pushd .pgenv
    git checkout -b pgenv "$PGENV_VERSION"
    popd
  fi
fi

# Configure how PostgreSQL is built
mkdir -p .pgenv/config/
cat >>.pgenv/config/default.conf <<EOF
PGENV_MAKE_OPTIONS=(-s)

PGENV_CONFIGURE_OPTIONS=(
    --enable-debug
    --enable-cassert
    'CFLAGS=-ggdb -O2 -fno-omit-frame-pointer'
    --with-openssl
    --with-libxml
    --with-libxslt
    --with-uuid=e2fs
    --with-icu
    --with-lz4
)
EOF

# Build PostgreSQL versions
IFS=","
for version in $PG_VERSION; do
  .pgenv/bin/pgenv build "$version"
  .pgenv/bin/pgenv switch "$version"
done
