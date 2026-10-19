#!/usr/bin/env bash
set -euo pipefail

SHARED_DIR="${IME_RIME_SHARED_DIR:-/usr/share/rime-data}"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
CACHE_HOME="${XDG_CACHE_HOME:-$HOME/.cache}"
case "$DATA_HOME" in /*) ;; *) DATA_HOME="$HOME/.local/share" ;; esac
case "$CACHE_HOME" in /*) ;; *) CACHE_HOME="$HOME/.cache" ;; esac
USER_DIR="${IME_RIME_USER_DIR:-$DATA_HOME/rpci/rime}"

if [ ! -d "$SHARED_DIR" ]; then
    echo "missing RIME shared data: $SHARED_DIR" >&2
    echo "install librime-data (or set IME_RIME_SHARED_DIR)" >&2
    exit 1
fi

clone_or_update() {
    local repo="$1"
    local dir="$2"
    if [ -d "$dir/.git" ]; then
        git -C "$dir" pull --ff-only >/dev/null 2>&1 || true
    else
        git clone --depth 1 "$repo" "$dir"
    fi
}

mkdir -p "$USER_DIR"
SCHEMA_SRC="${IME_RIME_SCHEMA_SRC:-}"
if [ -n "$SCHEMA_SRC" ] && [ -d "$SCHEMA_SRC" ]; then
    cp "$SCHEMA_SRC/wubi86.schema.yaml" "$USER_DIR/"
    cp "$SCHEMA_SRC/wubi86.dict.yaml" "$USER_DIR/"
    cp "$SCHEMA_SRC/wubi_pinyin.schema.yaml" "$USER_DIR/"
    cp "$SCHEMA_SRC/pinyin_simp.schema.yaml" "$USER_DIR/"
    cp "$SCHEMA_SRC/pinyin_simp.dict.yaml" "$USER_DIR/"
else
    clone_or_update https://github.com/rime/rime-wubi.git /tmp/rime-wubi-src
    clone_or_update https://github.com/rime/rime-pinyin-simp.git /tmp/rime-pinyin-simp-src
    cp /tmp/rime-wubi-src/wubi86.schema.yaml "$USER_DIR/"
    cp /tmp/rime-wubi-src/wubi86.dict.yaml "$USER_DIR/"
    cp /tmp/rime-wubi-src/wubi_pinyin.schema.yaml "$USER_DIR/"
    cp /tmp/rime-pinyin-simp-src/pinyin_simp.schema.yaml "$USER_DIR/"
    cp /tmp/rime-pinyin-simp-src/pinyin_simp.dict.yaml "$USER_DIR/"
fi

# A full default.yaml would shadow the shared presets (key_binder etc.).
# Use default.custom.yaml to patch only the schema list.
rm -f "$USER_DIR/default.yaml"
cat > "$USER_DIR/default.custom.yaml" <<'EOF'
patch:
  schema_list:
    - schema: wubi86
EOF

# Keep compiled RIME artifacts in XDG_CACHE_HOME, not in the data directory.
CACHE_BUILD="$CACHE_HOME/rpci/rime-build"
mkdir -p "$CACHE_BUILD"
if [ -d "$USER_DIR/build" ] && [ ! -L "$USER_DIR/build" ]; then
    cp -a "$USER_DIR/build/." "$CACHE_BUILD/" 2>/dev/null || true
    rm -rf "$USER_DIR/build"
fi
ln -sfn "$CACHE_BUILD" "$USER_DIR/build"

echo "RIME user data prepared at $USER_DIR"
echo "RIME build cache: $CACHE_BUILD"
