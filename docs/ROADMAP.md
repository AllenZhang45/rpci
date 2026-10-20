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
| 状态指示 | ✅ COSMIC Panel Applet（iced，Dock） |
| Debian 包 | ✅ `.deb` 构建、解包和脚本验证 |
| 架构/审计 | ✅ ADR 0004、协议/XDG/UI 审计 |

## 下一阶段

按优先级排列。每项完成后更新 [STATUS.md](STATUS.md)，不要只改本文件。

### 1. 设置界面收尾

**为什么**：iced 主入口已可用，但仍有明显缺口。

**做什么**：

- 拼音兜底/默认英文模式热加载（当前需重启）
- 鼠标交互与视觉细节验证
- 如需完全 COSMIC 原生控件，评估 libcosmic 迁移

**验收**：候选数量/主题热加载可用；恢复流程已在 GUI 完成；真实 COSMIC 会话目视通过。

### 2. 应用矩阵补齐

**已完成**：

- VS Code 1.142.0（Electron/Chromium）通过，提交「你」
- GNOME Text Editor 51.0（Flatpak/GTK4）通过，提交「你」
- Qt6 6.4.2 示例失败：没有 `Activate`，`wqiy` 原样进入编辑器
- 结果和截图写回 [APP-MATRIX.md](APP-MATRIX.md)

**待做**：

- 排查 Qt6 / QtWayland text-input 不激活
- 独立 Chromium 浏览器样本

**验收**：矩阵各类有明确结论；失败项有日志与 workaround。

### 3. COSMIC Panel Applet

**已完成**：

- 确认第三方 applet 机制：desktop 需要 `X-CosmicApplet=true`，加入 panel `plugins_center`
- iced applet 已嵌入 Dock，显示 `中 五` / `中 拼` / `英`
- IME 通过 `$XDG_RUNTIME_DIR/rpci/state` 暴露状态，applet 每 500ms 轮询
- 点击 applet 打开 iced 设置窗口

**待做**：

- 点击切换中英（当前只打开设置）
- 更完整的 applet 样式/可访问性

**验收**：applet 出现在面板；状态实时同步；点击打开设置（切换待做）。

### 4. XWayland / XIM

**为什么**：xterm 390 在 XWayland 下没有 `Activate`。

**做什么**：

- 评估 XIM 服务端方案，或
- 明确“不支持 XWayland”，给出替代应用建议

**验收**：XWayland 能输入中文；或文档明确不支持并说明原因。

**风险**：XIM 是另一套协议栈，不建议在当前 Wayland 原型内直接实现。

### 5. 打包、发布与升级

**为什么**：本地 `.deb` 已能构建，但还没有仓库、签名和升级/回滚流程。

**已完成**：

- `.deb` 构建：`prototype/packaging/build-deb.sh`
- 包内容：IME、iced/SCTK 设置、系统 autostart、RIME schema
- 验证：`prototype/packaging/test-deb.sh` + 解包后真实输入
- 文档：[PACKAGING.md](PACKAGING.md)

**待做**：

- APT 仓库 / 签名 / 自动更新
- 升级与回滚演练
- 非 amd64 架构

**验收**：一条命令安装/升级；卸载保留用户词库；版本可回滚。

## 暂不执行

- **提交 Firefox 上游 issue**：当前输入法仅个人使用，暂不向上游提交。问题、复现步骤和 issue 草案保留在 [FIREFOX.md](FIREFOX.md)，需要时可直接使用。

## 执行顺序

1. 设置界面收尾
2. 应用矩阵补齐
3. 打包与升级
4. Panel applet（依赖上游）
5. XWayland/XIM（最后评估）

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
