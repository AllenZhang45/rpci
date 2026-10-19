# XDG 路径审计

> 日期：2026-10-20。对应 [ADR 0004](../adr/0004-three-layer.md) 第二层。

## 目录职责

| 内容 | 路径 | 实现 |
|---|---|---|
| 用户配置 | `${XDG_CONFIG_HOME:-~/.config}/rpci/config.toml` | `paths::config_path()` |
| RIME 用户数据 | `${XDG_DATA_HOME:-~/.local/share}/rpci/rime` | `paths::rime_user_dir()` |
| 词库备份 | `${XDG_DATA_HOME:-~/.local/share}/rpci/backups` | `paths::backups_dir()` |
| RIME build 缓存 | `${XDG_CACHE_HOME:-~/.cache}/rpci/rime-build` | `setup-rime.sh` 创建 symlink |
| 运行时目录 | `$XDG_RUNTIME_DIR/rpci` | `paths::app_runtime_dir()`，当前保留 |
| autostart | `${XDG_CONFIG_HOME:-~/.config}/autostart/rpci-ime.desktop` | `install.sh` |
| 应用启动器 | `${XDG_DATA_HOME:-~/.local/share}/applications/rpci-settings*.desktop` | `install.sh` |

RIME 的 `BUILD_DIR` 固定在其用户目录下，因此使用：

```
$XDG_DATA_HOME/rpci/rime/build -> $XDG_CACHE_HOME/rpci/rime-build
```

备份逻辑会跳过 `build`，不会把缓存打进备份。

## 路径解析规则

`prototype/src/paths.rs` 统一处理：

- 环境变量为绝对路径：使用该值。
- 环境变量为相对路径：按 XDG 规范视为无效，回退默认目录。
- 环境变量未设置：
  - 配置：`~/.config`
  - 数据：`~/.local/share`
  - 缓存：`~/.cache`
  - 运行时：无默认值；未设置或相对路径返回 `None`。
- `IME_RIME_USER_DIR` 是显式覆盖，仍优先于 XDG 默认值。
- 无 `HOME` 时，数据/缓存路径可能返回 `None`；调用方需有兜底（RIME 回退 `/tmp`，备份回退 `/tmp/rpci/backups`）。

## 代码验证

新增 `paths` 单元测试：

- 绝对 XDG 值优先；
- 相对 XDG 值被忽略并回退；
- 未设置时使用默认；
- `XDG_RUNTIME_DIR` 只接受绝对路径。

运行：

```bash
cargo test -p ime-probe
```

当前：34 个测试通过。

## 实测路径

```bash
~/.config/rpci/config.toml
~/.local/share/rpci/rime/
~/.local/share/rpci/backups/
~/.cache/rpci/rime-build/
~/.config/autostart/rpci-ime.desktop
~/.local/share/applications/rpci-settings.desktop
```

安装脚本也加入了相对路径回退，避免 `XDG_CONFIG_HOME=relative` 污染当前目录。

## 未完成

- 运行时目录目前没有实际文件；如果未来增加 IPC/单实例锁，应放在 `$XDG_RUNTIME_DIR/rpci`，不要放 `/tmp`。
- 未实现 XDG 用户目录（`XDG_DOCUMENTS_DIR` 等）的导入/导出对话框。
- RIME 的 `installation.yaml` 仍可能包含绝对路径；迁移时需要单独处理。
