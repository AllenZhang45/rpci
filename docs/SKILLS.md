# DSH Skills

本项目按 DSH 的文件系统技能格式导入了一组工程流程技能。

## 来源与许可

- 来源：[mattpocock/skills](https://github.com/mattpocock/skills)
- 导入版本：`5beb35b040e3de97e54fa9e386d7e42d15c295c2`
- 许可：MIT，原文见 [`.agents/skills/LICENSE`](../.agents/skills/LICENSE)
- 技能正文保持上游内容；本文件只做 DSH 和本项目的适配说明。

## 已导入

| Skill | 何时使用 | 本项目映射 |
|---|---|---|
| `tdd` | 新功能或 bug，需要 test-first | seam 取 `engine`、`paths`、`backup`、`config`；命令 `cargo test -p ime-probe` |
| `diagnosing-bugs` | 难复现、难定位的 bug | 先读 [STATUS.md](STATUS.md)、[history/SPIKE.md](history/SPIKE.md)；Qt6/候选窗问题适用 |
| `code-review` | 提交前审查一个 diff | standards 用 [AGENTS.md](../AGENTS.md)，spec 用 [ROADMAP.md](ROADMAP.md)；`git diff <base>...HEAD` |
| `codebase-design` | 设计/重构模块接口 | 模块：`engine`、`popup`、`paths`、`main`、`backup`、`settings-iced` |
| `writing-for-agents` | 写 AGENTS.md、docs、技能 | 文档索引 [docs/README.md](README.md)；当前事实 [STATUS.md](STATUS.md) |

## 全自动模式适配

这些技能默认假设可以随时向用户确认。本项目在“全自动开发”模式下采用以下规则：

- `tdd`：seam 由现有接口和文档确定；把选择的 seam 和理由写进提交/文档，不阻塞等待确认。
- `code-review`：使用 `subagent` 并行审查 standards 和 spec；spec 以 ROADMAP/当前任务为准。
- `diagnosing-bugs`：保留复现、最小化、假设、验证的循环；跳过交互式 HITL 脚本，改为记录证据。
- `grill-*` 类技能未导入；需要需求澄清时由用户显式触发。
- 任何技能都不得覆盖 [AGENTS.md](../AGENTS.md) 的提交和测试要求。

## DSH 发现规则

DSH 只扫描：

```
<project>/.dsh/skills/<name>/SKILL.md
<project>/.agents/skills/<name>/SKILL.md
```

每个技能必须是一层目录，不能是 `skills/engineering/<name>/SKILL.md` 这种嵌套结构。

本项目放在 `.agents/skills/<name>/`，资源文件与 `SKILL.md` 同目录保留。

## 更新

1. 拉取上游；
2. 只复制需要的技能目录到 `.agents/skills/<name>/`；
3. 更新本文件的 commit 与技能列表；
4. 保留 LICENSE 和来源说明；
5. 运行一次对应的验证（例如 `skill("tdd")` 能加载）。

不要使用 README 里的 `claude plugin` / `codex plugin` / `npx skills` 安装方式；DSH 使用文件系统技能发现。
