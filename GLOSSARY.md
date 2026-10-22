# RPCI IME

COSMIC 桌面上的本地中文输入法；本文件只定义领域词汇，不记录实现或规格。

## Language

**Input session**:
一次从输入法激活到取消或失活的输入上下文。
_Avoid_: Connection, client session

**Composition**:
尚未提交到应用的候选输入状态，包括预编辑文本和候选列表。
_Avoid_: Preedit buffer, pending text

**Candidate**:
当前组合中可被选择并提交的一项文本。
_Avoid_: Suggestion, completion

**Commit**:
把选定文本正式写入应用的行为。
_Avoid_: Send, insert

**Wubi**:
以五笔 86 码表为基础的形码输入方式。
_Avoid_: Shape input, Wubi 98

**Pinyin fallback**:
在五笔没有可用结果时切换到拼音输入的策略，不是独立的拼音模式。
_Avoid_: Pinyin mode, mixed mode

**Mode**:
输入法当前的输入/透传状态，例如中文五笔或英文透传。
_Avoid_: Layout, keyboard state

**Passthrough**:
把按键交给应用处理而不提交中文文本的行为。
_Avoid_: Forward, raw mode

**Panel applet**:
位于 COSMIC 顶部 Panel 的只读输入法状态显示。
_Avoid_: Dock icon, tray applet

**Status state**:
输入法发布给 applet 的简短状态，例如 `中 五` 或 `英`。
_Avoid_: Badge, indicator

**Control request**:
外部组件要求输入法执行的状态切换请求。
_Avoid_: IPC command, signal

**Daily installation**:
通过系统 APT 安装、由系统 autostart 启动的日常运行路径。
_Avoid_: User install, local install

**Developer build**:
从源码构建、用于开发和调试的安装，不是日常运行路径。
_Avoid_: Dev install, test install

**Doctor**:
只读检查日常安装是否健康，并给出修复建议，不自动修改配置。
_Avoid_: Health check, auto-fix

**Upgrade snapshot**:
在包版本变化时创建的用户配置和 RIME 数据快照，用于回滚数据损坏。
_Avoid_: Backup, restore point

**Release gate**:
发布前必须通过的本地验证，当前包括物理键盘 uinput 检查和真实使用观察。
_Avoid_: CI gate, test suite

**Protocol audit**:
对目标 COSMIC 版本实际暴露的 Wayland 输入法能力所做的记录。
_Avoid_: Protocol spec, API list

领域状态入口：[docs/STATUS.md](docs/STATUS.md)。
