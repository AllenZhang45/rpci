# 架构（v0.1 原型）

> 状态：已确认（2026-10-20）。与 [PRINCIPLES.md](PRINCIPLES.md)、[PROTOTYPE.md](PROTOTYPE.md) 配套；范围变更需更新本文件并提交。
> 三层架构约定见 [ADR 0004](adr/0004-three-layer.md)；协议/XDG/UI 审计见 [PROTOCOL-AUDIT.md](PROTOCOL-AUDIT.md)、[XDG-AUDIT.md](XDG-AUDIT.md)、[UI-AUDIT.md](UI-AUDIT.md)。

## 1. 目标与约束

- 4 周原型，Pop!_OS 24.04（COSMIC），五笔 86 + 拼音兜底。
- 单进程、Rust，RIME 作为起步引擎。
- 引擎不依赖 COSMIC/UI；协议层可替换。
- 先证明协议链路，再谈体验完整度。

## 2. 总览

```
cosmic-comp
   │ input-method-v2 / input_popup_surface
   ▼
wayland-frontend ── candidate-ui
   │
   ▼
ime-core (Engine trait)
   │
   ▼
rime-backend (librime)
```

- `wayland-frontend`：协议适配，保持薄。
- `candidate-ui`：只负责画候选条。
- `ime-core`：输入状态、混合策略、候选模型。
- `rime-backend`：RIME 会话、schema、词库。

## 3. 组件职责

### 3.1 ime-core

- 纯 Rust，不依赖 Wayland、iced/libcosmic。
- `Engine` trait：`handle_key`、`candidates`、`commit`、`reset`、`set_mode`。
- 输入状态机：五笔 buffer、拼音 buffer、中/英模式、候选选择。
- 混合策略：先五笔；超过 4 码或无候选时转拼音；候选合并与排序。
- 用户学习接口：由 backend 提供，core 不直接碰存储。

### 3.2 rime-backend

- librime FFI；负责部署、schema、会话、userdb。
- 先支持 Wubi86 与 Pinyin 混输配置。
- 原型优先用 RIME 现有 schema 表达混输；表达不了再由 core 组合两次查询，或换 `HybridEngine`。
- 词库放 XDG data dir 下的专用目录，单写者锁。

### 3.3 wayland-frontend

- input-method-v2 客户端；绑定 wl_seat、keymap、input_method、popup surface。
- 处理 activate/deactivate、grab_keyboard、按键、preedit、commit、focus。
- 只做协议转换；不在这里写五笔/拼音规则。
- 协议版本先 v2；实验性协议只在 COSMIC 不支持 v2 时适配。

### 3.4 candidate-ui

- 画进 compositor 提供的 input_popup_surface，不自己开 layer-shell 窗口。
- raw wl_surface + wl_shm + cosmic-text 软件渲染。
- 已做 2x HiDPI、明暗主题、COSMIC HIG 圆角/颜色/间距。
- 不用 iced：iced/winit 不暴露 raw wl_surface；`iced_layershell` 也不是 popup。

### 3.5 cosmic-integration

- 自启动：systemd XDG autostart generator。
- 设置：iced 0.14 独立窗口；保留自绘 SCTK 窗口作为兼容回退。
- 面板指示器：layer-shell 胶囊；真正的 panel applet 待上游。
- 输入源注册：未做。

## 4. 关键决策

### 4.1 引擎：先 RIME-only

- 原型用 RIME 单会话 + Wubi86/Pinyin 混输配置，验证"五笔优先、无候选转拼音"能否表达。
- `Engine` trait 把行为与实现分开；表达不了再加 `HybridEngine`，不改前端。
- 风险：RIME schema 可能无法精确表达"无五笔候选才转拼音"；spike 先验证。
- 若改用双会话/双引擎，需要额外处理词库冲突与延迟。

### 4.2 激活：常驻 + 聚焦 + 内部中英

- 目标行为：IME 常驻；文本框聚焦时由合成器激活；IME 内部维护中/英模式。
- 协议风险：英文模式/快捷键需要把未处理按键透传给应用；Wayland 下没有全局钩子，可能需要 virtual-keyboard 或合成器侧转发。spike 必须验证。
- 若 COSMIC 不支持透传：退化为合成器管理的输入源切换，或只做中文模式，或转 Fcitx5 前端。

### 4.3 候选窗：raw popup + 软件渲染

- 理由：input_popup_surface 由合成器定位；layer-shell 不跟随光标，iced 不能直接画。
- 软件渲染对候选条足够；先不做 GPU/动画。

### 4.4 回退触发：超过 4 码或无五笔候选

- 4 码以内优先五笔，减少短码误判。
- 超过 4 码直接按拼音。
- 无五笔候选时按拼音（含 1–4 码）。
- 后续用真实输入语料调阈值。

### 4.5 进程模型：单进程

- 引擎做成库 crate，不做 daemon；降低 IPC/生命周期复杂度。
- 若将来 Fcitx5 前端也要用同一引擎，再评估拆服务。

### 4.6 词库与配置

- RIME userdb 放 XDG data dir；单写者锁；导出/导入后置。
- 原型配置用简单文件；设置页后置。

### 4.7 协议版本

- 先 input-method-v2；前端隔离协议细节。
- 实验性协议只在 v2 不可用时适配。

## 5. 数据流

1. 用户按键 → compositor → input-method-v2 → frontend。
2. frontend 用 xkbcommon 转 keysym，送 core。
3. core 更新 buffer；先查五笔；满足条件查拼音；生成候选。
4. frontend 把 preedit/候选送 candidate-ui；popup surface 由合成器定位。
5. 选词 → core 产出 commit 文本 → frontend 调 commit_string/commit。
6. 未处理按键 → 透传回应用（机制待验证）。

## 6. 风险与验证点

- 激活/grab：COSMIC 是否允许第三方 IME 被激活并收键。
- popup：跟随光标、边缘翻转、HiDPI。
- commit：4 类应用（GTK4、Qt6、Electron、浏览器）正常上屏。
- 透传：英文/快捷键/方向键不被吞。
- RIME 表达力：schema 能否实现回退规则。
- 性能：p99 延迟、内存、启动时间。

具体第 1 周验证清单见 [SPIKE.md](SPIKE.md)。

## 7. 测试策略

- core：单元测试覆盖回退规则、候选合并、模式切换。
- backend：RIME 会话的集成测试（部署、userdb、重启不丢）。
- 协议：spike + 手动应用矩阵；4 周内不做完整自动化。
- 记录每次验收的应用/版本/结果。

## 8. 非目标

- 设置页、面板 applet、Flatpak 打包、云候选/AI、皮肤系统、多语言、完整安装器。
- 跨发行版支持。
- 与搜狗五笔 100% 行为一致。

## 9. 决策记录

- 2026-10-20：原型采用 RIME-only、单进程、常驻 + 聚焦激活、内部中英切换、raw popup 软件渲染；回退规则为"超过 4 码或无五笔候选"。
- 待 spike 验证：COSMIC 激活/grab、popup、按键透传、RIME schema 表达力。
