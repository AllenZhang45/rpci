#!/usr/bin/env bash
set -euo pipefail

BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"
CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"

rm -f "$BIN_DIR/ime-probe" "$BIN_DIR/ime-settings"
rm -f "$CONFIG_HOME/autostart/rpci-ime.desktop"

if [ "${1:-}" = "--purge" ]; then
    rm -rf "$CONFIG_HOME/rpci" "$DATA_HOME/rpci"
    echo "removed binaries, autostart, config and RIME user data"
else
    echo "removed binaries and autostart"
    echo "kept config and RIME user data (use --purge to remove)"
fi
