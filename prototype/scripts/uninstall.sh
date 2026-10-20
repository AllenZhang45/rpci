#!/usr/bin/env bash
set -euo pipefail

BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"
CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
case "$CONFIG_HOME" in /*) ;; *) CONFIG_HOME="$HOME/.config" ;; esac
case "$DATA_HOME" in /*) ;; *) DATA_HOME="$HOME/.local/share" ;; esac

rm -f "$BIN_DIR/ime-probe" "$BIN_DIR/ime-settings" "$BIN_DIR/ime-settings-gui" "$BIN_DIR/ime-settings-iced" "$BIN_DIR/ime-applet"
rm -f "$CONFIG_HOME/autostart/rpci-ime.desktop"
rm -f "$DATA_HOME/applications/rpci-settings.desktop"
rm -f "$DATA_HOME/applications/rpci-settings-fallback.desktop"
rm -f "$DATA_HOME/applications/com.github.alec.RpciImeApplet.desktop"

APP_ID="com.github.alec.RpciImeApplet"
for panel in \
    "$CONFIG_HOME"/cosmic/com.system76.CosmicPanel.*/v1/plugins_center \
    "$CONFIG_HOME"/cosmic/com.system76.CosmicPanel.*/v1/plugins_wings; do
    [ -f "$panel" ] || continue
    if grep -q "$APP_ID" "$panel"; then
        sed -i "/\"$APP_ID\",/d" "$panel"
    fi
done

if [ "${1:-}" = "--purge" ]; then
    rm -rf "$CONFIG_HOME/rpci" "$DATA_HOME/rpci"
    echo "removed binaries, autostart, config and RIME user data"
else
    echo "removed binaries and autostart"
    echo "kept config and RIME user data (use --purge to remove)"
fi
