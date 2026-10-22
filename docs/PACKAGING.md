# Debian 打包

> 状态：2026-10-20。提供可本地构建、可检查、可安装的 `.deb`；尚未建立签名仓库。

## 构建

```bash
./prototype/packaging/build-deb.sh
```

输出：

```
dist/rpci-ime_0.1.3_amd64.deb
```

脚本会：

1. 构建 release 二进制
2. 把 RIME schema 复制到 `/usr/share/rpci/rime`
3. 生成应用启动器和 `/etc/xdg/autostart` 单元
4. 用 `dpkg-deb --build --root-owner-group` 打包

## 包内容

| 路径 | 说明 |
|---|---|
| `/usr/bin/ime-probe` | 输入法主程序 |
| `/usr/bin/ime-settings` | 配置 CLI |
| `/usr/bin/ime-settings-iced` | iced 设置窗口 |
| `/usr/bin/ime-settings-gui` | SCTK 兼容设置窗口 |
| `/usr/bin/ime-applet` | COSMIC 顶部 Panel 只读 applet |
| `/usr/bin/rpci-ime-launch` | 初始化 RIME、维护 applet、版本变化时快照，然后启动 IME |
| `/usr/bin/rpci-ime-doctor` | 只读安装健康检查，含版本匹配 |
| `/usr/share/rpci/version` | 构建时写入的期望版本，供 doctor 校验 |
| `/usr/bin/rpci-ime-setup` | 把 schema 复制到用户 XDG data |
| `/etc/xdg/autostart/rpci-ime.desktop` | 系统级 XDG autostart |
| `/usr/share/applications/rpci-settings*.desktop` | 应用菜单入口 |
| `/usr/share/applications/com.github.alec.RpciImeApplet.desktop` | Panel applet 注册 |
| `/usr/share/rpci/rime/` | Wubi86 / Pinyin schema |
| `/usr/share/doc/rpci-ime/` | README、STATUS |

## 安装

```bash
sudo apt install ./dist/rpci-ime_0.1.3_amd64.deb
```

登录 COSMIC 后，`rpci-ime-launch` 会检查：

```
$XDG_DATA_HOME/rpci/rime/default.custom.yaml
```

不存在时，用包内 schema 执行 `rpci-ime-setup`，然后启动 IME。
launcher 还会在 `~/.config/cosmic/com.system76.CosmicPanel.Dock/v1/plugins_center` 中加入 applet（如果尚未存在）。

> 如果之前用 `install.sh` 安装过用户级 autostart：
> `~/.config/autostart/rpci-ime.desktop`
> 请先删除，避免与 `/etc/xdg/autostart/rpci-ime.desktop` 重复。

设置入口：应用菜单 **COSMIC IME 设置**。

## 验证

```bash
./prototype/packaging/test-deb.sh
```

验证内容：

- `.deb` 元数据可读
- 七个二进制存在且可执行（含 `ime-applet`）
- schema 文件存在
- 三个启动器通过 `desktop-file-validate`（含 applet）
- 用包内 schema 执行 setup，生成用户 RIME 数据
- 用 fake `ime-probe` 执行 `rpci-ime-launch`，确认首次初始化 + 启动路径

实测输出：

```
package verification passed: /home/alec/rpci/dist/rpci-ime_0.1.3_amd64.deb
```

另外用解包后的 `ime-probe` 做过一次真实输入：`wqiy` + space → 「你」。

## 依赖

```
libc6, libgcc-s1, libstdc++6, libxkbcommon0,
librime1t64, librime-data, libwayland-client0,
libegl1, libgl1, libvulkan1, bash
```

`librime-data` 提供 `/usr/share/rime-data` 共享 preset；包内只带 Wubi86/Pinyin schema。

## 卸载

```bash
sudo apt remove rpci-ime
```

用户数据保留：

```
~/.config/rpci/
~/.local/share/rpci/
~/.cache/rpci/
```

## APT 仓库

签名仓库、客户端配置、升级和回滚见 [APT-REPO.md](APT-REPO.md)。

CI 流程和本地复现见 [CI.md](CI.md)。

## 限制

- 目前仅构建 `amd64`。
- 本地签名 APT 仓库已可构建和验证；尚未上传公共 HTTPS 源。
- 仓库签名保护元数据，`.deb` 本身未做二进制签名。
- 未在 CI 中自动构建/发布。
- `test-repo.sh` 的 0.1.4 仅改版本号，用于升级/回滚解析验证，内容与 0.1.3 相同。
- iced 设置窗口依赖 GL/Vulkan；无 GPU 环境可改用兼容启动器。
