# ADR 0006：用户词库删除采用导出-过滤-重建

> 状态：Proposed（2026-10-23）
> 关联：[USER-DICTIONARY-SPIKE.md](../USER-DICTIONARY-SPIKE.md)、[0005-system-apt-readonly-applet.md](0005-system-apt-readonly-applet.md)

## 背景

第一版用户词库只支持只读查看和导出。删除是用户词库管理里唯一明显缺失、但风险最高的能力：

- RIME 用户词库是 LevelDB，不是纯文本
- `rime_dict_manager` 没有 `--delete`，只有 list/backup/restore/export/import
- `RimeLeversApi` 同样没有单条删除接口
- fcitx5-rime 的删除只针对“当前组合中的候选”，不是离线词库浏览器
- 直接打开 LevelDB 删除会绕过 RIME 的元数据和锁，风险不可接受

## 决策

**删除单条词条采用“导出 → 过滤 → 重建 userdb → 验证 → 启动”的流程，使用官方 `rime_dict_manager`，不在 RPCI 内直接操作 LevelDB。**

固定流程：

```text
1. 停止输入法
2. 在 RPCI 备份目录创建完整快照（复用现有 backup）
3. 在临时目录执行 rime_dict_manager --export <dict> full.tsv
4. 解析 TSV，删除目标词条，保留 #@ 元数据头
5. 将 live <dict>.userdb 重命名为 <dict>.userdb.deleting-<timestamp>
6. 在用户目录执行 rime_dict_manager --import <dict> filtered.tsv
7. 再次 --export 验证：
   - 条目数 = 原条目数 - 1
   - 目标词条不存在
   - db_name / user_id 未变
8. 验证通过：删除旧目录，启动输入法
9. 任一步失败：删除新目录，恢复旧目录，启动输入法
```

不自动删除任何内容，UI 必须二次确认。

## 为什么不用其它方案

### 直接 LevelDB 删除

**拒绝。** 需要自己理解 key/value 编码、`UserDbValue`、metadata 和 LevelDB 锁；RIME 升级后可能失效。

### C++ helper 调用 `Db::Erase`

**暂不采用。** 可行，但会引入本地编译产物、librime ABI 依赖和新的崩溃面。第一版优先复用已有工具。

### 只保留当前候选删除

**不作为唯一方案。** 它符合 fcitx5-rime 的模式，但只覆盖“正在输入的候选”，不能从只读词库页面删除任意词条。

### 不提供删除

**暂不作为最终决策。** 只读查看已经可用；删除若长期缺失，用户词库管理仍不完整。若后续验收认为重建风险过高，可退回此方案。

## 已验证

在用户数据副本上实测：

```text
导出 170 条
删除 1 条
移除 wubi86.userdb
rime_dict_manager --import filtered.tsv
→ imported 169 entries
再次导出
→ exported 169 entries
→ 目标词条不存在
```

重建后 metadata 中 `db_name`、`db_type`、`rime_version`、`user_id` 保留；`tick` 可能重新生成。RPCI 当前不同步用户词典，因此可以接受。

## 后果

- 删除会重建整个 userdb；操作期间输入法必须停止。
- 需要保证磁盘空间足够存放导出文件和重建数据库。
- 删除失败必须恢复旧目录，不能留下半成品。
- 用户词库备份仍由现有 `backup::backup_now` 负责；删除前强制创建。
- 需要为重建流程写 fake backend 测试，真实操作只做手工验收。
- `.deb` 继续依赖 `librime-bin`；不能用纯 Rust 独立运行。

## 待确认

- 是否接受“删除 = 重建整个 userdb”的代价。
- 是否需要在删除前显示目标词条和“将重建词库”的确认文案。
- `tick` 重置是否会影响用户未来使用 RIME sync；当前 RPCI 未实现 sync。

## 参考

- [USER-DICTIONARY-SPIKE.md](../USER-DICTIONARY-SPIKE.md)
- [rime_dict_manager.cc](https://github.com/rime/librime/blob/master/tools/rime_dict_manager.cc)
- [Weasel DictManagementDialog](https://github.com/rime/weasel/blob/master/WeaselDeployer/DictManagementDialog.cpp)
