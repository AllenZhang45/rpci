# APT 仓库、签名、升级与回滚

> 状态：2026-10-20。仓库元数据签名已实现和自动验证；公共 HTTPS 源通过 [APT-PAGES.md](APT-PAGES.md) 的 GitHub Pages 工作流部署。

## 构建

```bash
./prototype/packaging/build-repo.sh
./prototype/packaging/test-repo.sh
```

默认输出：

```
dist/apt/
  dists/stable/InRelease
  dists/stable/Release
  dists/stable/Release.gpg
  dists/stable/main/binary-amd64/Packages
  dists/stable/main/binary-amd64/Packages.gz
  pool/main/r/rpci-ime/rpci-ime_0.1.6_amd64.deb
  rpci-ime-archive-keyring.asc
  rpci-ime-archive-keyring.gpg
  gnupg/                      # 默认本地测试密钥，不要提交或公开
```

`build-repo.sh` 会：

1. 调用 `build-deb.sh` 构建 `.deb`（可用 `RPCI_SKIP_BUILD=1` 跳过）；
2. 生成 `Packages` / `Packages.gz`；
3. 用 `apt-ftparchive` 生成带 SHA256 的 `Release`；
4. 生成 `Release.gpg` 和 `InRelease`；
5. 导出公钥 keyring。

## 签名密钥

本地默认在 `dist/apt/gnupg` 生成一个**无口令测试密钥**，只用于本机验证。

生产/发布使用自己的密钥：

```bash
export RPCI_GPG_HOME="$HOME/.gnupg-rpci"
export RPCI_GPG_KEY="<fingerprint-or-keyid>"
./prototype/packaging/build-repo.sh
```

要求：

- 私钥不要提交到仓库，也不要放在 `dist/apt` 之外的共享目录
- 发布前把 `rpci-ime-archive-keyring.gpg` 放到可信位置
- 公钥轮换时保留旧公钥一段时间，避免已配置客户端升级失败

## 客户端配置

本地文件仓库：

```bash
sudo install -m 0644 \
  /home/alec/rpci/dist/apt/rpci-ime-archive-keyring.gpg \
  /usr/share/keyrings/rpci-ime-archive-keyring.gpg

echo 'deb [signed-by=/usr/share/keyrings/rpci-ime-archive-keyring.gpg] file:/home/alec/rpci/dist/apt stable main' \
  | sudo tee /etc/apt/sources.list.d/rpci-ime.list

sudo apt update
sudo apt install rpci-ime
```

如果之前装过用户级 autostart，先移除：

```bash
rm -f ~/.config/autostart/rpci-ime.desktop
systemctl --user disable --now 'app-rpci\x2dime@autostart.service' 2>/dev/null || true
```

HTTPS 源只需把 `file:/...` 换成：

```
deb [signed-by=/usr/share/keyrings/rpci-ime-archive-keyring.gpg] https://example.com/rpci stable main
```

## 升级

```bash
sudo apt update
sudo apt install --only-upgrade rpci-ime
```

## 回滚

```bash
sudo apt update
sudo apt install rpci-ime=0.1.6
```

或先禁止自动升级：

```bash
sudo apt-mark hold rpci-ime
```

回滚后重启输入法：

```bash
systemctl --user restart 'app-rpci\x2dime@autostart.service'
```

## 自动验证

`test-repo.sh` 会：

- 生成 0.1.6 包和一个仅改版本号的 0.1.7 测试包
- 构建并签名仓库
- 用 `gpgv` 验证 `InRelease`
- 在隔离的 APT 目录中 `apt-get update`
- 断言候选版本为 0.1.6
- 断言默认安装解析到 0.1.6
- 断言 `rpci-ime=0.1.6` 解析到 0.1.6

在无 root 环境下使用宿主 `/var/lib/dpkg/status` 做依赖解析，不实际安装包。

## 当前限制

- 公共 HTTPS 已部署到 GitHub Pages；流程见 [APT-PAGES.md](APT-PAGES.md)
- 未做仓库内的自动构建/发布 CI
- `.deb` 本身未做二进制签名；当前签名保护的是 APT 元数据
- 仅 amd64
- 仓库的 0.1.7 仅用于升级/回滚验证，内容与 0.1.6 相同
