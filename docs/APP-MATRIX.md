# 应用矩阵

> 状态：2026-10-20。原型阶段的应用兼容性记录，按 [PRINCIPLES.md](PRINCIPLES.md) 第 4 条执行。

## 结论

- **通过**：合成 `zwp_text_input_v3` 客户端、cosmic-edit、LibreOffice Writer。
- **应用侧问题**：cosmic-term 能激活输入法，但提交没有写入 PTY。
- **上游问题**：Firefox 158、XWayland 下的 xterm 390 都没有收到 `Activate`，输入法无法接管。
- **未测**：Qt6、Electron/VS Code、Chromium、Flatpak 应用；当前环境没有可用样本。

## 矩阵

| 应用 | 版本 | 工具链 | 会话 | 激活 | 提交 | 备注 |
|---|---|---|---|---|---|---|
| 合成 text-input 客户端 | 本仓库 | SCTK | Wayland | ✅ | ✅ | 收到 `COMMIT "你"` |
| cosmic-edit | 1.10.0 | libcosmic | Wayland | ✅ | ✅ | Ctrl+S 后文件为 `你` |
| LibreOffice Writer | 24.2.7 | GTK3 | Wayland | ✅ | ✅ | ODT `content.xml` 含 `你` |
| cosmic-term | 1.10.0 | libcosmic | Wayland | ✅ | ⚠️ | 提交未进 PTY |
| Firefox | 158.0 | Gecko | Wayland | ❌ | — | 未收到 `Activate`，见 [FIREFOX.md](FIREFOX.md) |
| xterm | 390 | X11 / XWayland | XWayland | ❌ | — | 需要 XIM，当前不支持 |
| Qt6 应用 | — | Qt6 | — | 未测 | — | 环境无轻量 Qt6 应用；featherpad 是 Qt5 且缺 Qt5 库 |
| Electron/VS Code | — | Electron | — | 未测 | — | 未安装 |
| Chromium | — | Chromium | — | 未测 | — | 未安装 |
| Flatpak 应用 | — | — | — | 未测 | — | `flatpak list --app` 为空 |

## 说明

### cosmic-term

输入法能激活并收到按键，但 `commit_string` 没有写入终端 PTY。判断为终端自身的 IME 提交处理问题，记录为应用侧差异；后续可尝试升级 cosmic-term 或给它报 issue。

### Firefox

Firefox 158 已确认走 Wayland（加载 `libwayland-client`），但窗口/页面聚焦后没有 `Activate`。COSMIC 只暴露 `zwp_text_input_manager_v3` v1，疑似 Firefox 需要更新的 text-input 实现。详见 [FIREFOX.md](FIREFOX.md)。

### XWayland

xterm 390 通过 XWayland 运行。COSMIC 没有向输入法发送 `Activate`；虚拟键盘注入也没有观察到写入 xterm 的 `cat` 目标。X11 应用通常需要 XIM，而本项目当前只实现 Wayland 输入法协议，因此 XWayland 暂不支持。

### Qt6 / Electron / Chromium / Flatpak

- 系统有 Qt6 运行库，但没有可直接使用的 Qt6 应用。
- `featherpad` 是 Qt5，且缺少 Qt5 运行库，未作为矩阵样本。
- Electron/VS Code、Chromium 未安装。
- Flatpak 已安装，但没有已安装应用，无法测试 Flatpak 应用。

## 复现方法

- 合成客户端：`cargo run -p ime-probe --bin textinput`
- cosmic-edit：打开文件，输入 `wqiy` + 空格，再 Ctrl+S
- LibreOffice：打开 ODT，输入后 Ctrl+S，解压 `content.xml` 检查
- Firefox：`prototype/scripts/firefox-test-server.py` + `MOZ_ENABLE_WAYLAND=1 firefox`
- XWayland：`DISPLAY=:1 xterm -e 'cat > /tmp/xterm-ime.txt'`

## 后续

1. 找到可用的 Qt6、Electron、Chromium 样本，补充矩阵。
2. Flatpak 应用可用后，验证宿主侧输入法能否正常服务沙箱应用。
3. XWayland 若要支持，需要单独设计 XIM 或 X11 输入法路径，不在当前 Wayland 原型范围内。
4. Firefox 等上游 text-input 支持，详见 [FIREFOX.md](FIREFOX.md)。
