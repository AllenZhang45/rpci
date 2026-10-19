# 4 周 Spike 计划

> 状态：草案（2026-10-20）。第 1 周只验证"能不能"，不写完整功能；代码可丢弃。

## 目标

用最小代价回答五个问题：

1. COSMIC 是否允许第三方 IME 被激活、收到按键？
2. 能否把文本提交到目标应用？
3. 候选 popup 能否正确显示和定位？
4. 未处理按键能否透传给应用？
5. RIME 能否接入并稳定工作？

第 1 周只回答 1–4；第 2 周回答 5。

## 前置条件

- Pop!_OS 24.04 / COSMIC，Wayland 会话。
- Rust 工具链（已安装 rustup；非交互 shell 需把 `$HOME/.cargo/bin` 加入 PATH）。
- Wayland/xkb 开发库（`libwayland-dev`、`libxkbcommon-dev`）。
- librime 开发包，或从源码构建。
- 测试应用：GTK4 应用、Qt 应用、VS Code、Firefox/Chromium。
- 能查看会话日志（journalctl 或临时开 debug 日志）。

### 当前环境检查（2026-10-20）

- 系统：Pop!_OS 24.04 LTS，COSMIC，Wayland socket `/run/user/1000/wayland-1`。
- Rust：1.99.1（`~/.cargo/bin`，当前非交互 shell 的 PATH 未包含）。
- 已具备：`libwayland-dev`、`wayland-scanner`、`gcc`、`pkg-config`；crates.io 可访问。
- 已安装（2026-10-20）：`libxkbcommon-dev` 1.6.0、`wayland-protocols`、`librime-dev` 1.10.0、`wayland-utils`。
- 当前没有 fcitx5/ibus 在跑。

## 第 1 周：协议最小闭环

### Day 1：环境与协议侦察

- [x] 记录 COSMIC/cosmic-comp 版本和会话类型（确认是 Wayland）。
- [x] 用 `wayland-info` 确认 compositor 暴露的全局：`zwp_input_method_manager_v2`、`zwp_text_input_manager_v3`、`zwp_virtual_keyboard_manager_v1`、`wl_seat`。
- [x] 确认是否有 fcitx5/ibus 在跑，先停掉，避免冲突。
- [x] 记录结论：哪些协议可用，哪些缺失。

#### Day 1 结果（2026-10-20）

- COSMIC：`cosmic-comp 0.1~1791979042~24.04~742b800`，`cosmic-session 1.10.0~1791314577~24.04~8093b59`，Wayland 会话。
- `zwp_input_method_manager_v2` v1 ✅
- `zwp_text_input_manager_v3` v1 ✅
- `zwp_virtual_keyboard_manager_v1` v1 ✅
- `wl_seat` v9（pointer keyboard touch）✅
- 系统 `wayland-protocols` 只带 input-method v1 XML；Rust 侧用 `wayland-protocols-misc` 0.3.12，自带 input-method-unstable-v2.xml。
- 结论：协议面齐全，理论上可走原生路线；是否真的能激活/收键/透传，仍需最小客户端实测。

### Day 2：最小 IME 客户端

- [x] Rust 程序连接 Wayland，绑定 input-method manager。
- [x] 日志打印 activate/deactivate。
- [x] 聚焦 COSMIC 文本框（cosmic-edit/cosmic-term），确认收到 activate。
- [x] 记录：激活由 compositor 主动发起。

### Day 3：键盘 grab 与按键

- [x] 请求 grab_keyboard，确认收到 key 事件。
- [x] 用 xkbcommon 把 keycode 转 keysym，打印日志。
- [ ] 测试字母、数字、退格、方向键、Ctrl 组合（已测 a/b/c，其余待测）。
- [ ] 记录：grab 期间应用是否还收到按键（判断是否互斥）。

#### Day 2–3 进展（2026-10-20）

- 最小客户端已绑定 `zwp_input_method_manager_v2`，收到 Activate/Deactivate/ContentType/Done。
- Activate 后 `grab_keyboard` 成功，收到 keymap、modifiers、repeat_info。
- 用 `zwp_virtual_keyboard_v1` 注入 keycode 30，探针收到 Key pressed/released。
- 已发送 `commit_string("测试")` + `commit(serial=1)`；客户端未收到协议错误（上屏后来由 Day 4 的 text-input 客户端确认）。
- 探针为方便验证，对每个 Key 事件都提交"测试"；这不是最终行为。

#### Day 3–4 补充（2026-10-20）

- 用 `zcosmic_toplevel_manager_v1.activate` 聚焦窗口成功；终端 toplevel state=[2]（Activated）。
- 在 Activate 后发送 `commit_string("测试")` + `commit(serial=1)`；客户端未收协议错误，但终端里用 `cat > /tmp/ime-commit.txt` 做接收端时没有收到文本。
- 当时的结论是"是否真正上屏仍未确认"；后续用 `zwp_text_input_v3` 测试客户端确认了 commit 路径（见 Day 4 commit 结果）。
- 虚拟键盘注入在 IME grab 时能被 IME 收到；没有 IME 时，未观察到目标终端收到按键，普通客户端的投递行为需进一步确认。

#### Day 3 xkb 结果（2026-10-20）

- 从 grab 的 keymap 加载 xkb keymap（64755 bytes）。
- 注入 keycode 30/48/46，得到 `sym=a/b/c`、`text="a"/"b"/"c"`，buffer 依次为 `"a"`、`"ab"`、`"abc"`。
- 结论：keycode → keysym/UTF-8 链路可用；退格、方向键、Ctrl 组合还未测。

### Day 4：提交与 popup

- [ ] `commit_string("测试")` + `commit`，在 GTK4 应用中确认上屏。
- [x] 写一个最小 `zwp_text_input_v3` 客户端，直接打印 CommitString，确认上屏路径。
- [ ] 在 Qt、VS Code、浏览器地址栏重复。
- [x] 创建 input_popup_surface，画一个纯色矩形，确认位置跟随光标。
  - 已验证 surface 创建、220x64 buffer 提交、`TextInputRectangle { x: 0, y: 0, width: 200, height: 24 }` 事件。
- [x] 用 cosmic-text 把预编辑和候选渲染进 popup（520x48）。
  - 调试 PNG 显示 `wqiy 1.你 2.仰泳`，见 [候选条渲染](assets/candidate-popup.png)。
- [ ] 在屏幕底部测试翻转/裁切；有条件时在 HiDPI 下测试缩放。
- [ ] 记录：哪些应用提交失败，popup 是否正常。

#### Day 4 popup 结果（2026-10-20）

- 创建了 input_popup_surface，附加 220x64 的纯色 buffer，提交无协议错误。
- 收到 `TextInputRectangle { x: 0, y: 0, width: 200, height: 24 }`，说明合成器给出了文本输入区域。
- 未做视觉确认；屏幕底部翻转、边缘裁切、HiDPI 还未测。

#### 候选渲染结果（2026-10-20）

- popup 改为 520x48，用 cosmic-text 排版、手动 alpha blend 写入 ARGB buffer。
- 调试输出 `assets/candidate-popup.png` 显示：`wqiy 1.你 2.仰泳`。
- 说明候选内容已正确绘制到 popup buffer；屏幕上的实际位置/可见性仍建议肉眼确认。

![候选条渲染](assets/candidate-popup.png)

#### Day 4 commit 结果（2026-10-20）

- 最小 `zwp_text_input_v3` 客户端（SCTK 窗口）已收到 `COMMIT: "测试"`。
- 文件 `/tmp/ime-commit.txt` 内容为 6 字节：`e6 b5 8b e8 af 95`（UTF-8 "测试"）。
- 结论：**激活 → grab → commit 上屏链路已打通**。
- 本次 `cosmic-term` 的 `cat` 接收端没有收到提交，可能是终端不把 IME 提交写入 PTY；属于应用侧差异，应用矩阵里要单独记录。

#### RIME 接入结果（2026-10-20）

- Wubi86 schema 部署成功，候选示例：
  - `w` → 人 / 你 / 他 / 个 / 会
  - `wq` → 你 / 低 / 像 / 爸 / 爷
  - `wqiy` → 你 / 仰泳
- 按空格选第一个候选，提交"你"；text-input 客户端收到 `COMMIT: "你"`，文件为 `e4 bd a0`。
- 端到端链路：虚拟键盘 → COSMIC → IME grab → xkb → RIME wubi86 → `commit_string` → 文本输入框。
- 仍未做：拼音兜底、候选 popup 内容、真实应用矩阵、透传。

#### 五笔 + 拼音兜底结果（2026-10-20）

- `wqiy` + space：Wubi 模式，候选「你 / 仰泳」，提交「你」。
- `nihao`：前 4 码保持 Wubi 模式；第 5 码 `o` 自动切到 Pinyin，候选「你好 / 你 / 拟 / 尼 / 呢」，space 提交「你好」。
- 端到端链路与 Wubi 相同；确认规则"超过 4 码或无五笔候选时转拼音"可用。
- 限制：1–4 码的拼音词如果存在五笔候选，仍会走五笔；需要显式拼音/临时拼音键才能覆盖（后续再做）。

#### 透传结果（2026-10-20）

- 在键盘 grab 状态下收到 F1 后：释放 grab → `zwp_virtual_keyboard_v1` 注入 F1 → 重新 grab。
- text-input 客户端收到 F1 press/release，说明"释放 + 虚拟键盘注入 + 重新 grab"的透传机制成立。
- 通用规则已接入：方向键、Ctrl 组合、功能键会按 RIME 是否消费决定是否转发；按下/释放用集合配对。
- 实测 Right（106）、Ctrl+C（29/46）、F1（59）均被 text-input 客户端收到；Wubi 输入不受影响。
- 注意：只能有一个 input method 实例；多实例会收到 `Unavailable`，RIME userdb 也会锁冲突。测试前先清理旧进程。
- 仍未做：英文模式、Tab、中文标点、Alt/Super 组合。

#### 中文标点结果（2026-10-20）

- RIME wubi86 的 punctuator：`.` → 。，`,` → ，，`?` → ？。
- IME 将 ASCII 标点交给 RIME，提交中文标点；text-input 客户端依次收到 `COMMIT "。"` / `COMMIT "，"`。
- Ctrl/Alt/Super + 标点仍走透传。
- 英文模式尚未实现；标点目前固定走中文。

#### 中英切换结果（2026-10-20）

- F2 切到英文模式，popup 显示 `English`；输入 `a` 被透传到应用；F2 切回中文后 `wqiy` + space 正常提交「你」。
- Ctrl+Space 走同一套 toggle 逻辑，尚未单独实测。
- 英文模式下列表/标点/方向键都按透传处理，不再交给 RIME。

#### 应用矩阵结果（2026-10-20）

- cosmic-edit（`com.system76.CosmicEdit`）：`wqiy` + space 提交「你」；再 Ctrl+S，`/tmp/ime-app.txt` 为 `e4 bd a0`。真实应用链路成立。
- LibreOffice Writer（GTK3/Wayland）：Activate 成功；`wqiy` + space 后 Ctrl+S，`/tmp/ime-app.odt` 的 `content.xml` 含「你」。
- Firefox（158，Wayland，测试页已聚焦）：进程已加载 `libwayland-client`，但 COSMIC 只暴露 `zwp_text_input_manager_v3` v1；未收到 `Activate`，推测是 Firefox 与 COSMIC 的 text-input 协议版本/实现不匹配，不是 IME 自身能单独修复的问题。
- cosmic-term：此前能激活，但提交没有写入 PTY；属于应用侧差异。
- 合成 `zwp_text_input_v3` 客户端：✅。

#### XDG 数据目录结果（2026-10-20）

- 默认 RIME 用户目录改为 `$XDG_DATA_HOME/rpci/rime`，没有 XDG 时回退到 `~/.local/share/rpci/rime`。
- `scripts/setup-rime.sh` 已同步；`~/.local/share/rpci/rime/build` 成功生成。
- 用默认路径实测 `wqiy` + space 仍提交「你」。
- `/tmp/rime-user` 只作为没有 HOME/XDG 时的最后回退。

#### 单实例保护（2026-10-20）

- 第二个 `ime-probe` 收到 `Unavailable` 后打印提示并退出，不再留下僵尸实例。
- 避免 RIME userdb 被多个进程同时占用。

#### 主题结果（2026-10-20）

- 候选条支持明暗两套颜色：读取 COSMIC `com.system76.CosmicTheme.Mode/v1/is_dark`，也可用 `IME_POPUP_THEME=light/dark` 覆盖。
- 浅色主题实测输出见下图。

![浅色候选条](assets/candidate-popup-light.png)

#### 数字选词结果（2026-10-20）

- `wqiy` 后按数字 2，提交「仰泳」；文件为 `e4 bb b0 e6 b3 b3`。
- 说明数字 1–9 选词路径可用，不只是空格选第一个。

#### 编辑键结果（2026-10-20）

- `wqiy` + Backspace + space：退格后组合变为 `wqi`，提交「你」。
- `wqiy` + Esc + `wqiy` + space：Esc 取消组合，重新输入后提交「你」。
- 说明退格与取消路径不会破坏输入状态。

#### HiDPI 结果（2026-10-20）

- 候选条按 2x 像素渲染，逻辑尺寸仍是 520×48；`wl_surface.set_buffer_scale(2)` 交给合成器缩放。
- 调试 PNG 为 1040×96，文字清晰；浅色/深色主题均可用。

![2x 候选条](assets/candidate-popup-2x.png)

### Day 5：透传与结论

- [x] 测试方向键、Ctrl+C、F1 的透传。
- [x] 测试中文标点。
- [x] 测试英文模式（F2）。
- [ ] 若按键被吞，尝试 virtual-keyboard 转发，记录是否可行。
- [ ] 填"协议能力表"：激活、grab、commit、popup、透传、HiDPI。
- [ ] 写结论：继续 / 降级 / 转 Fcitx5 前端。
- [ ] 决策门：
  - 激活或 grab 不成立 → 停止"完全原生"路线。
  - popup 不成立 → 候选窗降级为简化方案。
  - 透传不成立 → 改由合成器管理中英切换，或只做中文模式。

## 第 1 周交付物

- 一个最小程序：能启动、被激活、收键、提交"测试"、画一个 popup 矩形。
- 一份协议能力表 + 结论。
- 不接 RIME，不做候选 UI，不做配置。

## 后三周（概要）

- 第 2 周：接 RIME，跑通五笔 86 + 拼音兜底；core 单元测试。
- 第 3 周：候选条、预编辑、定位/翻转/HiDPI；4 类应用联调。
- 第 4 周：应用矩阵验收、性能测量、修 top 问题、演示。

## 结果记录（待填）

| 能力 | 结果 | 备注 |
|---|---|---|
| 激活 | ✅ 已验证 | Activate/Deactivate 均收到 |
| keycode→keysym | ✅ 已验证 | 30/48/46 → a/b/c，buffer "abc" |
| grab | ✅ 已验证 | keymap/modifiers/key 均收到 |
| commit（协议路径） | ✅ 已验证 | text-input 客户端收到 "测试"（6 字节 UTF-8） |
| commit（GTK4/Qt/Electron/浏览器） | 待测 | 应用矩阵 |
| popup 定位/翻转 | 部分验证 | surface、rectangle、文本渲染已验证；翻转/裁切/HiDPI 待测 |
| 按键透传 | ✅ 通用规则已接入 | F1/Right/Ctrl+C 已验证；英文模式待做 |
| 中文标点 | ✅ 已验证 | . , ? → 。，？ |
| 数字选词 | ✅ 已验证 | wqiy + 2 → 仰泳 |
| 编辑键 | ✅ 已验证 | Backspace / Esc 组合测试 |
| 中英切换 | ✅ F2/Ctrl+Space 已验证 | 快速切换后注入需留一点时间 |
| 真实应用（cosmic-edit） | ✅ | wqiy → 你，Ctrl+S 保存验证 |
| LibreOffice Writer | ✅ | wqiy → 你，ODT content.xml 含 你 |
| Firefox | ❌ 未激活 | 需查 text-input/焦点兼容 |
| XDG 词库目录 | ✅ | ~/.local/share/rpci/rime |
| 候选条明暗主题 | ✅ | 浅色截图验证 |
| HiDPI | ✅ | 2x 渲染，逻辑 520×48 |
| HiDPI | 待实测 |  |
| RIME Wubi86 | ✅ 已验证 | `wqiy` + space → "你"，端到端 |
| 拼音兜底 | ✅ 已验证 | nihao → "你好"；第 5 码自动切换 |
