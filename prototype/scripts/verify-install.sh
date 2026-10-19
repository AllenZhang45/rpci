#!/usr/bin/env bash
set -euo pipefail

BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"
CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
case "$CONFIG_HOME" in /*) ;; *) CONFIG_HOME="$HOME/.config" ;; esac
case "$DATA_HOME" in /*) ;; *) DATA_HOME="$HOME/.local/share" ;; esac
DESKTOP="$CONFIG_HOME/autostart/rpci-ime.desktop"
UNIT='app-rpci\x2dime@autostart.service'
WANTS="/run/user/$(id -u)/systemd/generator.late/xdg-desktop-autostart.target.wants/$UNIT"
fail=0

check() {
    if eval "$2"; then
        echo "ok   $1"
    else
        echo "FAIL $1"
        fail=1
    fi
}

check "release binary installed" "[ -x '$BIN_DIR/ime-probe' ]"
check "settings CLI installed" "[ -x '$BIN_DIR/ime-settings' ]"
check "iced settings installed" "[ -x '$BIN_DIR/ime-settings-iced' ]"
check "settings launcher installed" "[ -f '$DATA_HOME/applications/rpci-settings.desktop' ]"
check "fallback settings launcher installed" "[ -f '$DATA_HOME/applications/rpci-settings-fallback.desktop' ]"
check "config exists" "[ -f '$CONFIG_HOME/rpci/config.toml' ]"
check "RIME data exists" "[ -d '$DATA_HOME/rpci/rime' ]"
check "autostart desktop exists" "[ -f '$DESKTOP' ]"

EXEC="$(sed -n 's/^Exec=//p' "$DESKTOP" 2>/dev/null | head -1 || true)"
check "autostart Exec exists: ${EXEC:-<empty>}" "[ -n '$EXEC' ] && [ -x '$EXEC' ]"
check "OnlyShowIn contains COSMIC" "grep -q '^OnlyShowIn=.*COSMIC' '$DESKTOP'"

if command -v desktop-file-validate >/dev/null 2>&1; then
    if desktop-file-validate "$DESKTOP" >/tmp/rpci-desktop-validate.out 2>&1; then
        echo "ok   desktop-file-validate"
    else
        # desktop-file-utils has no COSMIC in its registry; the systemd
        # generator still matches XDG_CURRENT_DESKTOP=COSMIC.
        echo "warn desktop-file-validate: $(cat /tmp/rpci-desktop-validate.out)"
        echo "     (expected: desktop-file-utils does not register COSMIC)"
    fi
fi

if command -v systemctl >/dev/null 2>&1; then
    systemctl --user daemon-reload 2>/dev/null || true
    check "systemd autostart unit generated" "systemctl --user list-unit-files | grep -qF '$UNIT'"
    check "autostart target is active" "systemctl --user is-active xdg-desktop-autostart.target >/dev/null"
    check "autostart unit linked into target" "[ -L '$WANTS' ]"
    echo "unit: $(systemctl --user is-active "$UNIT" 2>/dev/null || true)"
else
    echo "warn systemctl not available; autostart checked as desktop file only"
fi

if [ "${1:-}" = "--start" ]; then
    systemctl --user start "$UNIT"
    sleep 1
    check "autostart unit is active" "systemctl --user is-active '$UNIT' >/dev/null"
    pgrep -a -x ime-probe || true
fi

if [ "$fail" -ne 0 ]; then
    exit 1
fi
echo "install verification passed"
