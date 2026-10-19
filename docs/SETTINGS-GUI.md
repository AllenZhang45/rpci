# 独立设置 GUI

> 状态：2026-10-20。对应 [NEXT-STEPS.md](NEXT-STEPS.md) 第 2 条。

![设置窗口](assets/settings-gui.png)

## 启动

安装后应用菜单会出现 **COSMIC IME 设置**：

```bash
./prototype/scripts/install.sh
```

也可以直接运行：

```bash
./target/debug/ime-settings-gui
# 或安装后
~/.local/bin/ime-settings-gui
```

启动器文件：

```
~/.local/share/applications/rpci-settings.desktop
```

## 操作

| 按键 | 作用 |
|---|---|
| ↑ / ↓ | 选择设置项 |
| ← / → | 修改当前值 |
| Enter | 切换当前值 |
| S | 保存到 `config.toml` |
| B | 立即备份词库 |
| R | 重新加载配置 |
| Esc | 关闭窗口 |

当前是**键盘驱动**的窗口，鼠标点击尚未实现。

## 覆盖的设置

| 界面项 | 配置键 | 取值 |
|---|---|---|
| 默认英文模式 | `ascii_mode_default` | 开 / 关 |
| 拼音兜底 | `pinyin_fallback` | 开 / 关 |
| 候选数量 | `candidate_count` | 1–9 |
| 主题 | `theme` | auto / dark / light |

保存位置：

```
$XDG_CONFIG_HOME/rpci/config.toml
```

修改保存后需要重启输入法才会生效；窗口状态会提示这一点。

## 实现方式

- 一个独立的 Wayland 窗口，使用 SCTK + `wl_shm` + cosmic-text 软件渲染。
- 不引入 iced/libcosmic/GTK 依赖，避免当前环境的 GPU/GUI 工具链风险。
- 复用 `Config` 与 `backup` 模块；保存使用 `Config::save` 的临时文件 + rename。
- `B` 键调用 `backup_now`，保留最近 5 份，备份内容与 CLI 一致。

## 验证

自动化：

```
cargo test -p ime-probe
32 passed; 0 failed
```

端到端：

1. 用临时 `XDG_CONFIG_HOME` 启动 GUI，初始 `pinyin_fallback = true`。
2. 聚焦窗口，注入 Down、Right、S。
3. 配置文件变为 `pinyin_fallback = false`，窗口状态变为“配置已保存（重启输入法后生效）”。
4. 再执行两次 Down/Right/S，配置变为 `candidate_count = 7`、`theme = "dark"`。
5. 按 B，生成 `backups/<时间戳>/`，包含 `config.toml`、`manifest.txt`、`rime/user.yaml`。

截图中的窗口显示候选数量 7、主题 dark、状态“配置已保存”。

## 限制

- 仅键盘操作，鼠标点击/拖动未实现。
- 保存后不会热加载；需要重启输入法（或重新登录）。
- 样式是自绘的深色窗口，不是 libcosmic 原生控件。
- 备份时建议先停止输入法，避免 LevelDB 正在写入。
- 窗口固定最小尺寸 560×380，未做 HiDPI 2x 渲染。

## 后续

- 增加鼠标点击与悬停。
- 用 libcosmic/iced 重做视觉，或接入 COSMIC 设置页。
- 增加“保存并重启输入法”按钮。
- 支持词库恢复（当前 GUI 只做备份，导入仍用 CLI）。
