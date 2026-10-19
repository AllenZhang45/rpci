# 当前状态

> 最后更新：2026-10-20。这里是当前事实；历史过程见 [history/](history/)。

## 一句话

一个可安装、可自启动、能实际打字的 COSMIC 原生中文五笔/拼音输入法原型，已通过 cosmic-edit、LibreOffice 和合成 text-input 客户端的验证。

## 技术栈

- Rust 2024，单进程 Wayland 客户端
- COSMIC Epoch 1.10 / Pop!_OS 24.04
- RIME（Wubi86 + Pinyin 兜底）
- `zwp_input_method_v2`、`zwp_text_input_v3`、`zwp_input_popup_surface_v2`
- 候选窗：raw `wl_shm` + cosmic-text，COSMIC HIG 风格
- 设置：iced 0.14 独立窗口；另有自绘 SCTK 回退

## 已工作

| 能力 | 状态 |
|---|---|
| 激活 / grab / commit | ✅ |
| 五笔 86 + 拼音兜底 | ✅ |
| 数字选词、退格、Esc 取消 | ✅ |
| 候选窗显示、主题、HiDPI 2x、HIG 圆角 | ✅ |
| Enter 换行 | ✅（中文模式经 `commit_string("\n")`） |
| 方向键 / Ctrl 组合 / F1 透传 | ✅ |
| 中英切换（F2 / Ctrl+Space） | ✅ |
| 中文标点 | ✅ |
| XDG 配置 / 数据 / 缓存分离 | ✅ |
| release 安装 + systemd autostart | ✅ |
| iced 设置界面 + 应用菜单入口 | ✅ |
| 词库备份 / 导出 / 导入 | ✅ |
| layer-shell 状态胶囊 | ✅ |

## 验证基线

```bash
cargo test -p ime-probe           # 34 passed
cargo test -p rpci-settings-iced  # 2 passed
```

已实测：

- `wqiy` + space → 「你」
- `nihao` + space → 「你好」
- cosmic-edit：输入并保存，文件内容验证
- cosmic-edit：Enter 后保存，文件新增换行
- LibreOffice Writer：ODT `content.xml` 含「你」
- 候选窗实际屏幕截图：[assets/popup-visible.png](assets/popup-visible.png)
- 重启后系统 autostart 自动拉起 IME

## 已知限制

### 应用/上游

- Firefox 158：不发送 `Activate`，见 [FIREFOX.md](FIREFOX.md)。
- XWayland：xterm 390 不激活，需要 XIM；当前不支持。
- cosmic-term：激活成功，但提交没有写入 PTY。
- Qt6 / Electron / Chromium / Flatpak：环境无可用样本，未测。

### 输入法自身

- 真正的 COSMIC panel applet 尚未实现；当前是 layer-shell 胶囊。
- 候选窗未接 fractional scale；屏幕边缘翻转未验证。
- 每个 seat 只能有一个输入法对象；与 `cosmic-osk` 的长期仲裁需要上游确认。
- 设置保存后需要重启输入法才生效；没有热加载。
- 词库恢复 UI 只有 CLI；备份有 GUI。
- libcosmic 未采用，设置使用 iced 0.14。

### 构建/安装

- 构建依赖 `libclang`；本机使用 `/tmp/libclang` 临时解包，重启后需恢复。
- 安装脚本生成 XDG autostart，由 systemd 的 XDG autostart generator 接管。
- `desktop-file-validate` 会对 `OnlyShowIn=COSMIC` 报“未注册”警告，这是 desktop-file-utils 的注册表问题；systemd 的 `ExecCondition` 实际通过。

## 文档导航

从 [README.md](README.md) 开始。当前事实看本页；计划看 [ROADMAP.md](ROADMAP.md)；架构看 [ARCHITECTURE.md](ARCHITECTURE.md) 和 [ADR 0004](adr/0004-three-layer.md)。
