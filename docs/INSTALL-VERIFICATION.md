# Release 安装与自启动验证

> 日期：2026-10-20。对应 [NEXT-STEPS.md](NEXT-STEPS.md) 第 1 条。

## 结果

- 默认 **release** 构建完成，并已安装到真实 `$HOME`。
- 安装后的二进制实际输入通过：`wqiy` + space 提交「你」（`e4 bd a0`）。
- XDG autostart 文件被 `systemd-xdg-autostart-generator` 识别，生成
  `app-rpci\x2dime@autostart.service`。
- `xdg-desktop-autostart.target` 已 active，单元已链接进该 target。
- 手动 stop → start 该单元后，输入法自动拉起并正常提交；这验证了 autostart 的实际启动路径。
- 用户已于 2026-10-20 07:02 重启，autostart 自动拉起成功；详见下文“重启后实测”。

## Release 构建

```bash
export LIBCLANG_PATH=/tmp/libclang/usr/lib/x86_64-linux-gnu
export BINDGEN_EXTRA_CLANG_ARGS="-I/tmp/libclang/usr/lib/llvm-18/lib/clang/18/include"
cargo build --release -p ime-probe --bin ime-probe --bin ime-settings
```

产物：

| 文件 | 大小 | BuildID |
|---|---|---|
| `target/release/ime-probe` | 5.0M | `28f8ef8a100fbaa6fbde753f7d4fa3fb5a20368e` |
| `target/release/ime-settings` | 955K | `7ab2235f6dc6b7775db438c0c091ad595f75daa5` |

## 安装

```bash
IME_PROFILE=release IME_SKIP_BUILD=1 ./prototype/scripts/install.sh
```

真实 HOME 下生成：

```
~/.local/bin/ime-probe
~/.local/bin/ime-settings
~/.config/autostart/rpci-ime.desktop
~/.config/rpci/config.toml
~/.local/share/rpci/rime/
```

安装脚本在写完 autostart 后会执行 `systemctl --user daemon-reload`，让当前会话立即识别新单元。

## 自启动机制

COSMIC 会话使用 `systemd-xdg-autostart-generator` 处理 `~/.config/autostart`。生成的单元：

```
app-rpci\x2dime@autostart.service
SourcePath=/home/alec/.config/autostart/rpci-ime.desktop
ExecCondition=/usr/lib/systemd/systemd-xdg-autostart-condition "COSMIC" ""
PartOf=graphical-session.target
After=graphical-session.target
```

检查结果：

```bash
systemctl --user is-active xdg-desktop-autostart.target
# active

ls -l /run/user/$(id -u)/systemd/generator.late/xdg-desktop-autostart.target.wants/ | grep rpci
# app-rpci\x2dime@autostart.service -> ../app-rpci\x2dime@autostart.service
```

启动测试：

```bash
systemctl --user stop  'app-rpci\x2dime@autostart.service'
systemctl --user start 'app-rpci\x2dime@autostart.service'
pgrep -a -x ime-probe
# 32730 /home/alec/.local/bin/ime-probe
```

随后 `vkey wubi` 提交「你」，说明 autostart 拉起的实例可以正常工作。

## 验证脚本

```bash
./prototype/scripts/verify-install.sh
```

检查项：

- release 二进制、`ime-settings` 存在
- 配置和 RIME 数据存在
- autostart desktop 存在，`Exec=` 指向可执行文件
- `OnlyShowIn` 包含 `COSMIC`
- systemd 单元已生成并链接进 `xdg-desktop-autostart.target`
- target 处于 active

加 `--start` 会先启动该单元并检查进程。

### desktop-file-validate 的已知警告

`desktop-file-validate` 会报告：

```
value "COSMIC;" for key "OnlyShowIn" ... contains an unregistered value "COSMIC"
```

这是 `desktop-file-utils` 的注册表里还没有 COSMIC，不是 autostart 失效。systemd 生成器使用
`XDG_CURRENT_DESKTOP=COSMIC` 作为 `ExecCondition`，实际检查通过（`ExecCondition ... "COSMIC"` 返回 0）。
因此验证脚本把它标记为 `warn`，而不是失败。

## 重启后实测（已完成）

用户重启后没有手动启动输入法，实际检查结果：

- 系统启动时间：`2026-10-20 07:02:13`
- autostart 单元启动时间：`2026-10-20 07:02:28`
- 主进程：PID `1797`，路径 `/home/alec/.local/bin/ime-probe`
- `ActiveState=active`，`SubState=running`
- `xdg-desktop-autostart.target` 为 active，`wants` 链接存在
- 二进制 BuildID 为 `28f8ef8a100fbaa6fbde753f7d4fa3fb5a20368e`，与 release 产物一致
- 日志出现 `COMMIT = "你"`
- 聚焦测试窗口后执行 `vkey wubi`，文件字节为 `e4 bd a0`（「你」）

结论：真实重启后 autostart 自动拉起成功，无需手动启动。

## 注销/重新登录验证（已完成）

已于 2026-10-20 07:02 完成。以下步骤保留，供以后回归重测：

1. 注销当前 COSMIC 会话并重新登录。
2. 不手动启动任何东西，检查：

```bash
pgrep -a -x ime-probe
systemctl --user status 'app-rpci\x2dime@autostart.service'
```

3. 聚焦文本框，输入 `wqiy` + space，应提交「你」。

如果未自动启动：

```bash
systemctl --user status 'app-rpci\x2dime@autostart.service'
journalctl --user -u 'app-rpci\x2dime@autostart.service' -b --no-pager
systemctl --user start 'app-rpci\x2dime@autostart.service'
```

同时确认：

```bash
echo "$XDG_CURRENT_DESKTOP"      # 应为 COSMIC
cat ~/.config/autostart/rpci-ime.desktop
```

## 当前状态

- release 安装和运行：已验证。
- systemd autostart 生成、链接、手动启动、输入：已验证。
- 真实重启后的自动拉起：已验证（PID 1797，07:02:28 启动）。
- 当前会话中的输入法由 autostart 启动并保持运行；测试窗口已关闭。
