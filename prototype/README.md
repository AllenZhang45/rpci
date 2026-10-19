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
- 中文标点：`.` → 。、`,` → ，、`?` → ？
- 中英切换：F2 / Ctrl+Space；英文模式下所有按键透传

## 应用矩阵（当前）

| 应用 | 结果 | 验证方式 |
|---|---|---|
| 合成 text-input 客户端 | ✅ | 收到 `COMMIT "你"` |
| cosmic-edit | ✅ | `wqiy` + space 后 Ctrl+S，文件内容为 `你` |
| cosmic-term | ⚠️ | 能激活，但提交未写入 PTY |
| Firefox（Wayland） | ❌ | 页面已聚焦，但未收到 `Activate`，待查 text-input/焦点兼容 |

Firefox 测试页可用 `scripts/firefox-test-server.py` 启动，访问 `http://127.0.0.1:8765/`。

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

默认写入 `$XDG_DATA_HOME/rpci/rime`（通常是 `~/.local/share/rpci/rime`）。可用 `IME_RIME_USER_DIR` / `IME_RIME_SHARED_DIR` 覆盖。

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
- Tab、Alt/Super 组合的透传尚未完善
- 候选条在真实应用中的位置、屏幕边缘翻转、HiDPI 尚未验证
- `cosmic-term` 不把 IME 提交写入 PTY，应用矩阵需逐个记录
- 无临时拼音键、无设置界面
- Firefox 尚未激活 IME，需要继续排查 text-input-v3 兼容性
