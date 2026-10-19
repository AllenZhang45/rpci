# 协议能力审计（COSMIC 1.10.0）

> 日期：2026-10-20
> 环境：Pop!_OS 24.04 / COSMIC Epoch 1.10.0 / Wayland
> 目标：确认 `zwp_input_method_v2` 相关协议在目标版本的真实可用性，而不是只依据协议名称。

## Wayland 全局

`wayland-info` 实测：

| 全局 | 版本 |
|---|---|
| `zwp_input_method_manager_v2` | 1 |
| `zwp_text_input_manager_v3` | 1 |
| `zwp_virtual_keyboard_manager_v1` | 1 |
| `zwlr_layer_shell_v1` | 5 |
| `wp_fractional_scale_manager_v1` | 1 |

`zwp_input_popup_surface_v2` 不是 registry 全局，它由输入法对象通过 `get_input_popup_surface` 创建；本项目已实际使用该路径。

## 已验证能力

- **激活**：cosmic-edit、LibreOffice、合成 text-input 客户端能触发 `Activate`。
- **键盘抓取**：激活后 `grab_keyboard` 成功，xkb keymap 从 fd 加载。
- **提交**：`commit_string` 在验证应用中上屏。
- **候选窗**：`input_popup_surface` 创建成功，键盘事件中的光标矩形驱动位置；候选窗可渲染并提交。
- **透传**：方向键、Ctrl 组合、F1 通过释放 grab + `zwp_virtual_keyboard_v1` 注入，再重新 grab。
- **扩展协议**：`wp_fractional_scale_manager_v1` 存在；候选窗目前固定 2x，未接 fractional scale。

## 单 seat 单输入法

在第一个输入法运行时启动第二个实例：

```
IM event: Unavailable
another input method is already active on this seat; exiting
```

结论：COSMIC 1.10 强制执行一个 seat 一个 `zwp_input_method_v2` 对象。第二个实例不能静默接管；当前实现选择直接退出。

## cosmic-osk 共存

- 系统正在运行 `cosmic-osk` 1.10.0（PID 1826）。
- `cosmic-osk` 二进制包含 `zwp_text_input_v3` / `get_text_input` 路径，是 text-input 的**客户端**；未发现它绑定 `zwp_input_method_v2` 的证据。
- 当前会话中，本项目输入法持有 input-method 槽位，同时 `cosmic-osk` 运行，没有观察到两者同时争抢槽位。
- 仍未验证：如果 `cosmic-osk` 在未来版本改为占用 input-method 槽位，或启动顺序改变，第三方输入法可能收到 `Unavailable`。这需要上游确认仲裁策略。

## 应用侧边界

- Firefox 158：没有 `Activate`；COSMIC 只提供 text-input-v3 v1，疑似上游版本/实现不匹配。
- XWayland（xterm 390）：没有 `Activate`；X11 应用需要 XIM，当前实现不覆盖。
- cosmic-term：激活成功，但 `commit_string` 未写入 PTY。

这些是应用/上游问题，不是 input-method 协议层能单方面修复的。

## 结论

第一层（协议）当前实现符合 `zwp_input_method_v2` + `zwp_text_input_v3` + `zwp_input_popup_surface_v2` 的基本工作流，并在 COSMIC 1.10 上实际工作。需要持续审计：

1. fractional scale / 多显示器 / 屏幕边缘；
2. `cosmic-osk` 的槽位仲裁；
3. Firefox / XWayland 的上游兼容性；
4. 输入法崩溃或重启时的 `Unavailable` 恢复策略。
