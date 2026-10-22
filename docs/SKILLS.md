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

## DSH 全局技能库

除项目内的 9 个技能外，2026-10-23 将上游稳定的 engineering/productivity 技能安装到了 DSH 用户目录：

```text
~/.dsh/skills/
```

- 来源：[mattpocock/skills](https://github.com/mattpocock/skills)
- commit：`5beb35b040e3de97e54fa9e386d7e42d15c295c2`
- 排除：`setup-matt-pocock-skills`（会改 DSH 配置）
- 许可：MIT，见 `~/.dsh/skills/LICENSE-mattpocock-skills`

为了能从 DSH 显式调用，已移除全局副本中的 `disable-model-invocation: true`；上游原本把这些技能设计为“仅用户可调用”。这是本项目对 DSH 的适配，不改变项目内 `.agents/skills` 的同名技能。

DSH 的查找顺序是项目优先：

```text
.agents/skills/<name>   # 本项目，rank 200
~/.dsh/skills/<name>    # 全局，rank 400
```

因此 `tdd`、`grilling`、`grill-with-docs`、`diagnosing-bugs` 等仍使用项目内版本。

### 全局技能的本项目用法

| 全局技能 | RPCI 中的用途 |
|---|---|
| `to-spec` | 把 grilling 共识写成 `docs/` 下的可验收规格；本项目没有 issue tracker，不发布到外部 tracker |
| `to-tickets` | 拆成 todo / 任务清单；不创建外部 ticket |
| `implement`、`implement-spec` | 按规格实现；仍必须遵守 AGENTS.md 的测试和 commit 要求 |
| `improve-codebase-architecture` | 扫描 `engine`、`popup`、`paths`、`main` 等模块的耦合 |
| `research` | 查 Wayland/COSMIC/RIME 上游资料，结果写入 `docs/` |
| `pr` | 写 PR 描述 |
| `triage`、`wayfinder`、`wizard`、`retro`、`handoff` 等 | 按需使用；本项目当前没有 issue tracker，外部动作前先确认 |

“按需调用”指任务匹配时再加载对应技能，不代表每次任务都跑完整流程。

## 全自动模式适配

这些技能默认假设可以随时向用户确认。本项目在“全自动开发”模式下采用以下规则：

- `tdd`：seam 由现有接口和文档确定；把选择的 seam 和理由写进提交/文档，不阻塞等待确认。
- `code-review`：使用 `subagent` 并行审查 standards 和 spec；spec 以 ROADMAP/当前任务为准。
- `diagnosing-bugs`：保留复现、最小化、假设、验证的循环；跳过交互式 HITL 脚本，改为记录证据。
- `tdd`、`code-review`、`diagnosing-bugs` 仍按全自动模式执行；`grilling` 是例外，必须由用户显式触发并等待回答。
- 任何技能都不得覆盖 [AGENTS.md](../AGENTS.md) 的提交和测试要求。
- 全局技能中的 `to-spec` / `to-tickets` 默认面向 issue tracker；本项目改写为 `docs/` 和 todo，不擅自创建外部 issue。

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
