#!/bin/sh
set -eu

BIN_DIR="${RPCI_IME_BIN_DIR:-/usr/bin}"
SETUP="${RPCI_IME_SETUP:-$BIN_DIR/rpci-ime-setup}"
SCHEMA_DIR="${RPCI_IME_SCHEMA_DIR:-/usr/share/rpci/rime}"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
case "$DATA_HOME" in /*) ;; *) DATA_HOME="$HOME/.local/share" ;; esac
RIME_DIR="${IME_RIME_USER_DIR:-$DATA_HOME/rpci/rime}"

if [ ! -f "$RIME_DIR/default.custom.yaml" ]; then
    IME_RIME_SCHEMA_SRC="$SCHEMA_DIR" "$SETUP" >/dev/null 2>&1 || true
fi

exec "$BIN_DIR/ime-probe"
