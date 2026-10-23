# ADR 0006：用户词库删除优先采用候选内删除

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

**删除能力只做“当前候选内删除”，词库页面继续保持只读。**

采用与 fcitx5-rime 相同的模式：

```text
用户输入编码
→ 候选列表出现目标词
→ 用户在候选窗上触发删除动作
→ RIME 从用户词典删除该候选
```

实现约束：

1. 使用 RIME 的 `delete_candidate` / `delete_candidate_on_current_page` API。
2. 不直接操作 LevelDB。
3. 不重建 userdb。
4. 不需要停止输入法。
5. 候选窗保持只读；删除是按键动作，不增加可点击词库编辑器。
6. 只有来自用户词典的候选可以删除；底层词库候选删除失败时给出反馈。

## 为什么这是更安全的方法

- RIME 自己持有 userdb 锁和元数据
- 不需要 stop/start
- 不需要完整复制或重建词库
- 不会出现“导入成功但元数据丢失”
- 与 fcitx5-rime 的成熟行为一致

## 暂不采用

### 导出-过滤-重建

**暂不实现。**

在用户数据副本上验证过：

```text
170 条
删除 1 条
移除 userdb
--import 后
169 条
```

技术上可行，但需要重建整个数据库，且有 tick/元数据、磁盘空间、失败恢复风险。只有未来确实需要“从词库页面删除任意词条”时，才重新评估。

### 直接 LevelDB

**拒绝。** 需要理解 key/value 编码、`UserDbValue`、metadata 和 LevelDB 锁；RIME 升级后可能失效。

### C++ helper 调 `Db::Erase`

**暂不采用。** 虽然只改一条，但引入本地编译产物、librime ABI 依赖和新的崩溃面。

### 从读者页面删除任意词条

**暂不实现。** 当前不提供。

## 候选删除的接口方向

`rime-api 0.12.2` 没有封装 `delete_candidate`，需要：

- 增加一个最小 FFI/wrapper；或
- 在现有 RIME adapter 内调用 C API

删除动作应落在 InputSession/引擎中，而不是候选窗 UI 里，保持：

```text
按键/命令 → Action → RIME adapter
```

具体快捷键和反馈方式另行设计，不在本 ADR 决定。

## 后果

- 用户不能从只读词库页面删除任意词条。
- 删除只能在目标词作为当前候选出现时执行。
- 删除失败必须有反馈，不能静默。
- 不需要新增 `librime-bin` 之外的工具依赖。
- 不需要改现有备份/恢复流程。

## 待确认

- 删除快捷键用 `Ctrl+Delete` 还是候选窗动作。
- RIME 返回删除失败时，候选窗如何提示。
- 是否需要后续再做临时目录原子重建，作为“词库页面删除”的升级路径。

## 参考

- [USER-DICTIONARY-SPIKE.md](../USER-DICTIONARY-SPIKE.md)
- [fcitx5-rime rimecandidate.cpp](https://github.com/fcitx/fcitx5-rime/blob/master/src/rimecandidate.cpp)
- [fcitx5-rime rimestate.cpp](https://github.com/fcitx/fcitx5-rime/blob/master/src/rimestate.cpp)
- [rime_dict_manager.cc](https://github.com/rime/librime/blob/master/tools/rime_dict_manager.cc)
