# GitHub Actions CI

> 状态：2026-10-20。已覆盖格式检查、测试、`.deb` 构建、包校验、签名 APT 仓库验证和 tag Release。

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

## Release 工作流

文件：[.github/workflows/release.yml](../.github/workflows/release.yml)

触发：

```bash
git tag v0.1.0
git push origin v0.1.0
```

流程：

1. 校验 tag 为 `vX.Y.Z`
2. 安装与 CI 相同的依赖
3. 用 tag 版本执行完整 `ci.sh`
4. 可选：如果配置了 `APT_GPG_PRIVATE_KEY` secret，构建签名 APT 仓库并打包
5. 创建/更新 GitHub Release，附加 `.deb`（可选 APT 仓库 tar）

可选 secrets：

- `APT_GPG_PRIVATE_KEY`：正式 APT 签名私钥
- `APT_GPG_KEY_ID`：对应 key ID/fingerprint

未配置 secrets 时只发布 `.deb`，不会带测试签名仓库。

## CI 不覆盖什么

- 没有 COSMIC/Wayland 图形会话，因此不运行真实输入法、候选窗和 panel applet
- 不安装 `.deb` 到系统，只做解包和脚本级验证
- APT 仓库测试使用临时 key 和隔离 apt 目录，不发布
- Release 只附 `.deb`；公共 HTTPS APT 由独立的 Pages 工作流部署
- 不做正式密钥轮换/备份
- 可选 nightly 真实 COSMIC 会话 E2E 未做

## 实测记录

- CI run `39433064379`：成功
- Release run `39433071645`：成功
- Release：[v0.1.0](https://github.com/AllenZhang45/rpci/releases/tag/v0.1.0)
- 资产：`rpci-ime_0.1.0_amd64.deb`

未配置 `APT_GPG_PRIVATE_KEY`，因此本次 Release 只有 `.deb`，没有附签名 APT 仓库 tar。

## 下一步

- 配置 `APT_GPG_PRIVATE_KEY` / `APT_GPG_KEY_ID` 并启用 Pages
- 运行 APT Pages 工作流，验证 HTTPS 客户端安装
- 可选 nightly 真实 COSMIC 会话 E2E
- 非 amd64 构建

公共 HTTPS APT 流程见 [APT-PAGES.md](APT-PAGES.md)。
