# Qt6 / QtWayland 诊断

> 日期：2026-10-20。使用 `diagnosing-bugs` 流程；结论是上游 Qt 6.4.2 不支持 text-input-v3。

## 症状

Qt 6.4.2 的 `textedit` / `lineedits` 在 COSMIC Wayland 下：

- 窗口正常显示、能获得键盘焦点
- 输入法没有收到 `Activate`
- `wqiy` 等按键原样进入 Qt 控件

## 反馈回路

使用 Qt6 示例和独立 IME 日志：

```bash
# 1. 停止 autostart，启动带日志的 IME
systemctl --user stop 'app-rpci\x2dime@autostart.service'
./target/debug/ime-probe >/tmp/qt6-ime.log 2>&1 &

# 2. 启动 Qt6 lineedits，打开 QtWayland 日志
QT_QPA_PLATFORM=wayland \
QT_LOGGING_RULES='qt.qpa.wayland*=true' \
LD_LIBRARY_PATH=/tmp/qt6-probe/root/usr/lib/x86_64-linux-gnu \
  /tmp/qt6-probe/root/usr/lib/x86_64-linux-gnu/qt6/examples/widgets/widgets/lineedits/lineedits \
  >/tmp/qt6-app.log 2>&1 &

# 3. 聚焦并触发输入
./target/debug/focus lineedits
./target/debug/vkey tab

# 4. 判定
grep 'Activate' /tmp/qt6-ime.log        # 空 => 失败
grep -i 'text.input\|input method' /tmp/qt6-app.log
```

基线输出：

```text
qt.qpa.wayland: QT_WAYLAND_TEXT_INPUT_PROTOCOL= ""
qt.qpa.wayland: using input method: QComposeInputContext
```

IME 日志没有 `Activate`。

## 最小化与变体

强制指定 v3：

```bash
QT_WAYLAND_TEXT_INPUT_PROTOCOL=zwp_text_input_v3
```

输出：

```text
qt.qpa.wayland: QT_WAYLAND_TEXT_INPUT_PROTOCOL= "zwp_text_input_v3"
qt.qpa.wayland: text input: unknown protocol -  "zwp_text_input_v3"
qt.qpa.wayland: using input method: QComposeInputContext
```

强制 v2：

```bash
QT_WAYLAND_TEXT_INPUT_PROTOCOL=zwp_text_input_v2
```

输出：

```text
qt.qpa.wayland: QT_WAYLAND_TEXT_INPUT_PROTOCOL= "zwp_text_input_v2"
qt.qpa.wayland: using input method: QComposeInputContext
```

检查 QtWayland 客户端库：

```bash
strings /usr/lib/x86_64-linux-gnu/libQt6WaylandClient.so.6 \
  | grep -E 'text_input_v[0-9]|text_input_method'
```

结果只出现：

```text
qt_text_input_method_v1
zwp_text_input_v1
zwp_text_input_v2
```

没有 `zwp_text_input_v3`。

## 根因

**Qt 6.4.2 的 QtWayland 客户端不支持 `zwp_text_input_v3`。**

- Qt 只识别 v1/v2 的 text-input 协议
- COSMIC 1.10 只暴露 `zwp_text_input_manager_v3`
- 因此 Qt 退回 `QComposeInputContext`，不会请求输入法
- IME 没有收到 `Activate`，不是 IME 的协议实现问题

## 影响

- Qt6 6.4.2 应用在当前 COSMIC 上不能使用本 IME
- 这与 Firefox 的“无 Activate”现象相似，但根因不同：Qt 是明确缺少 v3 客户端支持

## 可行方向

1. **升级 Qt** 到确认支持 text-input-v3 的版本，再复测（当前环境只有 6.4.2）
2. 向上游 Qt 提交/查找 text-input-v3 支持 issue
3. 临时方案：XWayland + XIM，或使用非 Qt 应用
4. 不建议 IME 侧伪造 v1/v2；COSMIC 没有对应 manager 全局，且协议层不应模拟

## 清理

- 没有修改生产代码
- Qt6 示例和临时 `/tmp` 文件仅用于诊断
- autostart IME 已恢复运行，状态文件为 `中 五`
