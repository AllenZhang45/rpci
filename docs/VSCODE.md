# VS Code 中文输入

> 状态：2026-10-23。适用于 COSMIC Wayland + RPCI。

## 症状

VS Code 里英文可以输入，但：

- 输入 `wqiy` 不出现候选
- 按 `Ctrl+Space` 没反应
- 应用只收到原始字母

Chrome 和 cosmic-edit 正常。

## 原因

VS Code 默认没有启用 Wayland IME：

1. 可能通过 XWayland/X11 启动，X11 输入路径没有 RPCI 的 text-input。
2. 即使使用 Wayland，Electron 默认也可能不创建 `zwp_text_input_v3`。
3. COSMIC 只暴露 `zwp_text_input_manager_v3`，没有可用的 v1/v2 路径。

官方 VS Code issue：

- https://github.com/microsoft/vscode/issues/237061

其中说明：

- `--enable-wayland-ime` 是 Wayland 下使用 IME 所必需的。
- 默认走 text-input-v1；需要 text-input-v3 时使用 `--wayland-text-input-version=3`。

## 正确启动参数

```text
--ozone-platform=wayland
--enable-wayland-ime
--wayland-text-input-version=3
```

实测命令：

```bash
/usr/share/code/code \
  --ozone-platform=wayland \
  --enable-wayland-ime \
  --wayland-text-input-version=3 \
  --new-window /tmp/ime-vscode-flags.txt
```

结果：

```text
wqiy + space → 你
Ctrl+Space → 英
abc → abc
文件内容：你abc
```

## 永久配置

`~/.vscode/argv.json` **不支持** `--enable-wayland-ime`，不要往里面加。

推荐使用用户级 desktop override：

```bash
mkdir -p ~/.local/share/applications
cp /usr/share/applications/code.desktop ~/.local/share/applications/code.desktop
sed -i 's#^Exec=/usr/share/code/code #Exec=/usr/share/code/code --ozone-platform=wayland --enable-wayland-ime --wayland-text-input-version=3 #' \
  ~/.local/share/applications/code.desktop
update-desktop-database ~/.local/share/applications
```

然后：

1. **完全退出 VS Code**（不是只关窗口，确保主进程结束）
2. 从应用菜单重新打开

如果只是新开一个窗口，VS Code 的 single-instance 机制可能把它交给旧的 X11 进程，参数不会生效。

## 命令行使用

如果从终端启动：

```bash
code --ozone-platform=wayland \
  --enable-wayland-ime \
  --wayland-text-input-version=3
```

或设置 shell alias；但 desktop override 更适合从 Dock/应用菜单启动。

## 边界

- 这不是 RPCI 的协议问题；Chrome 和 cosmic-edit 已证明 text-input-v3 路径可用。
- 不应为 VS Code 在 RPCI 里增加 X11/XWayland 私有 workaround。
- 如果 VS Code 更新后默认原生支持 Wayland IME，可以移除这些参数。
