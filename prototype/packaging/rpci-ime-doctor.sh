#!/bin/sh
# Read-only health check for the daily RPCI IME installation.
set -eu

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
    cat <<'EOF'
rpci-ime-doctor - read-only RPCI IME installation check

Checks:
  - system package and version
  - user-level autostart
  - IME process count
  - top Panel applet registration
  - RIME data directory
  - APT source configuration
  - runtime state

Exit codes: 0 = all checks passed, 1 = warnings or failures.
EOF
    exit 0
fi

CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
case "$CONFIG_HOME" in /*) ;; *) CONFIG_HOME="$HOME/.config" ;; esac
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
case "$DATA_HOME" in /*) ;; *) DATA_HOME="$HOME/.local/share" ;; esac
RUNTIME="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
RIME_DIR="$DATA_HOME/rpci/rime"
APP_ID="com.github.alec.RpciImeApplet"
PANEL_WINGS="$CONFIG_HOME/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings"
DOCK_CENTER="$CONFIG_HOME/cosmic/com.system76.CosmicPanel.Dock/v1/plugins_center"

issues=0
ok() { printf 'OK   %s\n' "$*"; }
warn() { printf 'WARN %s\n' "$*"; issues=1; }

package_status="$(dpkg-query -W -f='${Status}' rpci-ime 2>/dev/null || true)"
package_version="$(dpkg-query -W -f='${Version}' rpci-ime 2>/dev/null || true)"
case "$package_status" in
    *"install ok installed"*)
        ok "system package rpci-ime ${package_version:-unknown}"
        ;;
    *)
        warn "system package rpci-ime is not installed"
        ;;
esac

USER_AUTOSTART="$CONFIG_HOME/autostart/rpci-ime.desktop"
if [ -f "$USER_AUTOSTART" ]; then
    warn "user autostart is still active: $USER_AUTOSTART"
else
    ok "user-level autostart is not active"
fi

ime_pids="$(pgrep -x ime-probe 2>/dev/null || true)"
ime_count="$(printf '%s\n' "$ime_pids" | sed '/^$/d' | wc -l | tr -d ' ')"
if [ "$ime_count" = "1" ]; then
    ok "one IME process is running"
elif [ "$ime_count" = "0" ]; then
    warn "no IME process is running"
else
    warn "$ime_count IME processes are running (expected 1)"
fi

if [ -f "$PANEL_WINGS" ] && grep -q "$APP_ID" "$PANEL_WINGS"; then
    ok "applet is registered in the top Panel"
elif [ -f "$DOCK_CENTER" ] && grep -q "$APP_ID" "$DOCK_CENTER"; then
    warn "applet is in the Dock; expected top Panel right wing"
else
    warn "applet is not registered in the top Panel"
fi

if [ -d "$RIME_DIR" ] && [ -w "$RIME_DIR" ]; then
    ok "RIME data directory is writable"
else
    warn "RIME data directory is missing or not writable: $RIME_DIR"
fi

if [ -n "${RPCI_DOCTOR_APT_SOURCES:-}" ]; then
    if grep -qs 'allenzhang45.github.io/rpci' "$RPCI_DOCTOR_APT_SOURCES"; then
        ok "public APT source is configured"
    else
        warn "public APT source is not configured"
    fi
elif grep -Rqs 'allenzhang45.github.io/rpci' /etc/apt/sources.list /etc/apt/sources.list.d 2>/dev/null; then
    ok "public APT source is configured"
else
    warn "public APT source is not configured"
fi

if [ -S "$RUNTIME/rpci/control.sock" ] && [ -f "$RUNTIME/rpci/state" ]; then
    ok "runtime control socket and state are present"
else
    warn "runtime control socket or state is missing"
fi

if [ "$issues" -eq 0 ]; then
    echo "doctor: all checks passed"
    exit 0
fi
echo "doctor: warnings found; review the lines above" >&2
exit 1
