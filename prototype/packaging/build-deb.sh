#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VERSION="${VERSION:-0.1.9}"
DIST="$ROOT/dist"
STAGE="$DIST/rpci-ime"
PACKAGE="$DIST/rpci-ime_${VERSION}_amd64.deb"

if [ -d /tmp/libclang/usr/lib/llvm-18/lib ]; then
    export LIBCLANG_PATH="${LIBCLANG_PATH:-/tmp/libclang/usr/lib/llvm-18/lib}"
    export BINDGEN_EXTRA_CLANG_ARGS="${BINDGEN_EXTRA_CLANG_ARGS:--I/tmp/libclang/usr/lib/llvm-18/lib/clang/18/include}"
fi

cd "$ROOT"
cargo build --release -p ime-probe \
    --bin ime-probe --bin ime-settings --bin ime-settings-gui --bin applet
cargo build --release -p rpci-settings-iced --bin rpci-settings-iced

rm -rf "$DIST"
mkdir -p "$STAGE"/usr/bin \
    "$STAGE/usr/share/applications" \
    "$STAGE/usr/share/rpci" \
    "$STAGE/usr/share/doc/rpci-ime" \
    "$STAGE/etc/xdg/autostart" \
    "$STAGE/DEBIAN"

install -Dm755 target/release/ime-probe "$STAGE/usr/bin/ime-probe"
install -Dm755 target/release/ime-settings "$STAGE/usr/bin/ime-settings"
install -Dm755 target/release/ime-settings-gui "$STAGE/usr/bin/ime-settings-gui"
install -Dm755 target/release/rpci-settings-iced "$STAGE/usr/bin/ime-settings-iced"
install -Dm755 target/release/applet "$STAGE/usr/bin/ime-applet"
install -Dm755 prototype/packaging/rpci-ime-launch.sh "$STAGE/usr/bin/rpci-ime-launch"
install -Dm755 prototype/packaging/rpci-ime-doctor.sh "$STAGE/usr/bin/rpci-ime-doctor"
printf '%s\n' "$VERSION" > "$STAGE/usr/share/rpci/version"
install -Dm755 prototype/scripts/setup-rime.sh "$STAGE/usr/bin/rpci-ime-setup"

# Build the RIME schema payload from the same schema sources as the installer.
rm -rf "$STAGE/usr/share/rpci/rime"
IME_RIME_USER_DIR="$STAGE/usr/share/rpci/rime" \
    "$ROOT/prototype/scripts/setup-rime.sh" >/dev/null
rm -f "$STAGE/usr/share/rpci/rime/build"
rm -rf "$STAGE/usr/share/rpci/rime/sync" "$STAGE/usr/share/rpci/rime/log"

cat > "$STAGE/usr/share/applications/rpci-settings.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=COSMIC IME 设置
Comment=Configure the COSMIC Wubi/Pinyin input method
Exec=/usr/bin/ime-settings-iced
Icon=preferences-desktop
Terminal=false
Categories=Settings;
StartupNotify=true
DESKTOP

cat > "$STAGE/usr/share/applications/rpci-settings-fallback.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=COSMIC IME 设置（兼容模式）
Comment=Keyboard-driven fallback settings window
Exec=/usr/bin/ime-settings-gui
Icon=preferences-desktop
Terminal=false
Categories=Settings;
DESKTOP

cat > "$STAGE/usr/share/applications/com.github.alec.RpciImeApplet.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=COSMIC IME
Comment=Wubi/Pinyin input method status
Exec=/usr/bin/ime-applet
Terminal=false
NoDisplay=true
X-CosmicApplet=true
X-HostWaylandDisplay=true
X-OverflowPriority=10
DESKTOP

cat > "$STAGE/etc/xdg/autostart/rpci-ime.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=COSMIC IME
Comment=Wubi/Pinyin input method for COSMIC
Exec=/usr/bin/rpci-ime-launch
Terminal=false
NoDisplay=true
X-GNOME-Autostart-enabled=true
OnlyShowIn=COSMIC;
DESKTOP

cp "$ROOT/prototype/README.md" "$STAGE/usr/share/doc/rpci-ime/README.md"
cp "$ROOT/docs/STATUS.md" "$STAGE/usr/share/doc/rpci-ime/STATUS.md"
cp "$ROOT/LICENSE" "$STAGE/usr/share/doc/rpci-ime/copyright"
cp "$ROOT/THIRD-PARTY-LICENSES.md" "$STAGE/usr/share/doc/rpci-ime/THIRD-PARTY-LICENSES.md"
mkdir -p "$STAGE/usr/share/doc/rpci-ime/LICENSES"
cp "$ROOT"/LICENSES/* "$STAGE/usr/share/doc/rpci-ime/LICENSES/"

INSTALLED_SIZE="$(du -sk "$STAGE" | cut -f1)"
cat > "$STAGE/DEBIAN/control" <<CONTROL
Package: rpci-ime
Version: $VERSION
Section: utils
Priority: optional
Architecture: amd64
Installed-Size: $INSTALLED_SIZE
Maintainer: rpci local build <root@localhost>
Depends: libc6, libgcc-s1, libstdc++6, libxkbcommon0, librime1t64, librime-data, librime-bin, libwayland-client0, libegl1, libgl1, libvulkan1, bash
Recommends: git
Description: COSMIC native Wubi/Pinyin input method
 A Wayland input-method-v2 client with RIME Wubi86 and Pinyin fallback,
 an iced settings window, XDG paths and systemd XDG autostart.
CONTROL

dpkg-deb --build --root-owner-group "$STAGE" "$PACKAGE" >/dev/null
echo "$PACKAGE"
