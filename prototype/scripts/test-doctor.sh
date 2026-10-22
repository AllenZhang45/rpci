#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

mkdir -p "$TMP/bin" "$TMP/xdg/cosmic/com.system76.CosmicPanel.Panel/v1" \
    "$TMP/data/rpci/rime" "$TMP/runtime/rpci"
cat > "$TMP/bin/dpkg-query" <<'EOF'
#!/bin/sh
case "$*" in
    *'${Status}'*) echo 'install ok installed' ;;
    *'${Version}'*) echo '0.1.3' ;;
esac
EOF
cat > "$TMP/bin/pgrep" <<'EOF'
#!/bin/sh
echo 4242
EOF
chmod +x "$TMP/bin/dpkg-query" "$TMP/bin/pgrep"
printf 'Some(([\n], [\n    "com.github.alec.RpciImeApplet",\n    "com.system76.CosmicAppletInputSources",\n]))\n' \
    > "$TMP/xdg/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings"
printf 'deb [signed-by=/tmp/key] https://allenzhang45.github.io/rpci stable main\n' \
    > "$TMP/apt-sources"
python3 - "$TMP/runtime/rpci/control.sock" <<'PY'
import socket, sys
socket.socket(socket.AF_UNIX, socket.SOCK_STREAM).bind(sys.argv[1])
PY
printf '中 五\n' > "$TMP/runtime/rpci/state"

XDG_CONFIG_HOME="$TMP/xdg" \
XDG_DATA_HOME="$TMP/data" \
XDG_RUNTIME_DIR="$TMP/runtime" \
RPCI_DOCTOR_APT_SOURCES="$TMP/apt-sources" \
PATH="$TMP/bin:$PATH" \
    "$ROOT/prototype/packaging/rpci-ime-doctor.sh" >"$TMP/out" 2>"$TMP/err"
grep -q 'doctor: all checks passed' "$TMP/out"

rm -f "$TMP/xdg/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings"
if XDG_CONFIG_HOME="$TMP/xdg" \
    XDG_DATA_HOME="$TMP/data" \
    XDG_RUNTIME_DIR="$TMP/runtime" \
    RPCI_DOCTOR_APT_SOURCES="$TMP/apt-sources" \
    PATH="$TMP/bin:$PATH" \
        "$ROOT/prototype/packaging/rpci-ime-doctor.sh" >"$TMP/out2" 2>&1; then
    echo "doctor should fail when applet is missing" >&2
    exit 1
fi
grep -q 'applet is not registered' "$TMP/out2"

echo "doctor tests passed"
