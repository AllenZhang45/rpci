# 文档索引

> 先看 [STATUS.md](STATUS.md) 了解当前事实；这里说明每份文档的职责，避免把历史记录当成现状。

## 推荐阅读顺序

1. [STATUS.md](STATUS.md) — 当前能力、验证基线、已知限制
2. [PRINCIPLES.md](PRINCIPLES.md) — 四条项目原则
3. [PROTOTYPE.md](PROTOTYPE.md) — 原型范围与验收标准
4. [ARCHITECTURE.md](ARCHITECTURE.md) — 分层与组件职责
5. [ADR 0004](adr/0004-three-layer.md) — 三层架构约定
6. [ROADMAP.md](ROADMAP.md) — 已完成事项与下一阶段优先级

## 当前事实与验证

| 文档 | 内容 |
|---|---|
| [STATUS.md](STATUS.md) | 当前能力、测试基线、限制 |
| [PROTOCOL-AUDIT.md](PROTOCOL-AUDIT.md) | Wayland input-method 协议与 COSMIC 1.10 实测 |
| [XDG-AUDIT.md](XDG-AUDIT.md) | 配置/数据/缓存/运行时路径 |
| [UI-AUDIT.md](UI-AUDIT.md) | iced 设置、候选窗 HIG、协议约束 |
| [APP-MATRIX.md](APP-MATRIX.md) | 应用兼容性矩阵 |
| [FIREFOX.md](FIREFOX.md) | Firefox 上游问题、复现与 issue 草案 |
| [QT6-AUDIT.md](QT6-AUDIT.md) | Qt6 未激活的反馈回路、根因与上游限制 |
| [XWAYLAND-AUDIT.md](XWAYLAND-AUDIT.md) | XWayland/XIM 缺失证据与不支持决策 |

## 计划

| 文档 | 内容 |
|---|---|
| [ROADMAP.md](ROADMAP.md) | 唯一路线图：完成项、下一步优先级、里程碑、非目标 |

`NEXT-STEPS.md` 已合并进 ROADMAP，不再单独维护。

## 使用指南

| 文档 | 内容 |
|---|---|
| [prototype/README.md](../prototype/README.md) | 构建、运行、安装、设置、测试工具 |
| [SETTINGS-GUI.md](SETTINGS-GUI.md) | 设置窗口（iced 主入口 + SCTK 回退） |
| [PACKAGING.md](PACKAGING.md) | Debian 包构建、验证、安装与卸载 |
| [APT-REPO.md](APT-REPO.md) | 签名 APT 仓库、升级与回滚 |
| [CI.md](CI.md) | GitHub Actions 工作流与本地复现 |
| [SKILLS.md](SKILLS.md) | DSH 工程技能来源、适用条件与全自动适配 |

## 历史记录

历史文档保留过程证据，不代表当前实现，阅读时以 [STATUS.md](STATUS.md) 为准。

| 文档 | 内容 |
|---|---|
| [history/SPIKE.md](history/SPIKE.md) | 2026-10-20 的逐日协议与功能验证日志 |
| [history/INSTALL-VERIFICATION.md](history/INSTALL-VERIFICATION.md) | release 安装、autostart、重启验证记录 |
| [history/assets/](history/assets/) | 旧候选窗/状态截图 |

## 资产

当前截图：

- [assets/applet-panel.png](assets/applet-panel.png) — COSMIC Dock 中的 IME applet
- [assets/popup-visible.png](assets/popup-visible.png) — 候选窗实际显示
- [assets/popup-hig.png](assets/popup-hig.png) — 候选窗 2x buffer
- [assets/app-qt6.png](assets/app-qt6.png) — Qt6 未激活
- [assets/app-vscode.png](assets/app-vscode.png) — VS Code 提交
- [assets/app-flatpak.png](assets/app-flatpak.png) — Flatpak 提交
- [assets/app-chrome.png](assets/app-chrome.png) — Chrome 提交
- [assets/xwayland-xterm.png](assets/xwayland-xterm.png) — XWayland xterm 原始按键
- [assets/settings-iced.png](assets/settings-iced.png) — iced 设置窗口（含词库恢复）
- [assets/settings-gui.png](assets/settings-gui.png) — SCTK 兼容设置窗口

## 维护约定

- 当前事实只写在 STATUS/AUDIT 类文档；SPIKE 只追加历史。
- 路线图只保留一份：ROADMAP。
- 新增文档必须从本索引链接，并更新相关交叉链接。
- 过期截图移入 `history/assets/`，不要继续放在主 assets。
