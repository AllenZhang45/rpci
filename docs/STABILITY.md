# 稳定性发布门槛

> 适用于 v0.1.4。目标版本：COSMIC Epoch 1.11 / Pop!_OS 24.04。

## 严重级别

### P0

- 完全无法输入中文或英文
- IME 崩溃、反复重启
- 焦点丢失、状态错乱、双实例
- 用户词库/配置丢失或损坏
- 系统包无法启动

### P1

- 日常应用中可稳定复现的错误上屏
- `Ctrl+Space` 失效，或切换后丢键
- applet 状态长期错误
- 候选窗明显错位、遮挡
- 有 workaround，但每天都会遇到

### P2

- 单个非日常应用的问题
- 纯视觉/间距/动画问题
- 上游已确认、协议层无法修的问题
- 偶发且无法复现的问题

只有 P0/P1 阻塞发布；P2 记录但不修。

## 日常验证应用

发布门槛只覆盖：

- cosmic-edit
- Chrome
- VS Code
- LibreOffice

以下明确不阻塞：

- Firefox
- XWayland 应用
- cosmic-term
- Qt 6.4.2 应用

## 3 天观察期

- 定义为连续 3 个自然日有真实日常使用。
- 观察期内只允许 P0/P1 修复和文档改动，不写新功能。
- 任何 P0/P1 修复都会让观察期从当天重新计时。
- P0/P1 记录在本文件，不创建外部 issue。

## 发布检查清单

- [ ] `prototype/scripts/test-physical.sh` 通过
- [ ] `prototype/scripts/test-doctor.sh` 通过
- [ ] `rpci-ime-doctor` 对真实会话通过
- [ ] `cargo test -p ime-probe` 通过
- [ ] `cargo test -p rpci-settings-iced` 通过
- [ ] `prototype/packaging/test-deb.sh` 通过
- [ ] `prototype/packaging/test-repo.sh` 通过
- [ ] 连续 3 天没有新的 P0/P1
- [ ] 公共 APT 的候选版本与 Release 资产一致

## 观察记录

| 日期 | 版本 | 级别 | 应用 | 现象 | 状态 |
|---|---|---|---|---|---|
| 2026-10-23 | 0.1.3 | - | - | 观察期开始前，等待 0.1.4 候选版本 | 未开始 |

## 停止扩展

满足任一条件后停止功能扩展，只做安全/上游兼容修复：

- COSMIC 原生输入法已经满足五笔/拼音需求
- 上游协议变化导致维护成本持续高于日常收益
- 连续一个月没有真实使用需求
- 公共 APT/发布流程维护成本超过输入法本身
