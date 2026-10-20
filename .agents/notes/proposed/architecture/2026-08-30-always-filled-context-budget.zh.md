# Agent Note: Always-filled context budget and on-demand capability gating

Status: proposed

[English](2026-08-30-always-filled-context-budget.md) | 中文

## Problem

在本仓库中，一个全新 standard 档会话的首次模型请求报告 26,988 个输入 token，上下文窗口为 262,144——用户还没输入一个字，就有约 10.3% 的上下文被填充。测量来自本会话自身的日志（会话 `8be52d15…`）：第一条 `assistant/message` 的 usage 记录，以及日志中 `request/header` 所含的完整系统提示与 48 个工具 schema；按报告用量以 4.45 字符/token 校准。

实测的基线构成：

| 组件 | ~Tokens | 占比 |
|---|---:|---:|
| 工具 schema（48 个工具） | 10,900 | 40% |
| 系统提示（身份、persona、工具策略） | 5,600 | 21% |
| 技能目录（62 个技能，完整描述） | 5,400 | 20% |
| 工作区指令（仓库 + 家目录 AGENTS.md） | 4,300 | 16% |
| 记忆预览、运行时快照、用户消息 | 790 | 3% |

工具 schema 内的家族：核心 file/edit/web/bash 2.0k，meta（ask/plan/todo/skill）0.8k，编排（agents/jobs/workflow/ralph）2.8k，cordis 动态插件 1.7k，goals 0.5k，OpenViking MCP 3.1k。系统提示中约 0.7k 在复述 `cordis-plugin-development` 技能已承载的动态插件教程。

供应商提示缓存从实测轮次的第二步起回收了约 96% 的前缀，因此基线的直接 API 成本在首次请求之后是缓存价。剩余成本是真实的：订阅的限额窗口按输入 token 计量，即使缓存命中，首 token 延迟也随提示长度增长，且每条用户消息之前都压着约 17k 的常驻样板。今天没有任何策略或测量协议为这些成本负责；常驻上下文是插件组合的涌现总和。

技能系统已经实现了惰性加载（会话开始时是目录，加载时是全文），动态 Cordis 插件也实现了会话中期的能力增长。缺失的是组合必须达标的预算，以及"能力组被隐藏而非加载"时的按需获取路径。

## Proposal

采纳带实测验收标准的常驻上下文预算，分三层落地。本 note 拥有该策略与仓库侧变更；部署预设与会话插件是记录在此作为背景的仓库外操作化。

1. **策略。** standard 档的常驻基线目标 ≤ 14.3k token（实测 27.0k），每请求增长由下述家族与步级预算约束。预算通过测量执行，不通过运行时拒绝。
2. **A 层——部署预设（仅配置，27.0k → ~16.7k）。** 精简 agent 预设挂载核心循环、file/shell/web 工具与 cordis 家族（保留 `cordis_define` 的会话中期增长路径），并指向约 12 个技能的精简根；省略编排、goals 与 OpenViking MCP 客户端。
3. **B 层——会话级门控（动态插件，27.0k → ~14.3k）。** 监听 `system-prompt/assemble` 的 waterfall 过滤可见工具至核心与 meta 家族加两个元工具：`enable_tools(group)` 让下一次请求携带被门控家族的 schema（启用后保持粘性），`find_skill(query)` 用 OpenViking 对技能描述的语义检索取代 62 项目录，目录收缩为一行索引。
4. **C 层——仓库变更（上游，随 A+B 语义出货后 27.0k → ~10.5k）。** 三项变更：两级技能目录（目录提示中只留名称加 ≤8 词标签；完整描述经技能工具的 describe/search 模式获取）、把动态插件教程从系统提示折入现有技能、MCP 按服务器的惰性工具暴露（每服务器一个分发工具，而非每操作一个 schema）。

## Measurement protocol

- **基线。** 全新会话；从会话日志（zstd JSONL）读取第一条 `assistant/message` 的 `usage.inputTokens` 与 `request/header` 的组件字符量。
- **增长。** 同一日志中每步的输入 token 增量。
- **能力回归。** 约 12 个脚本化任务，半数需要被门控的家族；门控后的成功率必须与未门控相等，并报告 under-summon 重试计数。
- **缓存行为。** 10 步会话的缓存读占比；每组任务的组翻转上限 2 次。

## Alternatives considered

- **什么都不做，依赖提示缓存。** 缓存已让基线的每请求成本很低，但五小时订阅窗口按输入 token 计量，每次请求的首 token 延迟随提示长度增长，且每个会话的首次请求仍按全价支付基线。作为策略被否决，因为没有任何东西为这个数字负责。
- **全局裁剪工作区指令。** 文档标准把常驻指令保存在根与子树 AGENTS.md 中作为受预算约束的层级，且仓库专属块在非仓库工作区本就不存在。裁剪它是以约 4.3k token 换走真实指引。否决。
- **隐藏工具却不留召唤路径。** 只有配置级工具移除而没有 `cordis_define` 兜底或 `enable_tools`，模型将无法请求它看不见的能力，重型任务无诊断地失败。否决；两层都保留召唤路径。

## Acceptance criteria

1. 无键 recorded-session 快照携带出货档位的首次请求 `inputTokens` 与目录/工具字符预算，并在变更把常驻基线推过策略目标时失败。
2. 两级目录落地后，目录提示块 ≤ 6k 字符且 62 个技能全部在列，每条完整描述经一次技能工具调用可达。
3. 系统提示中动态插件材料 ≤ 1k 字符，其余仍可按名称从技能目录加载。
4. OpenViking MCP 贡献每服务器一个分发工具；15 个操作全部经它可达。
5. 回归评测报告门控与未门控任务成功率相等，且平均每任务 ≤ 1 次 under-summon 重试。

## Risks

- **Under-summoning。** 模型无法请求它看不见的能力；常驻系统行列出被门控的组，under-summon 重试计数是被测的反向指标。
- **缓存抖动。** 每次 `enable_tools` 翻转都会重建一次提示缓存前缀；组保持粘性，翻转上限 2 次/任务。
- **可发现性回归。** 用语义检索取代目录会失去顺带发现技能的机会；一行索引保持名称可见，`find_skill` 承载召回路径。
- **Model-visible ⟺ logged。** 任何按需注入都必须经日志机制落地（spliced inbox 事件）；门控不得绕过会话日志。
- **下游更新。** 两级目录与教程折叠改变模型可见文本，因此 recorded 快照、包 README 的 Model Experience 小节与 [skill-system note](../../archived/feature/2026-07-05-skill-system.md) 在同一 PR 中更新；若 usage 事件形状变动，[projected-token-usage note](../../implemented/architecture/2026-07-29-projected-token-usage-and-request-context.zh.md) 一并更新。
