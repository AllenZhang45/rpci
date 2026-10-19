#!/usr/bin/env bash
set -euo pipefail

SHARED_DIR="${IME_RIME_SHARED_DIR:-/usr/share/rime-data}"
USER_DIR="${IME_RIME_USER_DIR:-/tmp/rime-user}"

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

clone_or_update https://github.com/rime/rime-wubi.git /tmp/rime-wubi-src
clone_or_update https://github.com/rime/rime-pinyin-simp.git /tmp/rime-pinyin-simp-src

mkdir -p "$USER_DIR"
cp /tmp/rime-wubi-src/wubi86.schema.yaml "$USER_DIR/"
cp /tmp/rime-wubi-src/wubi86.dict.yaml "$USER_DIR/"
cp /tmp/rime-wubi-src/wubi_pinyin.schema.yaml "$USER_DIR/"
cp /tmp/rime-pinyin-simp-src/pinyin_simp.schema.yaml "$USER_DIR/"
cp /tmp/rime-pinyin-simp-src/pinyin_simp.dict.yaml "$USER_DIR/"

# A full default.yaml would shadow the shared presets (key_binder etc.).
# Use default.custom.yaml to patch only the schema list.
rm -f "$USER_DIR/default.yaml"
cat > "$USER_DIR/default.custom.yaml" <<'EOF'
patch:
  schema_list:
    - schema: wubi86
EOF

echo "RIME user data prepared at $USER_DIR"
