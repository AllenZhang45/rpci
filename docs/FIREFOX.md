# Firefox 上游跟进

> 状态：2026-10-20。问题已定位到 Firefox/COSMIC 的 text-input 兼容层，不在输入法自身代码内。

## 结论

Firefox 158 在 COSMIC Wayland 下可以正常显示窗口并聚焦测试页面，但输入法进程始终收不到 `Activate`，因此无法接管键盘。COSMIC 只暴露 `zwp_text_input_manager_v3` v1；Firefox 很可能使用了更新版本的 text-input 实现，需要上游确认。

## 环境

- Firefox：158.0，BuildID 20261007201254
- COSMIC：cosmic-session 1.10.0，cosmic-comp 0.1~1791979042
- Wayland 全局：
  - `zwp_input_method_manager_v2` v1
  - `zwp_text_input_manager_v3` v1
- Firefox 进程已加载 `libwayland-client`，确认不是 XWayland

## 复现步骤

```bash
# 1. 启动测试页
python3 prototype/scripts/firefox-test-server.py

# 2. 启动 Firefox
MOZ_ENABLE_WAYLAND=1 firefox --no-remote --new-window http://127.0.0.1:8765/

# 3. 聚焦窗口
./target/debug/focus firefox

# 4. 启动输入法并观察日志
XDG_RUNTIME_DIR=/run/user/1000 WAYLAND_DISPLAY=wayland-1 \
  ./target/debug/ime-probe
```

## 观察结果

- Firefox 窗口标题为 `IME test — Mozilla Firefox`，toplevel 状态为 Activated。
- 页面 `textarea` 有 `autofocus`，并主动调用 `focus()`。
- 输入法日志中只有自身的初始化，没有 `IM event: Activate`。
- 因此 `grab_keyboard` 不会发生；`vkey` 注入也不会进入输入法。
- 测试页的 `/tmp/ime-firefox.txt` 不会产生内容。

## 假设

1. Firefox 158 的 Wayland text-input 实现与 COSMIC 暴露的 v1 不匹配（例如需要更新的协议版本/行为）。
2. 也可能是 Firefox 没有把 HTML textarea 的焦点上报给合成器；但从窗口 Activated 和页面 autofocus 看，前一种可能性更大。
3. 输入法自身没有收到任何 `Activate`，所以问题不在 `commit_string`、候选窗或 RIME。

## 上游 issue 草案

标题：

> Firefox 158 on COSMIC Wayland does not activate zwp_input_method_v2 (text-input-v3 v1)

正文要点：

- 环境：Pop!_OS 24.04 / COSMIC 1.10 / Firefox 158 Wayland。
- COSMIC 暴露 `zwp_input_method_manager_v2` v1 和 `zwp_text_input_manager_v3` v1。
- 自建最小输入法（绑定 input-method-v2）在 GTK3/LibreOffice、libcosmic 应用中能收到 `Activate`，但在 Firefox 中收不到。
- 测试页 textarea 已 autofocus；Firefox 窗口状态为 Activated。
- 期望：Firefox 聚焦文本框时向合成器启用 text-input，触发输入法 `Activate`。
- 实际：输入法从未收到 `Activate`，无法输入中文。
- 附：`wayland-info` 输出、Firefox 版本、最小输入法日志。

## 临时方案

- 在 Firefox 修复前，使用 cosmic-edit、LibreOffice 等已验证应用。
- 不建议用 XWayland 作为替代：xterm 测试同样没有 `Activate`，需要 XIM。
- Fcitx5 未在本环境验证；即使使用 Fcitx5，如果 Firefox 没有启用 text-input，问题可能仍然存在。

## 下一步

1. 用 Firefox Nightly / 更新版 COSMIC 复测。
2. 收集完整协议日志（`WAYLAND_DEBUG=1`）并提交 Mozilla Bugzilla / COSMIC 上游。
3. 若上游确认是协议版本问题，评估本项目是否需要适配新版本。
