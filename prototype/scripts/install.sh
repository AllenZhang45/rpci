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
        cargo build --release -p ime-probe --bin ime-probe --bin ime-settings
    else
        cargo build -p ime-probe --bin ime-probe --bin ime-settings
    fi
fi

TARGET_DIR="$ROOT/target/$PROFILE"
for bin in ime-probe ime-settings; do
    if [ ! -x "$TARGET_DIR/$bin" ]; then
        echo "missing $TARGET_DIR/$bin; build first or drop IME_SKIP_BUILD" >&2
        exit 1
    fi
done

install -Dm755 "$TARGET_DIR/ime-probe" "$BIN_DIR/ime-probe"
install -Dm755 "$TARGET_DIR/ime-settings" "$BIN_DIR/ime-settings"

if [ -x "$ROOT/prototype/scripts/setup-rime.sh" ]; then
    "$ROOT/prototype/scripts/setup-rime.sh"
fi

if [ ! -f "$CONFIG_HOME/rpci/config.toml" ]; then
    "$BIN_DIR/ime-settings" reset
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

echo "installed:"
echo "  $BIN_DIR/ime-probe"
echo "  $BIN_DIR/ime-settings"
echo "  $AUTOSTART_DIR/rpci-ime.desktop"
echo "config: $(XDG_CONFIG_HOME="$CONFIG_HOME" "$BIN_DIR/ime-settings" path)"
