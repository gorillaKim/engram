#!/bin/bash
# PostInstall hook script for engram-orchestrator
# This script ensures that engram CLI is installed on the user's machine.

set -e

echo "Checking for engram CLI..."
if command -v engram >/dev/null 2>&1; then
    VERSION=$(engram --version | awk '{print $2}')
    echo "engram CLI already installed (version: $VERSION)"
else
    echo "engram CLI not found. Installing via install.sh..."
    # Call the install.sh from the main repository
    curl -fsSL https://raw.githubusercontent.com/gorillaKim/engram/main/install.sh | sh
fi

if command -v engram >/dev/null 2>&1; then
    echo "engram CLI successfully verified."
else
    echo "Warning: engram CLI installation failed. Subagents will fallback to MCP tools only."
fi
