# DSH Skills

本项目按 DSH 的文件系统技能格式导入了一组工程流程技能。

## 来源与许可

- 来源：[mattpocock/skills](https://github.com/mattpocock/skills)
- 导入版本：`5beb35b040e3de97e54fa9e386d7e42d15c295c2`
- 许可：MIT，原文见 [`.agents/skills/LICENSE`](../.agents/skills/LICENSE)
- 技能正文保持上游工程流程；`grill-me` / `grill-with-docs` 移除了上游的 `disable-model-invocation`，以便 DSH 显式调用。

## 已导入

| Skill | 何时使用 | 本项目映射 |
|---|---|---|
| `tdd` | 新功能或 bug，需要 test-first | seam 取 `engine`、`paths`、`backup`、`config`；命令 `cargo test -p ime-probe` |
| `diagnosing-bugs` | 难复现、难定位的 bug | 先读 [STATUS.md](STATUS.md)、[history/SPIKE.md](history/SPIKE.md)；Qt6/候选窗问题适用 |
| `code-review` | 提交前审查一个 diff | standards 用 [AGENTS.md](../AGENTS.md)，spec 用 [ROADMAP.md](ROADMAP.md)；`git diff <base>...HEAD` |
| `codebase-design` | 设计/重构模块接口 | 模块：`engine`、`popup`、`paths`、`main`、`backup`、`settings-iced` |
| `writing-for-agents` | 写 AGENTS.md、docs、技能 | 文档索引 [docs/README.md](README.md)；当前事实 [STATUS.md](STATUS.md) |
| `grilling` | 对计划或设计做多轮压力测试 | 事实来自 STATUS、ROADMAP、PROTOCOL-AUDIT；问题按轮次，不一次性堆完 |
| `domain-modeling` | 打磨术语、更新 GLOSSARY / ADR | 根目录 [GLOSSARY.md](../GLOSSARY.md)、[docs/adr/](adr/) |
| `grill-me` | 只要 grilling 的入口 | 路由到 `grilling` |
| `grill-with-docs` | grilling + 领域文档 | 路由到 `grilling` + `domain-modeling`，决策落 GLOSSARY/ADR |

## 全自动模式适配

这些技能默认假设可以随时向用户确认。本项目在“全自动开发”模式下采用以下规则：

- `tdd`：seam 由现有接口和文档确定；把选择的 seam 和理由写进提交/文档，不阻塞等待确认。
- `code-review`：使用 `subagent` 并行审查 standards 和 spec；spec 以 ROADMAP/当前任务为准。
- `diagnosing-bugs`：保留复现、最小化、假设、验证的循环；跳过交互式 HITL 脚本，改为记录证据。
- `tdd`、`code-review`、`diagnosing-bugs` 仍按全自动模式执行；`grilling` 是例外，必须由用户显式触发并等待回答。
- 任何技能都不得覆盖 [AGENTS.md](../AGENTS.md) 的提交和测试要求。

## Grilling 项目适配

`grilling` / `grill-with-docs` 与本项目的配合规则：

1. 只由用户显式触发；不把全自动开发改成问答模式。
2. 事实由 Agent 从 `docs/STATUS.md`、`docs/ROADMAP.md`、`docs/PROTOCOL-AUDIT.md`、`docs/COSMIC-1.11-AUDIT.md`、`GLOSSARY.md`、`docs/adr/` 读取；不向用户索取可查事实。
3. 每轮只问当前 frontier；每题给推荐答案；等用户回答后再进入下一轮。
4. `grill-with-docs` 在决策稳定时更新 `GLOSSARY.md`；只有同时满足“难回滚、无上下文会意外、真实取舍”才新增 ADR。
5. 当前产品约束：COSMIC 1.11、Ctrl+Space、物理键盘、顶部 Panel 只读 applet、公共 APT、无需 F2。

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
