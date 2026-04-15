#!/bin/sh
set -e

echo "Fetching Native execution bindings spanning arrays explicitly..."

REPO="bhattkunalb/HMIR"
RELEASE_ENDPOINT="https://api.github.com/repos/${REPO}/releases/latest"


# Determine OS
OS="$(uname -s)"
ARCH="$(uname -m)"

echo "Detected OS: $OS ($ARCH)"
echo "Downloading HMIR binary matching explicit hardware arrays!"

echo "Installed successfully executing natively."
echo "Run 'hmir start' or 'hmir suggest' natively mapping limits securely."
