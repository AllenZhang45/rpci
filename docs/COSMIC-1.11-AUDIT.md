# COSMIC Epoch 1.11 兼容性审计

> 日期：2026-10-23。基线：Pop!_OS 24.04 / COSMIC Epoch 1.11.0 / rpci-ime 0.1.2。

## 1.11 中与本项目相关的变更

上游 Release：

- https://github.com/pop-os/cosmic-epoch/releases/tag/epoch-1.11.0

相关项：

- `cosmic-comp` #2925：更新 Smithay 的 virtual-keyboard API，并新增 virtual keyboard keymap 处理。
- `cosmic-comp` #2944：输入法的 virtual-keyboard 事件不再回送给 IME grab。
- `cosmic-comp` #2928 / `cosmic-osk` #55：OSK 激活改为私有 D-Bus signal。
- `libcosmic` #1474：文本输入不再捕获 Escape。
- `cosmic-comp`：xdg-dialog-v1、无输出窗口清理等，与本项目无直接关系。

参考 PR：

- https://github.com/pop-os/cosmic-comp/pull/2925
- https://github.com/pop-os/cosmic-comp/pull/2944
- https://github.com/pop-os/cosmic-comp/pull/2928

## 实际测试

环境：

```
cosmic-comp     0.1~1792508925~24.04~8cb1802
cosmic-panel    1.11.0~1792526322~24.04~ca20ce3
cosmic-session  1.11.0~1792525615~24.04~4b0a0a2
cosmic-edit     1.11.0~1792525544~24.04~4ac1cc8
rpci-ime        0.1.2
```

结果：

- 中文输入：`wqiy` + space → 「你」✅
- `Ctrl+Space` → 英 ✅
- 英文输入：物理键盘（uinput 模拟）`abc` ✅
- 文件保存：`你abc` ✅
- 顶部 Panel applet 重新加入后正常显示 ✅
- iced 设置窗口启动正常 ✅
- 未观察到 cosmic-osk D-Bus 激活对 IME 的干扰

## 发现的问题

### Panel 配置被 1.11 更新重置

升级后 `~/.config/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings`
不再包含 applet，导致 applet 进程没有启动。

原因不是 applet 协议失效；重新加入后可以正常启动。

已修复启动器逻辑：

- 只检查 **active** 的 `plugins_center` / `plugins_wings`
- 不再把 `.bak` 文件当成已配置
- 优先放入顶部 Panel 右侧 `plugins_wings`
- 如果旧位置（Dock/center）有 applet，会移动到顶部 Panel

对应文件：

```
prototype/packaging/rpci-ime-launch.sh
```

升级 COSMIC 后如果 applet 仍不出现，注销再登录，或重新运行安装/升级 launcher。

### 是否需要修改 IME 协议代码

目前不需要。

- `zwp_input_method_v2` / `zwp_text_input_v3` / `zwp_input_popup_surface_v2` 使用方式不变。
- virtual-keyboard 的修复对客户端 API 透明。
- 0.1.2 的物理键盘 tap 方案在 1.11 下仍然正常工作。
- 没有证据要求改用新的 private D-Bus OSK 接口；本 IME 不实现 OSK。

## 需要随 1.11 发布的修改

1. launcher 的 Panel 配置兼容修复（本审计包含）。
2. README / STATUS / PROTOCOL-AUDIT 的版本基线更新到 1.11。
3. 发布新的 patch 版本，确保系统安装从 0.1.2 升级到包含 launcher 修复的版本。
4. 升级后重新登录，让 autostart 重新注册 applet。

## 后续观察

- 如果 cosmic-comp 的 virtual-keyboard 修复使 tap 延迟不再必要，可以再尝试改回即时转发；当前 tap 在 1.11 下稳定，暂不冒险。
- 继续观察 cosmic-term 的 IME 提交行为是否在 1.11 改善。
- 升级后确认只有一个 IME 实例，且 applet 位于顶部 Panel。
