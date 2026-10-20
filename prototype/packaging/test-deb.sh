#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VERSION="${VERSION:-0.1.1}"
PACKAGE="${1:-$ROOT/dist/rpci-ime_${VERSION}_amd64.deb}"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

[ -f "$PACKAGE" ] || { echo "missing package: $PACKAGE" >&2; exit 1; }
dpkg-deb --info "$PACKAGE" >/dev/null
dpkg-deb --extract "$PACKAGE" "$TMP/root"

for bin in ime-probe ime-settings ime-settings-gui ime-settings-iced ime-applet rpci-ime-launch rpci-ime-setup; do
    test -x "$TMP/root/usr/bin/$bin" || { echo "missing $bin" >&2; exit 1; }
done
for schema in wubi86.schema.yaml wubi86.dict.yaml pinyin_simp.schema.yaml pinyin_simp.dict.yaml default.custom.yaml; do
    test -f "$TMP/root/usr/share/rpci/rime/$schema" || { echo "missing $schema" >&2; exit 1; }
done
desktop-file-validate "$TMP/root/usr/share/applications/rpci-settings.desktop"
desktop-file-validate "$TMP/root/usr/share/applications/rpci-settings-fallback.desktop"
desktop-file-validate "$TMP/root/usr/share/applications/com.github.alec.RpciImeApplet.desktop"

XDG_CACHE_HOME="$TMP/cache" \
IME_RIME_SCHEMA_SRC="$TMP/root/usr/share/rpci/rime" \
IME_RIME_USER_DIR="$TMP/data/rpci/rime" \
    "$TMP/root/usr/bin/rpci-ime-setup" >/dev/null
test -f "$TMP/data/rpci/rime/default.custom.yaml"

mkdir -p "$TMP/fakebin"
cat > "$TMP/fakebin/ime-probe" <<'FAKE'
#!/bin/sh
echo launched > "$RPCI_TEST_MARKER"
FAKE
chmod +x "$TMP/fakebin/ime-probe"
mkdir -p "$TMP/xdg/cosmic/com.system76.CosmicPanel.Panel/v1"
printf 'Some(([\n], [\n    "com.system76.CosmicAppletInputSources",\n]))\n' \
    > "$TMP/xdg/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings"

RPCI_TEST_MARKER="$TMP/marker" \
XDG_DATA_HOME="$TMP/launch" \
XDG_CONFIG_HOME="$TMP/xdg" \
XDG_CACHE_HOME="$TMP/cache" \
RPCI_IME_BIN_DIR="$TMP/fakebin" \
RPCI_IME_SETUP="$TMP/root/usr/bin/rpci-ime-setup" \
RPCI_IME_SCHEMA_DIR="$TMP/root/usr/share/rpci/rime" \
    "$TMP/root/usr/bin/rpci-ime-launch"
test -f "$TMP/marker"
grep -q 'com.github.alec.RpciImeApplet' \
    "$TMP/xdg/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings"

echo "package verification passed: $PACKAGE"
