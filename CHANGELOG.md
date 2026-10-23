# Changelog

所有值得记录的变更都在这里。格式参考 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)。

## [0.1.9] - 2026-10-23

### Added

- `Ctrl+Shift+1..9` 删除当前候选页对应候选，使用 RIME `delete_candidate_on_current_page`；不重建用户词库。

### Fixed

- 删除候选后刷新当前组合，避免候选窗停留在旧状态。

## [0.1.8] - 2026-10-23

### Added

- 设置界面新增只读用户词库查看与导出，使用 `rime_dict_manager` 读取 `wubi86` / `pinyin_simp` 用户词条。
- 用户词库导出到 `~/.local/share/rpci/exports/<dict>-userdict.tsv`。
- `.deb` 增加 `librime-bin` 依赖，提供 `rime_dict_manager`。

### Changed

- 用户词库加载走 stop → export → start；失败也会尝试恢复输入法。

## [0.1.7] - 2026-10-23

### Changed

- 设置界面的配置状态与副作用收进 `SettingsController`；新增 `SettingsBackend` seam，备份/恢复/重启流程可脱离 iced 测试。

## [0.1.6] - 2026-10-23

### Added

- 项目采用 GPL-3.0-or-later；新增 `LICENSE`、`THIRD-PARTY-LICENSES.md` 和完整第三方许可证文本。
- `.deb` 包安装许可证文件到 `/usr/share/doc/rpci-ime/`。
- 新增 `InputEngine` seam 和 `FakeEngine` 测试替身，按键行为测试不再依赖真实 RIME。

### Changed

- 候选窗生命周期收进 `CandidateWindow`：surface、buffer、尺寸、测量和绘制不再由 `main` 逐步编排。
- 按键状态机收进 `InputSession`；main 只执行 Action。
- Cargo 元数据补充 description、repository、homepage、authors、keywords、categories 和许可证。
- README 更新许可证、RIME 引用、VS Code 说明和两行候选窗截图。

### Fixed

- GitHub Release 更新日志只提取当前版本，不再带入历史版本。

## [0.1.5] - 2026-10-23

### Added

- 候选翻页：组合输入时 `PageUp` / `PageDown` 在当前候选页前后翻页，单页时不显示页码。
- 候选窗布局配置：`candidate_layout = "single" | "two-row" | "auto"`，默认单行。
- iced 设置界面新增“候选布局”选择：单行 / 两行 / 自动。
- 两行布局第一行显示预编辑和右侧页码，第二行显示编号候选和翻页箭头。
- 候选窗宽度使用 cosmic-text 实际排版宽度自适应，范围 200–760 逻辑像素。
- 候选窗高度按实际行高计算；两行之间加入反色分割线和额外行距。

### Changed

- 候选窗缓冲区尺寸随布局和内容宽度重建；默认仍为单行。
- 两行布局高度由 96 收紧到约 60 逻辑像素。

### Fixed

- 删空组合后不再残留 `第 1 页` / 翻页箭头候选窗。
- 宽度测量前先执行 cosmic-text shape，避免宽度退化为最小值导致候选被裁切。

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
