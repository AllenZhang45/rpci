# Changelog

所有值得记录的变更都在这里。格式参考 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)。

## [Unreleased]

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
