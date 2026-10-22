# GitHub Actions（仅 Release）

> 状态：2026-10-23。GitHub Actions 只在发版 tag 时运行；日常 push 和 docs 提交不触发 Actions。

## 触发方式

只有 Release 工作流：

```bash
git tag v0.1.4
git push origin v0.1.4
```

文件：[.github/workflows/release.yml](../.github/workflows/release.yml)

流程：

1. 校验 tag 为 `vX.Y.Z`
2. 安装构建依赖
3. 使用 tag 版本执行完整 `prototype/scripts/ci.sh`
4. 构建签名 APT 仓库并打包
5. 创建/更新 GitHub Release，附加 `.deb` + `rpci-ime-apt-repo.tar.gz`
6. 触发 APT Pages 更新公共仓库

## 本地验证

发布前手动运行：

```bash
./prototype/scripts/ci.sh
```

它会执行：

```bash
cargo fmt --all --check
cargo test --locked -p ime-probe
cargo test --locked -p rpci-settings-iced
./prototype/scripts/check-docs.py
./prototype/scripts/test-doctor.sh
./prototype/packaging/build-deb.sh
./prototype/packaging/test-deb.sh
./prototype/packaging/test-repo.sh
```

日常 push、文档修改、纯文档提交都不再触发 GitHub Actions。

## Release 必需配置

- `APT_GPG_PRIVATE_KEY`：APT 签名私钥，必填
- `APT_GPG_KEY_ID`：key ID/fingerprint，可留空自动推导

缺少签名密钥时 Release 会失败，避免发布未签名仓库。

## 不覆盖

- 不运行真实 COSMIC/Wayland 图形会话
- 不安装 `.deb` 到系统
- 不做 nightly 真实输入 E2E
- 不做正式密钥轮换

## 实测

- v0.1.2、v0.1.3 Release 已成功
- v0.1.4 Release 使用相同的 tag-only 流程
- APT Pages 由 Release 工作流显式触发

公共 HTTPS APT 流程见 [APT-PAGES.md](APT-PAGES.md)。
