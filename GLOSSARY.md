# 术语表

供 Agent 技能和文档共享的简短词汇。详细状态见 [docs/STATUS.md](docs/STATUS.md)。

- **Engine / `HybridEngine`**：输入状态与五笔/拼音策略；不依赖 Wayland 或 UI。
- **Wubi**：五笔 86；RIME schema `wubi86`。
- **Pinyin fallback**：超过 4 码或没有五笔候选时使用 `pinyin_simp`。
- **Candidate popup**：`input_popup_surface` 上的候选条；不是普通 GUI 窗口。
- **Status applet**：COSMIC Dock 中的 iced applet，显示 `中 五` / `中 拼` / `英`。
- **Control socket**：`$XDG_RUNTIME_DIR/rpci/control.sock`，applet 用它发送 `toggle`。
- **State file**：`$XDG_RUNTIME_DIR/rpci/state`，IME 发布当前状态给 applet。
- **XDG paths**：配置 `~/.config/rpci`、数据 `~/.local/share/rpci`、缓存 `~/.cache/rpci`。
- **Protocol audit**：COSMIC 1.10 上 input-method/text-input/popup 的实际能力记录。

领域文档入口：[docs/README.md](docs/README.md)。
