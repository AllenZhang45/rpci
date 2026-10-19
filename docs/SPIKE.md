# 4 周 Spike 计划

> 状态：草案（2026-10-20）。第 1 周只验证"能不能"，不写完整功能；代码可丢弃。

## 目标

用最小代价回答五个问题：

1. COSMIC 是否允许第三方 IME 被激活、收到按键？
2. 能否把文本提交到目标应用？
3. 候选 popup 能否正确显示和定位？
4. 未处理按键能否透传给应用？
5. RIME 能否接入并稳定工作？

第 1 周只回答 1–4；第 2 周回答 5。

## 前置条件

- Pop!_OS 24.04 / COSMIC，Wayland 会话。
- Rust 工具链（当前环境未安装 cargo，需先装 rustup）。
- Wayland/xkb 开发库（`libwayland-dev`、`libxkbcommon-dev`）。
- librime 开发包，或从源码构建。
- 测试应用：GTK4 应用、Qt 应用、VS Code、Firefox/Chromium。
- 能查看会话日志（journalctl 或临时开 debug 日志）。

## 第 1 周：协议最小闭环

### Day 1：环境与协议侦察

- [ ] 记录 COSMIC/cosmic-comp 版本和会话类型（确认是 Wayland）。
- [ ] 用 `wayland-info` 确认 compositor 暴露的全局：`zwp_input_method_manager_v2`、`zwp_text_input_manager_v3`、`zwp_virtual_keyboard_manager_v1`、`wl_seat`。
- [ ] 确认是否有 fcitx5/ibus 在跑，先停掉，避免冲突。
- [ ] 记录结论：哪些协议可用，哪些缺失。

### Day 2：最小 IME 客户端

- [ ] Rust 程序连接 Wayland，绑定 input-method manager。
- [ ] 日志打印 activate/deactivate。
- [ ] 聚焦 GTK4 文本框，确认是否收到 activate。
- [ ] 记录：激活是否由 compositor 主动发起。

### Day 3：键盘 grab 与按键

- [ ] 请求 grab_keyboard，确认收到 key 事件。
- [ ] 用 xkbcommon 把 keycode 转 keysym，打印日志。
- [ ] 测试字母、数字、退格、方向键、Ctrl 组合。
- [ ] 记录：grab 期间应用是否还收到按键（判断是否互斥）。

### Day 4：提交与 popup

- [ ] `commit_string("测试")` + `commit`，在 GTK4 应用中确认上屏。
- [ ] 在 Qt、VS Code、浏览器地址栏重复。
- [ ] 创建 input_popup_surface，画一个纯色矩形，确认位置跟随光标。
- [ ] 在屏幕底部测试翻转/裁切；有条件时在 HiDPI 下测试缩放。
- [ ] 记录：哪些应用提交失败，popup 是否正常。

### Day 5：透传与结论

- [ ] 测试英文模式/未处理按键：字母、方向键、Ctrl+C。
- [ ] 若按键被吞，尝试 virtual-keyboard 转发，记录是否可行。
- [ ] 填"协议能力表"：激活、grab、commit、popup、透传、HiDPI。
- [ ] 写结论：继续 / 降级 / 转 Fcitx5 前端。
- [ ] 决策门：
  - 激活或 grab 不成立 → 停止"完全原生"路线。
  - popup 不成立 → 候选窗降级为简化方案。
  - 透传不成立 → 改由合成器管理中英切换，或只做中文模式。

## 第 1 周交付物

- 一个最小程序：能启动、被激活、收键、提交"测试"、画一个 popup 矩形。
- 一份协议能力表 + 结论。
- 不接 RIME，不做候选 UI，不做配置。

## 后三周（概要）

- 第 2 周：接 RIME，跑通五笔 86 + 拼音兜底；core 单元测试。
- 第 3 周：候选条、预编辑、定位/翻转/HiDPI；4 类应用联调。
- 第 4 周：应用矩阵验收、性能测量、修 top 问题、演示。

## 结果记录（待填）

| 能力 | 结果 | 备注 |
|---|---|---|
| 激活 |  |  |
| grab |  |  |
| commit（GTK4/Qt/Electron/浏览器） |  |  |
| popup 定位/翻转 |  |  |
| 按键透传 |  |  |
| HiDPI |  |  |
| RIME 接入（第 2 周） |  |  |
