# COSMIC 原生五笔/拼音输入法原型

这是 COSMIC Wayland 输入法原型的可构建版本，当前用于验证协议链路和输入质量，不是可日常使用的成品。

## 当前能力

- 通过 `zwp_input_method_v2` 被 COSMIC 激活
- `grab_keyboard` 接管键盘，xkbcommon 解析 keycode
- RIME Wubi86 候选，空格/数字选词
- 五笔优先、拼音兜底（超过 4 码或无五笔候选时切到 `pinyin_simp`）
- `commit_string` 上屏
- `input_popup_surface` 候选条，cosmic-text 渲染
- 释放 grab + `zwp_virtual_keyboard_v1` 注入的按键透传：方向键、Ctrl 组合、功能键（F1/Right/Ctrl+C 已验证）

## 目录

- `src/main.rs`：IME 主程序
- `src/bin/textinput.rs`：最小 `zwp_text_input_v3` 测试窗口，打印收到的提交文本
- `src/bin/vkey.rs`：虚拟键盘注入工具（`a` / `abc` / `wubi` / `pinyin` / `preview` / `f1` / `alt-tab`）
- `src/bin/focus.rs`：通过 COSMIC toplevel 协议聚焦窗口
- `src/bin/rimeprobe.rs`：独立 RIME 候选探针
- `scripts/setup-rime.sh`：准备本地 RIME 用户数据

## 依赖

- Rust 工具链（rustup）
- `libwayland-dev`、`libxkbcommon-dev`、`librime-dev`、`librime-data`
- `libclang-dev`（`librime-sys` 的 bindgen 需要）
- `git`（下载 RIME schema）

Debian/Ubuntu：

```bash
sudo apt install libwayland-dev libxkbcommon-dev librime-dev librime-data libclang-dev git
```

如果 `librime-sys` 构建时找不到 libclang，设置：

```bash
export LIBCLANG_PATH=/usr/lib/x86_64-linux-gnu
```

本仓库开发环境曾使用本地解包的 libclang（`/tmp/libclang`），仅作临时方案。

## 准备 RIME 数据

```bash
./prototype/scripts/setup-rime.sh
```

默认写入 `/tmp/rime-user`。可用 `IME_RIME_USER_DIR` / `IME_RIME_SHARED_DIR` 覆盖。

## 构建

```bash
cargo build -p ime-probe
cargo test -p ime-probe
```

## 运行

需要一个 COSMIC Wayland 会话：

```bash
export XDG_RUNTIME_DIR=/run/user/$(id -u)
export WAYLAND_DISPLAY=wayland-1

cargo run -p ime-probe --bin ime-probe
```

启动后聚焦一个文本框；输入 `wqiy` 再按空格应上屏“你”。

测试窗口：

```bash
cargo run -p ime-probe --bin textinput
```

注入按键（另开终端）：

```bash
./target/debug/vkey wubi     # wqiy + space
./target/debug/vkey pinyin   # nihao + space
./target/debug/vkey preview  # wqiy，只出候选不上屏
```

## 已知限制

- 同一 seat 只能有一个 input method 实例；多开会收到 `Unavailable`，RIME userdb 也会锁冲突
- 英文模式、Tab、中文标点、Alt/Super 组合的透传尚未完善
- 候选条在真实应用中的位置、屏幕边缘翻转、HiDPI 尚未验证
- `cosmic-term` 不把 IME 提交写入 PTY，应用矩阵需逐个记录
- 用户词库/配置仍使用 `/tmp/rime-user` 默认路径，尚未接入 XDG 目录
- 无英文模式、无临时拼音键、无设置界面
