# ADR 0005：日常安装走系统 APT，Panel applet 只读

> 状态：Accepted（2026-10-23）
> 适用版本：COSMIC Epoch 1.11 / Pop!_OS 24.04
> 关联：[COSMIC-1.11-AUDIT.md](../COSMIC-1.11-AUDIT.md)、[SKILLS.md](../SKILLS.md)

## 背景

项目曾同时存在系统 deb、用户级 `~/.local/bin` 和用户级 autostart。两个实例、两个版本和两套 applet 配置并存时，很难判断当前运行的是哪一份，也曾把焦点问题误判为输入法协议问题。

Panel applet 曾经支持点击切换中英，但点击顶部 Panel 会让面板取得键盘焦点，编辑器失去焦点。COSMIC 1.11 升级还会重置 Panel 配置，使 applet 消失。

## 决策

1. **日常安装只走系统 APT**：`rpci-ime` deb + 系统 autostart。源码构建保留为开发者模式，不作为日常安装路径。
2. 系统包 launcher 检测到用户级 autostart 时，只把它重命名为 `.disabled-by-system-package`；不删除用户配置、RIME 数据或二进制。
3. **Panel applet 只是顶部 Panel 的只读状态显示**：显示 `中 五` / `英`，不处理点击，不提供齿轮按钮。
4. 中英切换只通过 `Ctrl+Space`；不使用 F2，也不依赖 applet 点击。
5. 发布前使用本地 `/dev/uinput` 脚本验证物理键盘；该脚本不进入 CI，也不打包。

## 后果

- 用户不能通过点击 applet 切换中英；这是为了避免焦点被抢走。
- 系统包和用户级安装同时存在时，launcher 会主动停用用户级 autostart，但保留数据。
- applet 位置仍由 launcher 维护；COSMIC 大版本升级后可能需要重新登录以重新注册。
- 开发者仍可用源码安装和 `~/.local/bin` 调试，但需要自行承担双实例风险，或先停用用户级 autostart。
