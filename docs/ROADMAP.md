# 路线图

> 状态：2026-10-20。当前事实以 [STATUS.md](STATUS.md) 为准；历史验证见 [history/](history/)。
> 本文件是唯一的路线图；`NEXT-STEPS.md` 已合并到这里。

## 已完成

### 原型能力

| 项目 | 结果 |
|---|---|
| Wayland input-method 链路 | ✅ `zwp_input_method_v2` + `text_input_v3` + `input_popup_surface_v2` |
| 五笔 + 拼音兜底 | ✅ |
| 候选窗 | ✅ 显示、主题、HiDPI 2x、COSMIC HIG 圆角 |
| 透传 / 中英切换 / 标点 | ✅ |
| Enter 换行 | ✅ 中文模式经 `commit_string("\n")` |
| 应用验证 | ✅ cosmic-edit、LibreOffice、合成 text-input 客户端 |
| 测试 | ✅ `ime-probe` 34、`rpci-settings-iced` 2 |

### 产品化基础

| 项目 | 结果 |
|---|---|
| release 安装 | ✅ 真实 HOME 安装 |
| autostart | ✅ systemd XDG autostart，重启后自动拉起 |
| 配置 | ✅ XDG 路径 + CLI |
| 设置界面 | ✅ iced 0.14 主入口；SCTK 回退 |
| 词库备份/导出/导入 | ✅ CLI；GUI 支持备份 |
| 状态指示 | ✅ layer-shell 胶囊（非 panel applet） |
| 架构/审计 | ✅ ADR 0004、协议/XDG/UI 审计 |

## 下一阶段

按优先级排列。每项完成后更新 [STATUS.md](STATUS.md)，不要只改本文件。

### 1. 提交 Firefox 上游 Issue

**为什么**：Firefox 158 在 COSMIC 下不发送 `Activate`，这是当前最大的应用缺口。

**做什么**：

- 按 [FIREFOX.md](FIREFOX.md) 的草案收集日志与版本
- 提交 Mozilla Bugzilla / COSMIC 上游
- 跟进回复，测试修复版本

**验收**：issue 有链接；得到确认/修复；或形成有依据的“暂不支持”结论。

### 2. 设置界面收尾

**为什么**：iced 主入口已可用，但仍有明显缺口。

**做什么**：

- 配置热加载（当前用“保存并应用”+ systemd 重启替代）
- 词库恢复 UI（当前只有 CLI 导入）
- 鼠标交互与视觉细节验证
- 如需完全 COSMIC 原生控件，评估 libcosmic 迁移

**验收**：热加载或一键重启可用；恢复流程可在 GUI 完成；真实 COSMIC 会话目视通过。

### 3. 应用矩阵补齐

**为什么**：Qt6、Electron、Chromium、Flatpak 没有样本。

**做什么**：

- 找到可安装/可分发的 Qt6、Electron、Chromium 应用
- 准备至少一个 Flatpak 应用
- 记录激活、候选位置、提交、预编辑、快捷键
- 结果写回 [APP-MATRIX.md](APP-MATRIX.md)

**验收**：四类应用各有明确结论；失败项有日志与 workaround。

### 4. 真正的 COSMIC Panel Applet

**为什么**：当前是 layer-shell 胶囊，不参与面板布局。

**做什么**：

- 确认 COSMIC panel 的第三方 applet 注册/通信机制
- 定义 applet 与 IME 进程的状态通道
- 实现中/英、五笔/拼音状态与点击切换

**验收**：applet 出现在面板；状态实时同步；点击可切换。

**风险**：上游 applet API 未确认；引入 IPC 需要重新评审单进程架构。

### 5. XWayland / XIM

**为什么**：xterm 390 在 XWayland 下没有 `Activate`。

**做什么**：

- 评估 XIM 服务端方案，或
- 明确“不支持 XWayland”，给出替代应用建议

**验收**：XWayland 能输入中文；或文档明确不支持并说明原因。

**风险**：XIM 是另一套协议栈，不建议在当前 Wayland 原型内直接实现。

### 6. 打包与升级

**为什么**：目前只有 `install.sh`，没有 `.deb`/升级/回滚流程。

**做什么**：

- 评估 Pop!_OS/COSMIC 的 `.deb` 打包
- 安装、升级、卸载、词库保留
- 构建依赖（librime/libclang）处理

**验收**：一条命令安装/升级；卸载保留用户词库；版本可回滚。

## 执行顺序

1. Firefox 上游 issue（成本低、影响大）
2. 设置界面收尾
3. 应用矩阵补齐
4. 打包与升级
5. Panel applet（依赖上游）
6. XWayland/XIM（最后评估）

## 非目标

- Windows/macOS、云输入、AI 联想、多语言、皮肤市场
- Flatpak 版输入法宿主（IME 需在宿主侧运行）
- 与搜狗五笔 100% 行为一致

## 风险与依赖

- COSMIC 输入法/面板/设置 API 仍在演进。
- 一个 seat 只能有一个 `zwp_input_method_v2` 对象；`cosmic-osk` 的长期仲裁需上游确认。
- 候选窗受 `input_popup_surface` 协议约束，不能换成普通 iced 窗口。
- 应用矩阵依赖环境中可用的 Qt/Electron/Flatpak 样本。

## 参考

- [STATUS.md](STATUS.md)
- [ARCHITECTURE.md](ARCHITECTURE.md)
- [ADR 0004](adr/0004-three-layer.md)
- [PROTOCOL-AUDIT.md](PROTOCOL-AUDIT.md)
- [XDG-AUDIT.md](XDG-AUDIT.md)
- [UI-AUDIT.md](UI-AUDIT.md)
- [APP-MATRIX.md](APP-MATRIX.md)
- [FIREFOX.md](FIREFOX.md)
