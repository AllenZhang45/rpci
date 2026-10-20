#!/bin/sh
set -eu

BIN_DIR="${RPCI_IME_BIN_DIR:-/usr/bin}"
SETUP="${RPCI_IME_SETUP:-$BIN_DIR/rpci-ime-setup}"
SCHEMA_DIR="${RPCI_IME_SCHEMA_DIR:-/usr/share/rpci/rime}"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
case "$DATA_HOME" in /*) ;; *) DATA_HOME="$HOME/.local/share" ;; esac
RIME_DIR="${IME_RIME_USER_DIR:-$DATA_HOME/rpci/rime}"

if [ ! -f "$RIME_DIR/default.custom.yaml" ]; then
    IME_RIME_SCHEMA_SRC="$SCHEMA_DIR" "$SETUP" >/dev/null 2>&1 || true
fi

CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
case "$CONFIG_HOME" in /*) ;; *) CONFIG_HOME="$HOME/.config" ;; esac
APP_ID="com.github.alec.RpciImeApplet"
if ! grep -Rq "$APP_ID" "$CONFIG_HOME"/cosmic/com.system76.CosmicPanel.*/v1/plugins_* 2>/dev/null; then
    WINGS="$CONFIG_HOME/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings"
    if [ -f "$WINGS" ] && grep -q '"com.system76.CosmicAppletInputSources"' "$WINGS"; then
        cp "$WINGS" "$WINGS.bak.rpci" 2>/dev/null || true
        sed -i "/\"com.system76.CosmicAppletInputSources\"/i\    \"$APP_ID\"," "$WINGS" 2>/dev/null || true
    else
        PANEL="$CONFIG_HOME/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_center"
        if [ ! -f "$PANEL" ]; then
            PANEL="$CONFIG_HOME/cosmic/com.system76.CosmicPanel.Dock/v1/plugins_center"
        fi
        if [ -f "$PANEL" ]; then
            cp "$PANEL" "$PANEL.bak.rpci" 2>/dev/null || true
            sed -i "/^])/i\    \"$APP_ID\"," "$PANEL" 2>/dev/null || true
        fi
    fi
fi

exec "$BIN_DIR/ime-probe"
