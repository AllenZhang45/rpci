# Changelog

所有值得记录的变更都在这里。格式参考 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)。

## [Unreleased]

### Added

- 候选翻页：组合输入时 `PageUp` / `PageDown` 在当前候选页前后翻页，单页时不显示页码。
- 候选窗布局配置：`candidate_layout = "single" | "two-row" | "auto"`，默认单行。
- 两行布局第一行显示预编辑和页码，第二行显示编号候选。

### Changed

- 候选窗高度按布局动态创建；默认仍为 520×48。

## [0.1.4] - 2026-10-23

### Added

- 本地物理键盘发布检查：`prototype/scripts/test-physical.sh`（使用 `/dev/uinput`，不进入 CI，不打包）。
- ADR 0005：日常安装走系统 APT，Panel applet 只读。
- 只读诊断命令 `rpci-ime-doctor`。
- 版本变化时自动创建 RIME/配置快照，校验失败则阻止启动。
- 设计与依赖证据审计：[docs/EVIDENCE-AUDIT.md](docs/EVIDENCE-AUDIT.md)。
- `rpci-ime-doctor` 校验 `/usr/share/rpci/version` 与已安装包版本是否一致。
- VS Code Wayland IME 启动参数说明：[docs/VSCODE.md](docs/VSCODE.md)。

### CI

- GitHub Actions 改为仅发版 tag 触发；日常 push 和文档提交不运行 CI。

### Changed

- 修正 STATUS 中 applet、APT 签名、Release 和版本状态等过期事实。
- 系统包 launcher 检测到用户级 autostart 时，将其重命名为 `.disabled-by-system-package`；保留配置、数据和二进制。

## [0.1.3] - 2026-10-23

### Added

- COSMIC Epoch 1.11 兼容性审计：[docs/COSMIC-1.11-AUDIT.md](docs/COSMIC-1.11-AUDIT.md)。
- 在 1.11 上验证中文、`Ctrl+Space`、物理键盘英文透传和 applet。

### Fixed

- COSMIC 1.11 升级重置 Panel 配置后，applet 不再出现：launcher 只检查 active 配置，忽略 `.bak`，并重新放入顶部 Panel 右侧。
- launcher 在旧的 Dock/center 配置中清理重复 applet。

### Changed

- 文档基线从 COSMIC 1.10 更新到 1.11。

## [0.1.2] - 2026-10-20

### Added

- 公共 HTTPS APT 源部署到 `https://allenzhang45.github.io/rpci/`，使用新轮换的签名密钥。
- 实际 HTTPS `apt-get update` 验证通过，候选版本为 `0.1.2`。
- Release 强制使用 APT 签名密钥；每次发布自动构建并附加 `rpci-ime-apt-repo.tar.gz`。
- APT Pages 工作流在 Release 发布后自动更新公共仓库。

### Changed

- APT 签名密钥轮换为 `38DE9A94D8BD8F4A`；泄露的旧密钥停用。

## [0.1.1] - 2026-10-20

### Fixed

- 修复 `Ctrl+Space` 切换中英后 text-input 失焦：Ctrl 先暂存，不把半按住的 Ctrl 转发给应用。
- 修复物理键盘在英文模式下无法输入：非修饰键改为在物理释放后发送完整虚拟 tap，避免合成事件与仍按下的物理键冲突。
- 切换后显式清理虚拟键盘的 Ctrl 修饰状态，避免 `Ctrl+字母` 被应用当成快捷键。
- 修复 APT 仓库测试在包已安装时无法解析回滚 URI 的问题。

### Changed

- 中英切换统一使用 `Ctrl+Space`；不再使用 F2。
- Panel applet 改为只读状态显示，放入顶部 Panel 右侧 `plugins_wings`，不放在 Dock。
- applet 不再提供点击切换和齿轮按钮，避免点击顶部面板导致焦点离开编辑器。
- Panel applet 改用 SCTK 只读窗口，移除 iced applet 依赖。
- 安装/打包脚本优先把 applet 放入顶部 Panel，Dock 仅作为回退。

### Added

- 物理键盘英文透传的 tap 日志和 `CTRL+SPACE` / `control` 来源区分，便于诊断。
- Git 仓库、GitHub Actions CI 和 `vX.Y.Z` Release 工作流。
- 签名 APT 仓库、升级/回滚测试和 GitHub Pages 发布工作流。
- Chrome、Flatpak、Qt6、XWayland 应用矩阵结果与审计文档。
- DSH 工程技能及项目适配文档。
- README 安装教程和项目总览。

## [0.1.0] - 2026-10-20

### Added

- COSMIC Wayland 输入法：`zwp_input_method_v2`、`zwp_text_input_v3`、`input_popup_surface`。
- RIME Wubi86 优先、拼音兜底。
- 候选窗、明暗主题、HiDPI 2x 和 HIG 圆角。
- 中英切换、中文标点、Enter 换行、快捷键透传。
- iced 设置窗口和 SCTK 兼容设置窗口。
- XDG 配置/数据/缓存分离。
- systemd XDG autostart。
- Debian 包构建与验证。
