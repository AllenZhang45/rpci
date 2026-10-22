#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VERSION="${VERSION:-0.1.3}"
REPO="${RPCI_REPO_DIR:-$ROOT/dist/apt}"
KEY_HOME="${RPCI_GPG_HOME:-$REPO/gnupg}"
SKIP_BUILD="${RPCI_SKIP_BUILD:-0}"
DEBS="${RPCI_DEBS:-$ROOT/dist/rpci-ime_${VERSION}_amd64.deb}"

if [ "$SKIP_BUILD" != "1" ]; then
    "$ROOT/prototype/packaging/build-deb.sh"
fi

for deb in $DEBS; do
    [ -f "$deb" ] || {
        echo "missing package: $deb" >&2
        exit 1
    }
done

rm -rf "$REPO/dists" "$REPO/pool"
mkdir -p "$REPO/dists/stable/main/binary-amd64" "$REPO/pool/main/r/rpci-ime"

for deb in $DEBS; do
    cp "$deb" "$REPO/pool/main/r/rpci-ime/"
done

(
    cd "$REPO"
    dpkg-scanpackages --multiversion pool \
        > dists/stable/main/binary-amd64/Packages
    gzip -9c dists/stable/main/binary-amd64/Packages \
        > dists/stable/main/binary-amd64/Packages.gz
    apt-ftparchive \
        -o APT::FTPArchive::Release::Origin="RPCI IME" \
        -o APT::FTPArchive::Release::Label="RPCI IME" \
        -o APT::FTPArchive::Release::Suite="stable" \
        -o APT::FTPArchive::Release::Codename="stable" \
        -o APT::FTPArchive::Release::Architectures="amd64" \
        -o APT::FTPArchive::Release::Components="main" \
        -o APT::FTPArchive::Release::Description="RPCI IME APT repository" \
        release dists/stable > dists/stable/Release
)

mkdir -p "$KEY_HOME"
chmod 700 "$KEY_HOME"

if [ -n "${RPCI_GPG_KEY:-}" ]; then
    KEY_ID="$RPCI_GPG_KEY"
elif gpg --homedir "$KEY_HOME" --list-secret-keys --with-colons 2>/dev/null | grep -q '^sec:'; then
    KEY_ID="$(gpg --homedir "$KEY_HOME" --list-secret-keys --with-colons 2>/dev/null \
        | awk -F: '/^sec:/{print $5; exit}')"
else
    echo "generating local signing key in $KEY_HOME" >&2
    gpg --batch --homedir "$KEY_HOME" \
        --pinentry-mode loopback --passphrase '' \
        --quick-gen-key "RPCI IME Archive (local) <rpci@localhost>" rsa3072 sign never
    KEY_ID="$(gpg --homedir "$KEY_HOME" --list-secret-keys --with-colons 2>/dev/null \
        | awk -F: '/^sec:/{print $5; exit}')"
fi

if [ -z "$KEY_ID" ]; then
    echo "no signing key found" >&2
    exit 1
fi

gpg --homedir "$KEY_HOME" --batch --yes --local-user "$KEY_ID" \
    --armor --detach-sign -o "$REPO/dists/stable/Release.gpg" \
    "$REPO/dists/stable/Release"
gpg --homedir "$KEY_HOME" --batch --yes --local-user "$KEY_ID" \
    --clearsign -o "$REPO/dists/stable/InRelease" \
    "$REPO/dists/stable/Release"

gpg --homedir "$KEY_HOME" --armor --export "$KEY_ID" \
    > "$REPO/rpci-ime-archive-keyring.asc"
gpg --homedir "$KEY_HOME" --export "$KEY_ID" \
    > "$REPO/rpci-ime-archive-keyring.gpg"
chmod 644 "$REPO/rpci-ime-archive-keyring.asc" "$REPO/rpci-ime-archive-keyring.gpg"

echo "repository: $REPO"
echo "key id: $KEY_ID"
echo "public key: $REPO/rpci-ime-archive-keyring.asc"
