# ADR 0004：三层架构约定

> 状态：Accepted（2026-10-20）
> 适用版本：COSMIC Epoch 1.10.0 / Wayland
> 关联：[PROTOCOL-AUDIT.md](../PROTOCOL-AUDIT.md)、[XDG-AUDIT.md](../XDG-AUDIT.md)、[UI-AUDIT.md](../UI-AUDIT.md)

## 背景

项目此前已经确定三条原则：协议层遵循 Wayland 标准、配置层遵循 XDG、界面层遵循 COSMIC HIG。实践后需要补充两个边界：

1. **遵循 Wayland 协议不等于所有应用都能输入中文。** 协议存在、合成器实现、应用实现是三件事。
2. **使用 libcosmic/iced 不等于候选窗自动获得输入法能力。** 候选窗必须使用 `zwp_input_popup_surface_v2` 与输入上下文协作，普通 GUI 窗口不能替代。

## 决策

保留三层架构，并明确每层的职责与验证方式。

### 第一层：协议与系统集成

- 输入法使用 `zwp_input_method_v2` 接收激活/上下文事件并提交文本。
- 键盘抓取使用 `zwp_input_method_keyboard_grab_v2`。
- 应用侧文本输入使用 `zwp_text_input_v3`（由合成器转发）。
- 候选窗使用 `zwp_input_popup_surface_v2`，位置由输入上下文的光标矩形驱动。
- 透传使用 `zwp_virtual_keyboard_v1`。
- 不自行发明私有键盘协议。

约束：

- `zwp_input_method_v2` 仍是实验性协议；必须针对 COSMIC 1.10.0 的实际实现做审计。
- 一个 seat 同时只能有一个输入法对象；需要确认与 `cosmic-osk` 的共存/仲裁。
- 应用是否启用 text-input 由应用决定；Firefox、XWayland 的失败属于应用/上游问题，不能靠 IME 单方面修复。

验证：见 [PROTOCOL-AUDIT.md](../PROTOCOL-AUDIT.md)。

### 第二层：核心引擎与 XDG 数据

- 拼音、五笔、候选排序、用户词典与 UI 解耦；`HybridEngine` 不依赖窗口系统。
- 目录职责分离：

| 内容 | 路径 |
|---|---|
| 配置 | `${XDG_CONFIG_HOME:-~/.config}/rpci/` |
| 数据（RIME 用户数据、备份） | `${XDG_DATA_HOME:-~/.local/share}/rpci/` |
| 缓存（RIME build） | `${XDG_CACHE_HOME:-~/.cache}/rpci/` |
| 运行时 | `$XDG_RUNTIME_DIR/rpci/` |
| autostart | `${XDG_CONFIG_HOME:-~/.config}/autostart/` |
| 应用启动器 | `${XDG_DATA_HOME:-~/.local/share}/applications/` |

- XDG 环境变量为相对路径时视为无效，回退默认目录；`XDG_RUNTIME_DIR` 没有默认值，且必须是绝对路径。
- 配置目录不是词典目录；词典与用户数据必须可独立备份/迁移。

验证：见 [XDG-AUDIT.md](../XDG-AUDIT.md)。

### 第三层：COSMIC 原生界面

- 设置界面优先使用 libcosmic/iced，继承 COSMIC 主题、字体、间距和控件。
- 候选窗使用 COSMIC HIG 的视觉语言（颜色、圆角、间距、字体），但窗口角色必须是 `zwp_input_popup_surface_v2`；不能用普通 libcosmic 窗口假装候选窗。
- 候选窗必须单独验证定位、焦点、缩放、多显示器和屏幕边缘行为。
- 输入法核心不得依赖 UI 框架。

验证：见 [UI-AUDIT.md](../UI-AUDIT.md)。

## 已确认的风险

- **协议实验性**：`zwp_input_method_v2` 版本 1，不同合成器行为可能不同。
- **单 seat 单输入法**：第二个实例会收到 `Unavailable`。`cosmic-osk` 1.10 正在运行；当前证据显示它使用 `zwp_text_input_v3` 作为客户端，而不是抢占 input-method 槽位，但仍需上游确认启动顺序/仲裁。
- **应用兼容性**：Firefox 158、XWayland 应用不激活 IME；cosmic-term 不把提交写入 PTY。
- **UI 框架边界**：iced/libcosmic 不能直接渲染 `zwp_input_popup_surface_v2`；候选窗继续使用自绘 + HIG 样式。

## 后果

- 新增代码必须明确属于哪一层；UI 改动不得侵入 `HybridEngine`。
- 协议/XDG/UI 三个审计文档作为验收依据，随版本更新。
- 设置界面使用 iced 作为主入口，保留自绘 SCTK 窗口作为兼容回退。
- 候选窗的视觉可以迭代，但协议角色不可替换。

## 相关文档

- [架构说明](../ARCHITECTURE.md)
- [协议审计](../PROTOCOL-AUDIT.md)
- [XDG 审计](../XDG-AUDIT.md)
- [UI 审计](../UI-AUDIT.md)
- [路线图](../ROADMAP.md)
