# 设计与依赖证据审计

> 日期：2026-10-23。目的：用仓库代码、系统探测和上游文档核对关键假设，不把“看起来应该如此”当成事实。

## 方法

- 仓库事实：读取 `prototype/src`、`prototype/packaging`、`docs/`、`Cargo.lock`。
- 系统事实：`wayland-info`、`dpkg-query`、进程和文件检查。
- 上游事实：COSMIC 1.11 Release、COSMIC PR patch、Wayland 协议 XML。
- 行为事实：`test-deb.sh`、`test-doctor.sh`、`test-physical.sh` 的实际输出。

## 已核实

| 事实 | 证据 | 结论 |
|---|---|---|
| COSMIC 1.11 暴露 input-method v2 / text-input v3 / virtual-keyboard v1 | `wayland-info`：`zwp_input_method_manager_v2` v1、`zwp_text_input_manager_v3` v1、`zwp_virtual_keyboard_manager_v1` v1 | 与实现使用的全局一致 |
| text-input-v3 的接口版本为 1 | `/usr/share/wayland-protocols/unstable/text-input/text-input-unstable-v3.xml`：`zwp_text_input_v3 version="1"` | 文档与实现一致 |
| COSMIC 1.11 修复 virtual-keyboard 回送 IME grab | cosmic-comp PR #2925、#2944；#2944 标题为 “Don't send virtual keyboard input back to ime grab” | 官方行为变化，客户端 API 不变 |
| IME 使用标准 virtual-keyboard 路径 | `prototype/src/main.rs` 的 `forward_key` / `forward_tap`；`zwp_virtual_keyboard_manager_v1` | 没有私有协议 |
| 物理键盘需要 release 后 tap | `prototype/src/main.rs:334` 的 `forward_tap`；`test-physical.sh` 输出 `你abc` | 1.11 上仍通过 |
| applet 不接收指针输入 | `prototype/src/bin/applet.rs:58-63` 创建空 input region | 只读实现成立 |
| launcher 在版本变化时创建快照并校验 | `rpci-ime-launch.sh:13-54`；`test-deb.sh` 输出 `upgrade snapshot created`，并断言 tar 含 `default.custom.yaml` | 有实际测试证据 |
| 包和公共 APT 版本不同 | 本机 `dpkg-query` 为 0.1.2；公共 APT 候选 0.1.3；工作区源码 0.1.4 未发布 | “已发布”和“工作区”必须分开描述 |
| 设置窗口依赖 iced 0.14 | `Cargo.lock`：`iced 0.14.0` | 与 README/STATUS 一致 |
| RIME 绑定版本 | `Cargo.lock`：`rime-api 0.12.2`、`librime-sys 0.4.0` | 构建事实 |
| SCTK 版本不唯一 | `Cargo.lock` 同时有 `smithay-client-toolkit 0.19.2`、`0.20.0`、`0.21.1` | 文档不应笼统写“SCTK 0.21” |

## 发现并修正的文档矛盾

- `docs/STATUS.md` 曾写 applet 是“iced 按钮”，但代码已改为 SCTK 只读 applet，且没有点击处理。
- `docs/STATUS.md` 曾写 `.deb` 未签名、公共源未上传；实际 APT 元数据已签名，公共 HTTPS 源已上线。
- `docs/STATUS.md` 曾写 Release “可选签名仓库”；当前 Release workflow 强制要求签名 secret。
- `docs/STATUS.md` 没有记录 `rpci-ime-doctor` 和升级快照。

这些已在 2026-10-23 的 STATUS 更新中修正。

## 尚未证实或仍有缺口

### 1. doctor 版本匹配

已决定并实现：构建时写入 `/usr/share/rpci/version`，doctor 将 `dpkg-query` 版本与它比较；不联网。版本不一致会输出 `WARN` 并返回非零。

测试覆盖：

- 正常版本通过
- 期望版本与包版本不一致时失败

### 2. 1.11 修复后是否还需要 tap-on-release

PR #2944 让 virtual-keyboard 事件绕过 IME grab；理论上可能允许即时转发。

决定：**v0.1.4 保留 tap-on-release，不做 A/B，不删除**。原因是 1.11 上物理键盘测试通过，而即时转发没有同等证据；A/B 作为 P2 后续实验。

### 3. 发布版本与工作区版本分开记录

决定：README 标注公共 `0.1.3`，STATUS 同时列出工作区 `0.1.4` 未发布；安装命令继续指向公共版本。

### 4. 3 天观察期尚未开始

当前源码版本 0.1.4 未发布，候选包也未安装到系统；`docs/STABILITY.md` 的观察记录仍是“未开始”。

### 5. cosmic-osk 仲裁仍未由上游确认

1.11 的 OSK 激活改走私有 D-Bus，但没有官方文档说明第三方 `zwp_input_method_v2` 的长期仲裁策略。当前只是“本会话未观察到冲突”。

### 6. cosmic-term 1.11 行为未复测

`docs/APP-MATRIX.md` 仍引用 1.10 的“提交未进 PTY”，并标注 1.11 待复测。

## 上游链接

- COSMIC 1.11 Release：https://github.com/pop-os/cosmic-epoch/releases/tag/epoch-1.11.0
- cosmic-comp #2925：https://github.com/pop-os/cosmic-comp/pull/2925
- cosmic-comp #2944：https://github.com/pop-os/cosmic-comp/pull/2944
- cosmic-comp #2928：https://github.com/pop-os/cosmic-comp/pull/2928
- wayland-protocols text-input-v3：https://gitlab.freedesktop.org/wayland/wayland-protocols/-/blob/main/unstable/text-input/text-input-unstable-v3.xml

## 结论

当前实现与 1.11 的协议全局、virtual-keyboard 行为、applet 只读设计和升级快照都有代码或测试证据。

已决定：

1. doctor 使用包内 `/usr/share/rpci/version` 做离线版本匹配；
2. v0.1.4 保留 tap-on-release，A/B 延后；
3. README/STATUS 分开记录公共版本和工作区版本；
4. cosmic-term/OSK 继续作为未证实 P2，不阻塞 v0.1.4。

证据审计只在新增 API/依赖、协议版本变化或发布版本变化时要求，不用于每次小改。
