#!/bin/bash
set -euo pipefail

# engram CLI installer
# Usage: curl -fsSL https://raw.githubusercontent.com/gorillaKim/engram/main/install.sh | sh

GITHUB_OWNER="gorillaKim"
GITHUB_REPO="engram"
INSTALL_DIR="${HOME}/.local/bin"
BINARY_NAME="engram"

# OS/Arch detection
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "${OS}" in
  darwin)
    case "${ARCH}" in
      arm64) TARGET="aarch64-apple-darwin" ;;
      x86_64) TARGET="x86_64-apple-darwin" ;;
      *) echo "Error: Unsupported architecture ${ARCH} for macOS"; exit 1 ;;
    esac
    ;;
  linux)
    case "${ARCH}" in
      x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
      *) echo "Error: Unsupported architecture ${ARCH} for Linux"; exit 1 ;;
    esac
    ;;
  *)
    echo "Error: Unsupported OS ${OS}"
    echo "Please install manually using 'cargo install --git https://github.com/${GITHUB_OWNER}/${GITHUB_REPO} engram-cli'"
    exit 1
    ;;
esac

# Get version (default to latest if not specified)
if [ -z "${INSTALL_VERSION:-}" ]; then
  # Fetch latest version tag from GitHub API
  VERSION=$(curl -s "https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/')
  if [ -z "${VERSION}" ]; then
    VERSION="0.1.0" # Fallback
  fi
else
  VERSION="${INSTALL_VERSION#v}"
fi

# Idempotency check
if command -v "${BINARY_NAME}" >/dev/null 2>&1; then
  CURRENT_VER=$("${BINARY_NAME}" --version | awk '{print $2}' | sed 's/^v//')
  if [ "${CURRENT_VER}" == "${VERSION}" ]; then
    echo "engram v${VERSION} is already installed."
    exit 0
  fi
fi

echo "Installing engram v${VERSION} for ${TARGET}..."

# Download
DOWNLOAD_URL="https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}/releases/download/v${VERSION}/engram-${VERSION}-${TARGET}.tar.gz"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "${TMP_DIR}"' EXIT

echo "Downloading ${DOWNLOAD_URL}..."
if ! curl -fsSL "${DOWNLOAD_URL}" -o "${TMP_DIR}/engram.tar.gz"; then
  echo "Error: Failed to download binary. The release v${VERSION} might not be ready yet."
  exit 1
fi

# Extract
tar -C "${TMP_DIR}" -xzf "${TMP_DIR}/engram.tar.gz"

# Install
mkdir -p "${INSTALL_DIR}"
mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

echo "Successfully installed engram v${VERSION} to ${INSTALL_DIR}/${BINARY_NAME}"

# PATH check
if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
  echo ""
  echo "Warning: ${INSTALL_DIR} is not in your PATH."
  echo "Please add the following line to your ~/.zshrc or ~/.bashrc:"
  echo "  export PATH=\"\$PATH:${INSTALL_DIR}\""
fi

"${INSTALL_DIR}/${BINARY_NAME}" --version
