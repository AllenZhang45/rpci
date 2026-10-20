#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TMP="$(mktemp -d /tmp/rpci-repo-test.XXXXXX)"
case "$TMP" in
    /tmp/rpci-repo-test.*) ;;
    *) echo "unexpected temp dir: $TMP" >&2; exit 1 ;;
esac
trap 'rm -rf "$TMP"' EXIT

VERSION="${VERSION:-0.1.0}"
DEB="$ROOT/dist/rpci-ime_${VERSION}_amd64.deb"
if [ ! -f "$DEB" ]; then
    "$ROOT/prototype/packaging/build-deb.sh" >/dev/null
fi

mkdir -p "$TMP/versions"
cp "$DEB" "$TMP/versions/"
dpkg-deb -R "$DEB" "$TMP/pkg"
sed -i 's/^Version: .*/Version: 0.1.1/' "$TMP/pkg/DEBIAN/control"
dpkg-deb --build --root-owner-group "$TMP/pkg" \
    "$TMP/versions/rpci-ime_0.1.1_amd64.deb" >/dev/null

RPCI_DEBS="$TMP/versions/rpci-ime_${VERSION}_amd64.deb $TMP/versions/rpci-ime_0.1.1_amd64.deb" \
RPCI_SKIP_BUILD=1 \
RPCI_REPO_DIR="$TMP/repo" \
RPCI_GPG_HOME="$TMP/gnupg" \
    "$ROOT/prototype/packaging/build-repo.sh" >/dev/null

gpgv --keyring "$TMP/repo/rpci-ime-archive-keyring.gpg" \
    "$TMP/repo/dists/stable/InRelease" >/dev/null 2>&1
grep -q '^Version: 0.1.0$' "$TMP/repo/dists/stable/main/binary-amd64/Packages"
grep -q '^Version: 0.1.1$' "$TMP/repo/dists/stable/main/binary-amd64/Packages"

CLIENT="$TMP/client"
mkdir -p "$CLIENT/etc/apt/sources.list.d" \
    "$CLIENT/etc/apt/preferences.d" \
    "$CLIENT/var/lib/apt/lists/partial" \
    "$CLIENT/var/cache/apt/archives/partial"
cp "$TMP/repo/rpci-ime-archive-keyring.gpg" "$CLIENT/rpci-ime.gpg"
printf 'deb [signed-by=%s] file:%s stable main\n' \
    "$CLIENT/rpci-ime.gpg" "$TMP/repo" > "$CLIENT/etc/apt/sources.list"

OPTS=(
    -o "Dir::Etc::sourcelist=$CLIENT/etc/apt/sources.list"
    -o "Dir::Etc::sourceparts=$CLIENT/etc/apt/sources.list.d"
    -o "Dir::Etc::trusted=$CLIENT/etc/apt/trusted.gpg"
    -o "Dir::Etc::trustedparts=$CLIENT/etc/apt/trusted.gpg.d"
    -o "Dir::State=$CLIENT/var/lib/apt"
    -o "Dir::State::status=/var/lib/dpkg/status"
    -o "Dir::Cache=$CLIENT/var/cache/apt"
    -o "Dir::Etc::preferences=$CLIENT/etc/apt/preferences"
    -o "Dir::Etc::preferencesparts=$CLIENT/etc/apt/preferences.d"
    -o "APT::Sandbox::User=$(id -un)"
    -o "Debug::NoLocking=1"
)

LC_ALL=C apt-get "${OPTS[@]}" update >/dev/null 2>&1
POLICY="$(LC_ALL=C apt-cache "${OPTS[@]}" policy rpci-ime)"
printf '%s\n' "$POLICY" | grep -q 'Candidate: 0.1.1'
printf '%s\n' "$POLICY" | grep -q '0.1.1'
printf '%s\n' "$POLICY" | grep -q '0.1.0'

UPDATE_URI="$(LC_ALL=C apt-get "${OPTS[@]}" --print-uris --yes install rpci-ime 2>&1)"
printf '%s\n' "$UPDATE_URI" | grep -q 'rpci-ime_0.1.1_amd64.deb'

ROLLBACK_URI="$(LC_ALL=C apt-get "${OPTS[@]}" --print-uris --yes install rpci-ime=0.1.0 2>&1)"
printf '%s\n' "$ROLLBACK_URI" | grep -q 'rpci-ime_0.1.0_amd64.deb'

echo "apt repository verification passed"
echo "  update candidate: 0.1.1"
echo "  rollback target: 0.1.0"
echo "  keyring: $TMP/repo/rpci-ime-archive-keyring.gpg"
