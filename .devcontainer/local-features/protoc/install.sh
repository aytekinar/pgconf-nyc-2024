#!/usr/bin/env bash

set -euo pipefail

# Install protoc requirements
apt update
apt install -y unzip

# Download and extract the protoc archive
curl -fLsSo /tmp/protoc.zip https://github.com/protocolbuffers/protobuf/releases/download/${PROTOC_VERSION}/protoc-${PROTOC_VERSION##v}-linux-x86_64.zip
echo "${PROTOC_SHA256}  /tmp/protoc.zip" | sha256sum -c
unzip -q /tmp/protoc.zip -x readme.txt -d /usr/local

# Clean up build dependencies
rm -f /tmp/protoc.zip
apt purge -y unzip
apt autoremove -y
apt clean
