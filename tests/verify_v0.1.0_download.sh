#!/bin/bash
set -euo pipefail

# Test script for Task 54: Verify v0.1.0 download and version
# This script simulates a user downloading the binary and checking the version.

VERSION="0.1.0"
OWNER="gorillaKim"
REPO="engram"
TARGET="aarch64-apple-darwin" # Testing on current machine architecture if possible, or just checking URL

# We will test the ARM64 one as it's already 302
DOWNLOAD_URL="https://github.com/${OWNER}/${REPO}/releases/download/v${VERSION}/engram-${VERSION}-${TARGET}.tar.gz"

echo "Testing download from: ${DOWNLOAD_URL}"

TMP_DIR=$(mktemp -d)
trap 'rm -rf "${TMP_DIR}"' EXIT

if curl -fsSL "${DOWNLOAD_URL}" -o "${TMP_DIR}/engram.tar.gz"; then
  echo "Download successful."
  tar -C "${TMP_DIR}" -xzf "${TMP_DIR}/engram.tar.gz"
  
  INSTALLED_VERSION=$("${TMP_DIR}/engram" --version)
  echo "Installed version: ${INSTALLED_VERSION}"
  
  if [[ "${INSTALLED_VERSION}" == *"v${VERSION}"* || "${INSTALLED_VERSION}" == *"${VERSION}"* ]]; then
    echo "Version check passed."
  else
    echo "Version check failed. Expected ${VERSION}, got ${INSTALLED_VERSION}"
    exit 1
  fi
else
  echo "Download failed. Release might not be fully ready."
  exit 1
fi
