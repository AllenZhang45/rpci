#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"
CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
case "$CONFIG_HOME" in /*) ;; *) CONFIG_HOME="$HOME/.config" ;; esac
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
case "$DATA_HOME" in /*) ;; *) DATA_HOME="$HOME/.local/share" ;; esac
AUTOSTART_DIR="$CONFIG_HOME/autostart"
PROFILE="${IME_PROFILE:-release}"
SKIP_BUILD="${IME_SKIP_BUILD:-0}"

if ! command -v cargo >/dev/null 2>&1 && [ -x "$HOME/.cargo/bin/cargo" ]; then
    export PATH="$HOME/.cargo/bin:$PATH"
fi

if [ "$SKIP_BUILD" != "1" ]; then
    if [ "$PROFILE" = "release" ]; then
        cargo build --release -p ime-probe --bin ime-probe --bin ime-settings --bin ime-settings-gui
        cargo build --release -p rpci-settings-iced --bin rpci-settings-iced --bin rpci-applet
    else
        cargo build -p ime-probe --bin ime-probe --bin ime-settings --bin ime-settings-gui
        cargo build -p rpci-settings-iced --bin rpci-settings-iced --bin rpci-applet
    fi
fi

TARGET_DIR="$ROOT/target/$PROFILE"
for bin in ime-probe ime-settings ime-settings-gui rpci-settings-iced rpci-applet; do
    if [ ! -x "$TARGET_DIR/$bin" ]; then
        echo "missing $TARGET_DIR/$bin; build first or drop IME_SKIP_BUILD" >&2
        exit 1
    fi
done

install -Dm755 "$TARGET_DIR/ime-probe" "$BIN_DIR/ime-probe"
install -Dm755 "$TARGET_DIR/ime-settings" "$BIN_DIR/ime-settings"
install -Dm755 "$TARGET_DIR/ime-settings-gui" "$BIN_DIR/ime-settings-gui"
install -Dm755 "$TARGET_DIR/rpci-settings-iced" "$BIN_DIR/ime-settings-iced"
install -Dm755 "$TARGET_DIR/rpci-applet" "$BIN_DIR/ime-applet"

if [ -x "$ROOT/prototype/scripts/setup-rime.sh" ]; then
    "$ROOT/prototype/scripts/setup-rime.sh"
fi

if [ ! -f "$CONFIG_HOME/rpci/config.toml" ]; then
    "$BIN_DIR/ime-settings" reset
fi

APPS_DIR="$DATA_HOME/applications"
mkdir -p "$APPS_DIR"
cat > "$APPS_DIR/rpci-settings.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=COSMIC IME 设置
Comment=Configure the COSMIC Wubi/Pinyin input method
Exec=$BIN_DIR/ime-settings-iced
Icon=preferences-desktop
Terminal=false
Categories=Settings;
StartupNotify=true
EOF

cat > "$APPS_DIR/rpci-settings-fallback.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=COSMIC IME 设置（兼容模式）
Comment=Keyboard-driven fallback settings window
Exec=$BIN_DIR/ime-settings-gui
Icon=preferences-desktop
Terminal=false
Categories=Settings;
EOF

APP_ID="com.github.alec.RpciImeApplet"
cat > "$APPS_DIR/$APP_ID.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=COSMIC IME
Comment=Wubi/Pinyin input method status
Exec=$BIN_DIR/ime-applet
Terminal=false
NoDisplay=true
X-CosmicApplet=true
X-HostWaylandDisplay=true
X-OverflowPriority=10
EOF

# Place the applet in the top Panel's right wing (the taskbar status area).
# Fall back to the Panel center or Dock center when the wings config is absent.
# Remove duplicates from every known panel config first.
for panel_file in \
    "$CONFIG_HOME"/cosmic/com.system76.CosmicPanel.*/v1/plugins_center \
    "$CONFIG_HOME"/cosmic/com.system76.CosmicPanel.*/v1/plugins_wings; do
    [ -f "$panel_file" ] || continue
    sed -i "/$APP_ID/d" "$panel_file"
done

if [ -n "${RPCI_PANEL_CONFIG:-}" ]; then
    PANEL_CONFIG="$RPCI_PANEL_CONFIG"
    if [ -f "$PANEL_CONFIG" ]; then
        cp "$PANEL_CONFIG" "$PANEL_CONFIG.bak.rpci"
        sed -i "/^])/i\    \"$APP_ID\"," "$PANEL_CONFIG"
    fi
else
    WINGS="$CONFIG_HOME/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings"
    if [ -f "$WINGS" ] && grep -q '"com.system76.CosmicAppletInputSources"' "$WINGS"; then
        cp "$WINGS" "$WINGS.bak.rpci"
        sed -i "/\"com.system76.CosmicAppletInputSources\"/i\    \"$APP_ID\"," "$WINGS"
    else
        PANEL_CONFIG="$CONFIG_HOME/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_center"
        if [ ! -f "$PANEL_CONFIG" ]; then
            PANEL_CONFIG="$CONFIG_HOME/cosmic/com.system76.CosmicPanel.Dock/v1/plugins_center"
        fi
        if [ -f "$PANEL_CONFIG" ]; then
            cp "$PANEL_CONFIG" "$PANEL_CONFIG.bak.rpci"
            sed -i "/^])/i\    \"$APP_ID\"," "$PANEL_CONFIG"
        fi
    fi
fi

mkdir -p "$AUTOSTART_DIR"
cat > "$AUTOSTART_DIR/rpci-ime.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=COSMIC IME
Comment=Wubi/Pinyin input method for COSMIC
Exec=$BIN_DIR/ime-probe
Terminal=false
NoDisplay=true
X-GNOME-Autostart-enabled=true
OnlyShowIn=COSMIC;
EOF

# XDG autostart is consumed by systemd-xdg-autostart-generator on COSMIC.
# desktop-file-utils does not know the COSMIC name yet; the generator uses
# OnlyShowIn=COSMIC and passes it as the ExecCondition.
if command -v systemctl >/dev/null 2>&1; then
    systemctl --user daemon-reload 2>/dev/null || true
fi

echo "installed:"
echo "  $BIN_DIR/ime-probe"
echo "  $BIN_DIR/ime-settings"
echo "  $BIN_DIR/ime-settings-gui"
echo "  $BIN_DIR/ime-settings-iced"
echo "  $BIN_DIR/ime-applet"
echo "  $APPS_DIR/rpci-settings.desktop"
echo "  $APPS_DIR/rpci-settings-fallback.desktop"
echo "  $APPS_DIR/$APP_ID.desktop"
echo "  $AUTOSTART_DIR/rpci-ime.desktop"
echo "config: $(XDG_CONFIG_HOME="$CONFIG_HOME" "$BIN_DIR/ime-settings" path)"
