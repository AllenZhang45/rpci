#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"
CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
AUTOSTART_DIR="$CONFIG_HOME/autostart"
PROFILE="${IME_PROFILE:-release}"
SKIP_BUILD="${IME_SKIP_BUILD:-0}"

if ! command -v cargo >/dev/null 2>&1 && [ -x "$HOME/.cargo/bin/cargo" ]; then
    export PATH="$HOME/.cargo/bin:$PATH"
fi

if [ "$SKIP_BUILD" != "1" ]; then
    if [ "$PROFILE" = "release" ]; then
        cargo build --release -p ime-probe --bin ime-probe --bin ime-settings --bin ime-settings-gui
    else
        cargo build -p ime-probe --bin ime-probe --bin ime-settings --bin ime-settings-gui
    fi
fi

TARGET_DIR="$ROOT/target/$PROFILE"
for bin in ime-probe ime-settings ime-settings-gui; do
    if [ ! -x "$TARGET_DIR/$bin" ]; then
        echo "missing $TARGET_DIR/$bin; build first or drop IME_SKIP_BUILD" >&2
        exit 1
    fi
done

install -Dm755 "$TARGET_DIR/ime-probe" "$BIN_DIR/ime-probe"
install -Dm755 "$TARGET_DIR/ime-settings" "$BIN_DIR/ime-settings"
install -Dm755 "$TARGET_DIR/ime-settings-gui" "$BIN_DIR/ime-settings-gui"

if [ -x "$ROOT/prototype/scripts/setup-rime.sh" ]; then
    "$ROOT/prototype/scripts/setup-rime.sh"
fi

if [ ! -f "$CONFIG_HOME/rpci/config.toml" ]; then
    "$BIN_DIR/ime-settings" reset
fi

APPS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
mkdir -p "$APPS_DIR"
cat > "$APPS_DIR/rpci-settings.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=COSMIC IME 设置
Comment=Configure the COSMIC Wubi/Pinyin input method
Exec=$BIN_DIR/ime-settings-gui
Icon=preferences-desktop
Terminal=false
Categories=Settings;
StartupNotify=true
EOF

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
echo "  $APPS_DIR/rpci-settings.desktop"
echo "  $AUTOSTART_DIR/rpci-ime.desktop"
echo "config: $(XDG_CONFIG_HOME="$CONFIG_HOME" "$BIN_DIR/ime-settings" path)"
