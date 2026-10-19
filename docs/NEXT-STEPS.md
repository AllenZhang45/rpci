# 后续建议

> 状态：2026-10-20。来自原型验收后的建议清单，按优先级排列；第 1 条正在执行。

## 1. Release 安装 + 注销/重启验证

### 为什么

当前安装测试使用的是 debug 构建、`IME_SKIP_BUILD=1`，并没有真正跑过默认 release 安装；autostart 文件也只在临时 HOME 中生成，没有经过注销/重启。

### 要做什么

- 用默认 `IME_PROFILE=release` 执行 `prototype/scripts/install.sh`
- 验证安装后的二进制、RIME 数据、配置、autostart 文件
- 用安装后的二进制实际输入一次
- 注销并重新登录，确认 autostart 自动拉起输入法

### 验收

- `~/.local/bin/ime-probe` 为 release 构建
- `~/.config/autostart/rpci-ime.desktop` 指向存在的可执行文件
- 安装后 `wqiy` + space 提交「你」
- 注销/重启后无需手动启动，输入法自动可用

### 当前状态

已完成：release 构建、真实 HOME 安装、systemd autostart 单元生成/链接/启动、安装后输入均已验证。用户于 2026-10-20 07:02 重启后，autostart 自动拉起（PID 1797），输入 `wqiy` 提交「你」。详见 [INSTALL-VERIFICATION.md](INSTALL-VERIFICATION.md)。

## 2. 独立设置 GUI

### 为什么

目前只有 `ime-settings` CLI，普通用户配置需要终端。

### 要做什么

- 基于 iced/libcosmic 或轻量 GTK 的独立设置窗口
- 覆盖：中/英默认、五笔/拼音兜底、候选数量、主题、词库备份/恢复
- 与 `config.toml` 双向同步；保存后提示重启输入法或热加载

### 验收

- 不依赖终端即可修改配置
- 修改后输入法行为变化
- 无 GTK/iced 运行环境时 CLI 仍可用

### 风险

iced/wgpu 在当前无 GPU 环境可能无法启动；需要准备软件渲染 fallback，或先做 CLI + 简单 TUI。

## 3. 真正的 COSMIC Panel Applet

### 为什么

当前状态指示是 layer-shell 胶囊，不参与面板布局。真正的 applet 才能融入 COSMIC 面板。

### 要做什么

- 确认 COSMIC panel 的第三方 applet 注册/通信机制
- 定义 applet 与 IME 进程的状态通道（IPC/共享状态）
- 实现中/英、五笔/拼音状态与点击切换

### 验收

- applet 出现在 COSMIC 面板中
- 状态与 IME 实时同步
- 点击可切换中英/模式

### 风险

上游第三方 applet API 未确认；如果引入 IPC，需要重新评审单进程架构。

## 4. Qt6 / Electron / Chromium / Flatpak 应用矩阵

### 为什么

当前矩阵只有 cosmic-edit、LibreOffice 通过；Qt6、Electron、Chromium、Flatpak 没有样本。

### 要做什么

- 找到可安装/可分发的 Qt6、Electron、Chromium 应用
- 准备至少一个 Flatpak 应用
- 对每个应用记录：激活、候选位置、提交、预编辑、快捷键
- 把结果写回 [APP-MATRIX.md](APP-MATRIX.md)

### 验收

- 四类应用各有明确结论（通过/失败/需上游）
- 失败项有日志与 workaround

### 风险

当前环境没有这些应用；可能需要下载 AppImage/Flatpak 或使用测试机。

## 5. 提交 Firefox 上游 Issue

### 为什么

Firefox 158 在 COSMIC 下完全不激活 IME；这是当前最大的应用缺口，且只能由上游解决。

### 要做什么

- 按 [FIREFOX.md](FIREFOX.md) 的草案整理日志和复现步骤
- 提交 Mozilla Bugzilla 和/或 COSMIC 上游
- 跟进回复，测试修复版本

### 验收

- issue 已提交并有链接
- 得到上游确认或修复
- 若确认无解，记录替代方案

### 风险

需要上游账号与网络；当前会话未提交。

## 6. XWayland / XIM 支持

### 为什么

xterm 390 在 XWayland 下没有 `Activate`，X11 应用需要 XIM 或等效路径。

### 要做什么

- 评估 XIM 服务端方案（fcitx5/ibus 已有实现可参考）
- 或明确不支持 XWayland，给出文档与替代应用建议
- 如果要支持，需要单独设计进程模型与协议层

### 验收

- XWayland 应用能输入中文；或
- 文档明确“不支持”，并说明原因和替代方案

### 风险

XIM 是另一套协议栈，工作量和维护成本高，不建议在当前 Wayland 原型内直接做。

## 建议执行顺序

1. 完成第 1 条（release 安装 + 注销/重启验证）
2. 提交第 5 条（Firefox issue，成本低、影响大）
3. 做第 2 条（设置 GUI）
4. 补第 4 条（应用矩阵）
5. 评估第 3 条（panel applet，依赖上游）
6. 最后评估第 6 条（XWayland/XIM）
