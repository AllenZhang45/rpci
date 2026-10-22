#!/usr/bin/env bash
set -euo pipefail
trap 'code=$?; echo "::error title=CI failed::command failed at line $LINENO with exit $code"' ERR

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

if [ -d /tmp/libclang/usr/lib/llvm-18/lib ]; then
    export LIBCLANG_PATH="${LIBCLANG_PATH:-/tmp/libclang/usr/lib/llvm-18/lib}"
    export BINDGEN_EXTRA_CLANG_ARGS="${BINDGEN_EXTRA_CLANG_ARGS:--I/tmp/libclang/usr/lib/llvm-18/lib/clang/18/include}"
fi

cargo fmt --all --check
cargo test --locked -p ime-probe
cargo test --locked -p rpci-settings-iced
./prototype/scripts/check-docs.py
./prototype/scripts/test-doctor.sh

./prototype/packaging/build-deb.sh
./prototype/packaging/test-deb.sh
./prototype/packaging/test-repo.sh
