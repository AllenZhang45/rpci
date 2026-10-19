# 下一阶段路线图

> 状态：优先级已确认（2026-10-20）；各阶段范围与验收标准为草案。
> 前置：原型已完成，能力与限制见 [SPIKE.md](SPIKE.md)、[ARCHITECTURE.md](ARCHITECTURE.md)。

## 目标

把现在的原型推进到“可安装、可日常使用、可维护”的 COSMIC 中文输入法，同时继续遵守 [PRINCIPLES.md](PRINCIPLES.md) 的四条原则：引擎独立、协议优先、质量优先、应用矩阵验收。

## 当前进展

- 2026-10-20：M1 的安装/卸载脚本、XDG 自启动、配置 CLI 已完成；独立设置 GUI 仍待做。
- 2026-10-20：M2 的备份、导出、导入与保留策略已完成，含自动备份与恢复测试。
- 2026-10-20：M3 以 layer-shell 状态胶囊实现（显示中/英、五笔/拼音）；COSMIC 第三方 panel applet API 未确认，暂不阻塞。
- 2026-10-20：M4 应用矩阵扩展：cosmic-edit/LibreOffice 通过，Firefox/XWayland 未激活；Qt6/Electron/Flatpak 环境无样本，见 [APP-MATRIX.md](APP-MATRIX.md)。
- 2026-10-20：M5 Firefox 证据与 issue 草案已整理，尚未向上游提交，见 [FIREFOX.md](FIREFOX.md)。
- 2026-10-20：第 1 条后续建议完成：release 安装、systemd autostart 单元启动与输入验证；用户重启后自动拉起（PID 1797），见 [INSTALL-VERIFICATION.md](INSTALL-VERIFICATION.md)。

## 优先级总览

1. 安装/自启动与设置集成
2. 词库持久化与备份
3. 面板状态指示
4. 更多应用矩阵
5. Firefox 上游跟进

---

## 1. 安装/自启动与设置集成

### 目标

让用户不需要手动编译、手动启动，就能在 COSMIC 会话里使用输入法，并能完成基本配置。

### 范围

- 安装方式：优先 Pop!_OS/COSMIC 的 `.deb` 或 `install.sh`；不做 Flatpak 版输入法宿主（Flatpak 应用通过合成器接收输入）。
- 自启动：XDG autostart 或 systemd user service，按 COSMIC 会话实际支持选择。
- 单实例：沿用已有的 `Unavailable` 退出保护。
- 设置入口：优先独立设置窗口，覆盖：
  - 中/英默认状态
  - 五笔/拼音模式与兜底开关
  - 候选数量
  - 明暗主题
  - 词库路径与备份
- COSMIC 设置集成：`cosmic-settings` 目前没有稳定的第三方设置页插件接口，需向上游确认；确认前不阻塞独立设置窗口。
- 安装/升级/卸载脚本；卸载时保留用户词库。

### 验收标准

- 新机器一条命令安装，登录 COSMIC 后输入法自动可用。
- 注销/重启后仍然生效，不需要手动启动。
- 设置修改即时生效并持久化。
- 卸载后不残留服务/autostart，用户词库保留。

### 风险

- COSMIC 会话管理/输入源管理仍在演进。
- 打包需要处理 RIME schema 数据与 `librime`/`libclang` 构建依赖。

---

## 2. 词库持久化与备份

### 目标

用户词库不丢、可迁移、可恢复。

### 范围

- 继续使用 XDG 路径（`$XDG_DATA_HOME/rpci/rime`）。
- 单写者：依赖单实例保护，避免多进程写同一 userdb。
- 导出/导入：用户词库 + 配置打包导出，导入后恢复候选与词频。
- 自动备份：启动、退出或定时备份，保留最近 N 份。
- 版本迁移与损坏恢复：schema/userdb 升级失败时可回滚。
- 隐私：默认本地、不上传。

### 验收标准

- 异常退出、升级、重启后词库不丢。
- 导出 → 清空 → 导入，候选与词频恢复。
- 备份损坏时可回滚到上一份。
- 备份文件路径、格式、保留策略有文档。

---

## 3. 面板状态指示

### 目标

在 COSMIC 面板或状态区看到当前输入状态，并能快速切换。

### 范围

- 基于 libcosmic 的 panel applet，或独立状态窗口（取决于 COSMIC panel 的第三方 applet 机制）。
- 显示：中/英、五笔/拼音、当前候选页。
- 交互：点击切换中英/模式；状态与输入法引擎同步。
- 需要定义 applet 与 IME 进程之间的状态通道（IPC 或共享状态）；这会影响进程模型，需单独评审。

### 验收标准

- 状态变化（中英切换、提交、取消）在 100ms 内反映到面板。
- 点击切换后输入法行为立即变化。
- 输入法退出/重启时面板状态不残留错误。

### 风险

- COSMIC panel applet API 的稳定性。
- 如果引入 IPC，需要重新评估单进程架构与延迟。

---

## 4. 更多应用矩阵

### 目标

把“能输入”从单个测试窗口扩展到真实应用组合，并留下可回归的记录。

### 当前结果

- ✅ 合成 `zwp_text_input_v3` 客户端
- ✅ cosmic-edit（文件保存验证）
- ✅ LibreOffice Writer（ODT 保存验证）
- ⚠️ cosmic-term（能激活，提交未进 PTY）
- ❌ Firefox 158（未收到 `Activate`，疑似上游 text-input 版本不匹配）

### 范围

按 [PRINCIPLES.md](PRINCIPLES.md) 的最小矩阵补齐：

- GTK4、Qt6
- Electron/VS Code
- Chromium/Firefox 地址栏
- LibreOffice
- 终端（cosmic-term、foot/kitty 等）
- XWayland 应用
- Flatpak 应用

每个应用记录：版本、激活、候选位置、提交、预编辑、快捷键、已知问题。

### 验收标准

- 每个矩阵应用有明确结论（通过/失败/需上游）。
- 失败项有日志、复现步骤和 workaround。
- 矩阵结果进入文档，作为后续版本回归清单。

---

## 5. Firefox 上游跟进

### 已确认

- Firefox 158 在 COSMIC 下走 Wayland（已加载 `libwayland-client`）。
- COSMIC 只暴露 `zwp_text_input_manager_v3` v1。
- Firefox 测试页聚焦后仍未收到 `Activate`。
- 推测为 Firefox 与 COSMIC 的 text-input 协议版本/实现不匹配，不是 IME 自身能单独修复的问题。

### 行动

1. 收集 COSMIC/Firefox 版本、`wayland-info`、Firefox 启动参数、协议日志。
2. 查 Firefox Bugzilla 是否已有同类报告；没有则提交 issue。
3. 跟进 COSMIC 的 text-input 支持进度。
4. 测试新版本 Firefox/COSMIC 组合。
5. 评估临时方案：XWayland + XIM、Fcitx5 前端、或明确“暂不支持 Firefox”。

### 验收标准

- Firefox 能正常激活并提交中文；或
- 形成有依据的“暂不支持”结论，并给出可用替代方案。

### 依赖

- 上游 Firefox/COSMIC；本项目的 IME 代码不是阻塞点。

---

## 里程碑

| 里程碑 | 内容 | 退出条件 |
|---|---|---|
| M1 | 安装/自启动 + 独立设置窗口最小闭环 | 新机器安装后自动可用 |
| M2 | 词库导出/导入/备份/恢复 | 备份回滚演练通过 |
| M3 | 面板状态指示 | 状态同步与点击切换通过 |
| M4 | 应用矩阵达标 | 最小矩阵有完整结论 |
| M5 | Firefox 结论 | 支持或明确 workaround |

## 非目标

- Windows/macOS、云输入、AI 联想、多语言、皮肤市场。
- Flatpak 版输入法宿主（IME 需在宿主侧运行）。
- 与搜狗五笔 100% 行为一致。

## 风险与依赖

- COSMIC 输入法/面板/设置 API 仍在演进，可能需要上游配合。
- RIME 构建依赖 `librime`/`libclang`，打包需处理。
- “面板状态指示”可能迫使引入 IPC，需重新评审单进程架构。
- 应用矩阵依赖环境里可用的 Qt/Electron/Flatpak 应用。

## 参考

- [SPIKE.md](SPIKE.md)：已有验证记录与应用矩阵
- [ARCHITECTURE.md](ARCHITECTURE.md)：v0.1 原型架构
- [PROTOTYPE.md](PROTOTYPE.md)：原型范围与验收标准
- [PRINCIPLES.md](PRINCIPLES.md)：四条项目原则

## 后续建议

验收后的 6 条具体建议见 [NEXT-STEPS.md](NEXT-STEPS.md)。
