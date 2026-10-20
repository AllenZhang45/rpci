# GitHub Actions CI

> 状态：2026-10-20。已覆盖格式检查、测试、`.deb` 构建、包校验和签名 APT 仓库验证；未做发布。

## 工作流

文件：[.github/workflows/ci.yml](../.github/workflows/ci.yml)

触发：

- push 到 `main`
- pull request
- 手动 `workflow_dispatch`

运行环境：Ubuntu 24.04。

步骤：

1. 安装构建依赖（libclang、librime、Wayland、XKB、EGL/GL/Vulkan、GnuPG、dpkg）
2. 安装稳定版 Rust + rustfmt
3. 缓存 Rust build
4. 运行 `prototype/scripts/ci.sh`
5. 上传 `dist/rpci-ime_*.deb` 为构建产物

## 本地复现

```bash
./prototype/scripts/ci.sh
```

它会依次执行：

```bash
cargo fmt --all --check
cargo test --locked -p ime-probe
cargo test --locked -p rpci-settings-iced
./prototype/scripts/check-docs.py
./prototype/packaging/build-deb.sh
./prototype/packaging/test-deb.sh
./prototype/packaging/test-repo.sh
```

如果本机有 `/tmp/libclang/usr/lib/llvm-18/lib`，脚本会自动设置 `LIBCLANG_PATH` 和 `BINDGEN_EXTRA_CLANG_ARGS`。

## 文档链接检查

`prototype/scripts/check-docs.py` 检查：

- `docs/**/*.md`
- `prototype/README.md`
- `GLOSSARY.md`

忽略 `http://`、`https://`、`mailto:` 和纯锚点链接。

## CI 不覆盖什么

- 没有 COSMIC/Wayland 图形会话，因此不运行真实输入法、候选窗和 panel applet
- 不安装 `.deb` 到系统，只做解包和脚本级验证
- APT 仓库测试使用临时 key 和隔离 apt 目录，不发布
- 不做签名密钥管理、HTTPS 部署和 GitHub Release

## 下一步

- 发布工作流：tag 触发、产物附加到 Release
- 使用仓库 secret 注入正式 APT 签名密钥
- 可选的 nightly 真实 COSMIC 会话 E2E
