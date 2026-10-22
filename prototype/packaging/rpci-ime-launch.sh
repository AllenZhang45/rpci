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

# One daily installation path: if the system package is installed, retire a
# user-level autostart left over from a source install. Keep its data and
# binaries; only the autostart is disabled.
SYSTEM_PACKAGE="${RPCI_IME_SYSTEM_PACKAGE:-}"
if [ -z "$SYSTEM_PACKAGE" ]; then
    if command -v dpkg-query >/dev/null 2>&1 &&
        dpkg-query -W -f='${Status}' rpci-ime 2>/dev/null | grep -q 'install ok installed'; then
        SYSTEM_PACKAGE=1
    else
        SYSTEM_PACKAGE=0
    fi
fi
if [ "$SYSTEM_PACKAGE" = 1 ]; then
    USER_AUTOSTART="$CONFIG_HOME/autostart/rpci-ime.desktop"
    if [ -f "$USER_AUTOSTART" ]; then
        mv -f "$USER_AUTOSTART" "$USER_AUTOSTART.disabled-by-system-package"
        printf '%s\n' "rpci-ime: disabled user-level autostart; using the system package" >&2
    fi
fi

WINGS="$CONFIG_HOME/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings"
CENTER="$CONFIG_HOME/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_center"
DOCK="$CONFIG_HOME/cosmic/com.system76.CosmicPanel.Dock/v1/plugins_center"

# COSMIC major upgrades can reset the panel config. Check only active files,
# never .bak files, and keep the applet in the top Panel's right wing.
if [ -f "$WINGS" ] && grep -q '"com.system76.CosmicAppletInputSources"' "$WINGS"; then
    if ! grep -q "$APP_ID" "$WINGS"; then
        for f in "$CENTER" "$DOCK" "$CONFIG_HOME/cosmic/com.system76.CosmicPanel.Dock/v1/plugins_wings"; do
            [ -f "$f" ] && sed -i "/$APP_ID/d" "$f" 2>/dev/null || true
        done
        cp "$WINGS" "$WINGS.bak.rpci" 2>/dev/null || true
        sed -i "/\"com.system76.CosmicAppletInputSources\"/i\    \"$APP_ID\"," "$WINGS" 2>/dev/null || true
    fi
elif [ -f "$CENTER" ]; then
    if ! grep -q "$APP_ID" "$CENTER"; then
        cp "$CENTER" "$CENTER.bak.rpci" 2>/dev/null || true
        sed -i "/^])/i\    \"$APP_ID\"," "$CENTER" 2>/dev/null || true
    fi
elif [ -f "$DOCK" ]; then
    if ! grep -q "$APP_ID" "$DOCK"; then
        cp "$DOCK" "$DOCK.bak.rpci" 2>/dev/null || true
        sed -i "/^])/i\    \"$APP_ID\"," "$DOCK" 2>/dev/null || true
    fi
fi

exec "$BIN_DIR/ime-probe"
