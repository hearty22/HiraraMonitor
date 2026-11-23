#!/bin/sh
# Install the latest HiraraMonitor release binary into ~/.local/bin.
set -eu

REPO="hearty22/HiraraMonitor"
ASSET_SUFFIX="x86_64-linux"
PREFIX="${PREFIX:-$HOME/.local/bin}"
BIN="hirara-monitor"

need() {
    command -v "$1" >/dev/null 2>&1 || {
        echo "error: $1 is required" >&2
        exit 1
    }
}

need curl
need uname

arch="$(uname -m)"
case "$arch" in
    x86_64|amd64) ;;
    *)
        echo "error: unsupported architecture: $arch (only x86_64 builds are published)" >&2
        exit 1
        ;;
esac

echo "Fetching latest release metadata..."
url="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
    | grep -o '"browser_download_url": *"[^"]*"' \
    | cut -d'"' -f4 \
    | grep -- "$ASSET_SUFFIX" \
    | head -n1)"

if [ -z "$url" ]; then
    echo "error: no release asset found for $ASSET_SUFFIX" >&2
    exit 1
fi

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT HUP INT TERM

echo "Downloading $(basename "$url")..."
curl -fsSL "$url" -o "$tmp"
chmod +x "$tmp"

mkdir -p "$PREFIX"
mv "$tmp" "$PREFIX/$BIN"
trap - EXIT HUP INT TERM

echo "Installed to $PREFIX/$BIN"

case ":$PATH:" in
    *":$PREFIX:"*) ;;
    *) echo "note: add $PREFIX to your PATH to run hirara-monitor directly" ;;
esac

if command -v pacman >/dev/null 2>&1; then
    echo "note: for the system tray install gtk3 and libappindicator-gtk3"
fi
