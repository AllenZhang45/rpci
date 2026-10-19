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
- 候选条明暗主题：读 COSMIC `is_dark` 配置，也可用 `IME_POPUP_THEME=light/dark` 覆盖
- 候选条按 2x 像素渲染（逻辑 520×48），HiDPI 输出更清晰
- 状态胶囊：layer-shell 顶层小窗，显示 `中 五` / `中 拼` / `英`
- 中英切换：F2 / Ctrl+Space；英文模式下所有按键透传

## 应用矩阵（当前）

| 应用 | 结果 | 验证方式 |
|---|---|---|
| 合成 text-input 客户端 | ✅ | 收到 `COMMIT "你"` |
| cosmic-edit | ✅ | `wqiy` + space 后 Ctrl+S，文件内容为 `你` |
| LibreOffice Writer | ✅ | `wqiy` + space 后 Ctrl+S，ODT 的 `content.xml` 含 `你` |
| cosmic-term | ⚠️ | 能激活，但提交未写入 PTY |
| Firefox（Wayland） | ❌ | 页面已聚焦，但未收到 `Activate`，待查 text-input/焦点兼容 |

Firefox 测试页可用 `scripts/firefox-test-server.py` 启动，访问 `http://127.0.0.1:8765/`。

## 目录

- `src/main.rs`：Wayland 协议、状态与主循环
- `src/engine.rs`：五笔/拼音策略、RIME 会话、按键透传判定
- `src/popup.rs`：候选条渲染（cosmic-text + wl_shm）
- `src/config.rs`：配置读写与校验
- `src/status.rs`：layer-shell 状态胶囊
- `src/settings_model.rs`：设置窗口状态机
- `src/bin/ime-settings-gui.rs`：独立设置窗口
- `src/bin/ime-settings.rs`：设置 CLI
- `src/bin/textinput.rs`：最小 `zwp_text_input_v3` 测试窗口，打印收到的提交文本
- `src/bin/vkey.rs`：虚拟键盘注入工具（`a` / `abc` / `wubi` / `pinyin` / `preview` / `f1` / `alt-tab`）
- `src/bin/focus.rs`：通过 COSMIC toplevel 协议聚焦窗口
- `src/bin/rimeprobe.rs`：独立 RIME 候选探针
- `scripts/setup-rime.sh`：准备本地 RIME 用户数据
- `scripts/install.sh` / `uninstall.sh`：安装、自启动、卸载

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

## 安装与自启动

```bash
./prototype/scripts/install.sh
```

默认安装到 `~/.local/bin`，并写入 XDG autostart：

```
~/.config/autostart/rpci-ime.desktop
```

卸载：

```bash
./prototype/scripts/uninstall.sh           # 保留配置和词库
./prototype/scripts/uninstall.sh --purge   # 全部删除
```

开发时可用 `IME_PROFILE=debug IME_SKIP_BUILD=1` 复用已有 debug 构建。

安装后验证：

```bash
./prototype/scripts/verify-install.sh
```

autostart 由 systemd 的 XDG autostart generator 接管；验证细节见 [docs/INSTALL-VERIFICATION.md](../docs/INSTALL-VERIFICATION.md)。

## 设置

独立设置窗口（iced）：

```bash
cargo run -p rpci-settings-iced
# 安装后可从应用菜单打开 "COSMIC IME 设置"
```

- iced 0.14 窗口，覆盖默认中英、拼音兜底、候选数量、主题、词库备份
- 兼容回退：`ime-settings-gui`（自绘、键盘操作）
- 保存后需要重启输入法生效
- 详见 [docs/SETTINGS-GUI.md](../docs/SETTINGS-GUI.md)

CLI：

```bash
ime-settings show
ime-settings set candidate_count 5
ime-settings set theme light
ime-settings set pinyin_fallback off
ime-settings get theme
ime-settings reset
```

配置文件位于 `$XDG_CONFIG_HOME/rpci/config.toml`。修改在下次启动输入法时生效。

| key | 取值 |
|---|---|
| `ascii_mode_default` | `true` / `false` |
| `pinyin_fallback` | `true` / `false` |
| `candidate_count` | `1`–`9` |
| `theme` | `auto` / `dark` / `light` |

## 词库备份与迁移

```bash
ime-settings backup              # 备份到 $XDG_DATA_HOME/rpci/backups/<时间戳>
ime-settings backups             # 列出备份
ime-settings export <dir>        # 导出到指定目录
ime-settings import <dir>        # 从目录恢复
```

- 导出结构：`rime/`、`config.toml`、`manifest.txt`
- 自动跳过 `build/`、`log/`、`sync/`、`LOCK`、`*.lock`
- 备份保留最近 5 份
- 导入前会自动备份当前状态
- 建议先退出输入法再备份/导出，避免 LevelDB 仍在写入

## 已知限制

- 同一 seat 只能有一个 input method 实例；第二个实例会收到 `Unavailable` 并直接退出
- Tab、Alt/Super 组合的透传尚未完善
- 候选条屏幕边缘翻转尚未验证（HiDPI 已按 2x 渲染）
- `cosmic-term` 不把 IME 提交写入 PTY，应用矩阵需逐个记录
- iced 设置窗口已验证启动；候选窗因 input_popup_surface 协议限制仍是自绘；保存后需重启输入法
- 状态指示是 layer-shell 胶囊，不是 COSMIC panel applet；第三方 applet API 尚未确认
- Firefox 尚未激活 IME，需要继续排查 text-input-v3 兼容性

## 路线图

下一阶段优先级见 [docs/ROADMAP.md](../docs/ROADMAP.md)。

应用兼容性见 [docs/APP-MATRIX.md](../docs/APP-MATRIX.md)，Firefox 问题见 [docs/FIREFOX.md](../docs/FIREFOX.md)。

架构约定见 [ADR 0004](../docs/adr/0004-three-layer.md)；协议/XDG/UI 审计见 [PROTOCOL-AUDIT](../docs/PROTOCOL-AUDIT.md)、[XDG-AUDIT](../docs/XDG-AUDIT.md)、[UI-AUDIT](../docs/UI-AUDIT.md)。

后续 6 条建议见 [docs/NEXT-STEPS.md](../docs/NEXT-STEPS.md)。
