# 用户词库管理 Spike

日期：2026-10-23

目标：确认 RIME 用户词典的可读性、可修改性，以及 RPCI 第一版用户词库管理能安全做到什么程度。

结论先行：

> **第一版只做只读查看/导出，不做删除。**
>
> `rime-api 0.12.2` 没有用户词典接口；`rime_dict_manager` 有受支持的
> `--list` / `--export` / `--backup` / `--import`，但**没有删词命令**。
> Weasel、Squirrel、fcitx5-rime 的处理方式也支持这个边界：成熟前端提供
> 列出/备份/恢复/导出/导入，删除通常只发生在当前候选上下文。
> 直接改 LevelDB 或绕过 RIME 生命周期风险过高，应单独设计并先用 ADR 固化。

## 已验证事实

### 1. rime-api 没有用户词典 API

本地 crate：

```text
~/.cargo/registry/src/index.crates.io-*/rime-api-0.12.2/src/lib.rs
```

`rime-api` 的 `Session` 只封装：

- `process_key`
- `context`
- `commit`
- `status`
- `select_schema`
- `simulate_key_sequence`

没有 userdb 枚举、导出、导入或删除方法。

`RimeDeleteCandidate` 虽然在 C 头文件中存在，但 rime-api 没有封装，而且它面向“当前候选”，不是通用的词库浏览器。

### 2. 系统提供了受支持的维护工具

```text
$ rime_dict_manager
options:
  -l|--list
  -s|--sync
  -b|--backup dict_name
  -r|--restore xxx.userdb.txt
  -e|--export dict_name export.txt
  -i|--import dict_name import.txt
```

来源：

- `/usr/bin/rime_dict_manager`
- `dpkg -S`：`librime-bin`
- librime 源码：`tools/rime_dict_manager.cc`
  https://github.com/rime/librime/blob/master/tools/rime_dict_manager.cc

`--export` 的实现：

- `UserDbManager::Export`
- `db->OpenReadOnly()`
- 写 TSV：`text<TAB>code<TAB>commit_count`
- 返回导出条目数；失败返回 `-1`

源码：

```text
tools/rime_dict_manager.cc:94-100
src/rime/lever/user_dict_manager.cc:107-128
```

### 3. 用户词典实际存储为 LevelDB

本机路径：

```text
~/.local/share/rpci/rime/wubi86.userdb/
~/.local/share/rpci/rime/pinyin_simp.userdb/
```

内部文件示例：

```text
CURRENT
MANIFEST-*
*.ldb
*.log
LOCK
```

这是 LevelDB，不是可直接按行编辑的文本。权威接口是 librime 的
`UserDb` / `Db`，不是直接改文件。

相关头文件：

```text
/usr/include/rime/dict/user_db.h
/usr/include/rime/dict/db.h
```

`UserDbValue` 包含：

```text
commits
dee
tick
```

用户词条不是只有文本，还有词频/时间等元数据。

### 4. 导出格式已实测

在**用户数据副本**上执行（不改真实词库）：

```bash
rime_dict_manager --export wubi86 /tmp/rime-wubi-export.txt
```

结果：

```text
exported 170 entries.
```

头部：

```text
# Rime user dictionary export
#@/db_name    wubi86
#@/db_type    userdb
#@/rime_version    1.10.0
#@/tick    183
#@/user_id    6b333f84-4ec2-4325-95a1-039eb85d30af
```

词条示例：

```text
工	a	1
东奔西走	adsf	1
世界	anlw	2
苛夺	asdf	17
```

字段含义：

```text
text    code    commits
```

这说明只读查看和导出是可行且低风险的。

### 5. 工具必须以 RIME 用户数据目录为工作目录

`rime_dict_manager` 使用 `Service::instance().deployer()`，而
`Deployer` 的默认路径是 `"."`：

```text
src/rime/deployer.cc:17
user_data_dir(".")
```

工具自身从当前目录读取 `installation.yaml`，并未接受
`--user-data-dir` 参数。因此调用方式必须是：

```bash
cd ~/.local/share/rpci/rime
rime_dict_manager --export wubi86 /tmp/export.txt
```

或在 Rust 中使用：

```rust
Command::new("rime_dict_manager")
    .current_dir(paths.rime)
    .args(["--export", "wubi86", export_path])
```

### 6. 必须先停止输入法

`rime_dict_manager` 的头文件明确要求：

```text
CAVEAT: the user dict should be closed before the following operations
```

LevelDB 由当前运行的 RIME 进程持有。导出虽然使用 `OpenReadOnly()`，
但仍可能遇到锁或读到正在写入的状态。

安全流程：

```text
stop systemd input method
→ export
→ start systemd input method
```

不论成功失败都必须 `start`，避免输入法停留在停止状态。

### 7. 没有删词命令

`rime_dict_manager` 的命令只有：

```text
list / sync / backup / restore / export / import
```

`--import` 是合并/覆盖写入，不是删除；不能用它精确移除单个词条。

可行但高风险的替代：

1. **直接 LevelDB 删除**：需要自己打开 DB、理解 key/value 编码和元数据；不推荐。
2. **写 C++ helper 调 `Db::Erase`**：可行，但引入本地编译产物和 ABI 风险。
3. **用 `RimeDeleteCandidate`**：只适合当前组合中的候选，不适合离线管理任意词条。
4. **导出 → 过滤 → 重建 userdb**：会重建数据库，必须 stop、备份、验证，风险最高。

因此第一版不做删除。

### 8. 现有 RPCI 备份已覆盖用户词库

`prototype/src/backup.rs` 的 `export_to` 会复制整个：

```text
~/.local/share/rpci/rime
```

到备份目录，其中包括 `*.userdb`。所以用户词库天然进入现有备份/恢复链路，不需要另建备份格式。

但恢复时仍应走现有：

```text
stop → import_from → start
```

不要绕过 `SettingsController` / `SettingsBackend` 的流程。

## 其它 RIME 前端怎么做

| 前端 | 用户词库管理 | 删除词条 | 结论 |
|---|---|---|---|
| Weasel（Windows） | 独立词典管理对话框：列出、备份、恢复、导出、导入 | 无 | 与本项目建议的只读/导出范围最接近 |
| Squirrel（macOS） | 打包 `rime_dict_manager`，未发现独立词典管理窗口 | 无独立入口 | 依赖官方维护工具 |
| fcitx5-rime（Linux） | 没有独立词典管理 UI | 仅在当前候选上调用 `delete_candidate` | 删除是候选动作，不是词库浏览器 |
| RIME 官方工具 | `rime_dict_manager`：list/backup/restore/export/import | 无 | 受支持能力边界 |

### Weasel

Weasel 的 `WeaselDeployer/DictManagementDialog` 提供：

- 列出 user dict
- backup
- restore
- export
- import

源码：

- https://github.com/rime/weasel/blob/master/WeaselDeployer/DictManagementDialog.cpp
- https://github.com/rime/weasel/blob/master/WeaselDeployer/Configurator.cpp

它通过 `RimeLeversApi` 的：

```text
user_dict_iterator_init
next_user_dict
backup_user_dict
restore_user_dict
export_user_dict
import_user_dict
```

实现，而不是直接打开 LevelDB。

这些 API 定义在：

```text
/usr/include/rime_levers_api.h
```

注意：`RimeLeversApi` 不是 `RimeApi`，需要 `find_module("levers")` 获取；
`rime-api 0.12.2` 没有封装。Weasel 同时使用 maintenance mode 在操作前暂停服务。

Weasel 的管理对话框**没有删除**用户词条的功能。

### Squirrel

Squirrel 的构建脚本把 `rime_dict_manager` 复制进 app：

```text
Makefile:52
cp $(RIME_BIN_DIR)/rime_dict_manager bin/
```

源码中未发现类似 Weasel 的独立用户词典管理窗口；它沿用官方工具的能力边界。

### fcitx5-rime

fcitx5-rime 的删除入口是候选词的 `forget`：

```text
RimeCandidateWord::forget()
  → RimeState::deleteCandidate()
  → api->delete_candidate(session, idx)
```

源码：

- https://github.com/fcitx/fcitx5-rime/blob/master/src/rimecandidate.cpp
- https://github.com/fcitx/fcitx5-rime/blob/master/src/rimestate.cpp

它没有“离线浏览全部用户词条”的界面；删除只针对当前输入会话中的候选。

### 对 RPCI 的含义

生态里的成熟前端并没有把“任意浏览 + 任意删除”做成一个通用词库编辑器。
它们普遍：

1. 用 RIME 的 userdb/levers API 或官方工具做 list/backup/restore/export/import
2. 把删除限制在当前候选上下文
3. 不直接碰 LevelDB

这支持本 spike 的结论：第一版做只读查看/导出是符合生态做法的；删除应单独设计。

## API 清单

| 能力 | 可用性 | 接口 |
|---|---|---|
| 列出用户词典 | ✅ | `rime_dict_manager --list` |
| 导出词条 | ✅ | `rime_dict_manager --export <dict> <file>` |
| 备份 userdb | ✅ | `rime_dict_manager --backup <dict>` |
| 恢复/合并快照 | ✅ | `rime_dict_manager --restore <file>` |
| 导入词条 | ✅ | `rime_dict_manager --import <dict> <file>` |
| 查看任意词条 | ⚠️ 需导出后解析 TSV | 无直接 API |
| 删除词条 | ❌ | 无受支持命令 |
| 调整词频 | ⚠️ 只能重建/导入 | 无安全单条接口 |
| Rust 直接 API | ❌ | `rime-api 0.12.2` 无 userdb |

## 推荐的第一版范围

### 做

- 设置界面增加“用户词库”只读页面
- 列出 `wubi86` / `pinyin_simp` 的导出结果
- 显示：词条、编码、词频
- 支持导出到用户选择的文本文件
- 操作前 stop，操作后 start
- 任何失败都保证 start
- 导出内容按 TSV 解析，空行/注释行忽略

### 暂不做

- 删除词条
- 修改词频
- 自造词 UI
- 直接操作 LevelDB
- 重建 userdb
- 导入/合并 UI

## 建议的实现 seam

新增 library module：

```text
prototype/src/user_dictionary.rs
```

接口保持小而可测：

```rust
pub trait UserDictionaryBackend {
    fn list_dicts(&mut self) -> Result<Vec<String>, String>;
    fn export(&mut self, dict: &str) -> Result<Vec<UserEntry>, String>;
}
```

真实 adapter：

```text
RuntimeCommandBackend
  current_dir(user_data_dir)
  stop input method
  run rime_dict_manager --export
  start input method
```

测试 adapter：

```text
FakeBackend
```

需要注入：

- command runner（方便断言命令和 cwd）
- service start/stop（或复用 SettingsBackend 的 stop/start）
- 临时文件路径

不要测试真实 LevelDB；只测试：

- TSV 解析
- stop/export/start 顺序
- export 失败仍 start
- 空词典
- 含中文、空格、制表符转义

## 风险

| 风险 | 影响 | 缓解 |
|---|---|---|
| RIME 正在写 userdb | 导出失败或数据不一致 | 先 stop，失败也 start |
| 直接改 LevelDB | 词库损坏 | 第一版禁止 |
| 恢复覆盖用户词库 | 丢失自造词 | 现有备份 + 二次确认 |
| 导出的词条含控制字符 | UI/终端问题 | 转义/过滤控制字符 |
| `librime-bin` 未安装 | 命令不存在 | 包依赖或运行时检查并提示 |
| 大词库导出慢 | UI 卡住 | 放到后台 Task；先限制大小/提示 |

## 包依赖

当前 `.deb` 只依赖 `librime1t64` / `librime-data`，不保证有
`rime_dict_manager`。如果实现只读导出，需要增加：

```text
librime-bin
```

或运行时检查并显示明确的安装提示。

注意许可证：`rime_dict_manager` 来自 Debian/Ubuntu `librime-bin`，其中包含
GPL-3 组件；RPCI 整体 GPL-3.0-or-later，兼容。

## 测试策略

单元测试：

- TSV 解析
- 命令参数与 cwd
- stop → export → start 顺序
- export 失败仍 start
- 空列表
- 导出路径清理

手工测试：

- 真实 `wubi86` 导出
- 中文词条显示
- 设置界面打开/关闭
- 导出过程中取消/失败
- 输入法在操作后仍可正常输入

## 当前实现状态

第一版只读查看/导出已落地：

- `prototype/src/user_dictionary.rs`
- 设置界面“用户词库（只读）”卡片
- `rime_dict_manager` 列表/导出
- stop → export → start
- 导出到 `~/.local/share/rpci/exports/`

候选内删除已实现：`Ctrl+Shift+1..9` 删除当前候选页对应候选。
词库页面任意删除、调频、自造词仍未实现。删除方案见 [ADR 0006](adr/0006-user-dictionary-deletion.md)：优先候选内删除，不做词库页面任意删除。

## 开放问题

1. 只读页面是否应该显示全部 170+ 条，还是搜索/分页？
2. 导出路径由用户选择，还是固定到 `~/Downloads`？
3. 是否把 `rime_dict_manager` 的调用放进现有 `SettingsBackend`，还是独立
   `UserDictionaryBackend`？
4. 删除功能是否值得做？如果做，优先 C++ helper 还是 RIME session 删除候选？
5. 用户词库页面是否需要独立窗口，避免设置窗口阻塞？
