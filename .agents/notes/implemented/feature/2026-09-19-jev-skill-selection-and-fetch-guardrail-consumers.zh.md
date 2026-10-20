# Agent Note: JEV 技能选择与网页内容把关 decision 消费方

Status: implemented

[English](2026-09-19-jev-skill-selection-and-fetch-guardrail-consumers.md) | 中文

## 问题

harness 中两类高频判断仍要付出完整的 LLM 轮次，否则什么都没有。技能目录路由意味着 agent 要么烧掉一个推理轮次来决定是否加载技能，要么完全依赖循环自身的判断；而本机的 TypeSafe JEV decision seam 热调用实测为 345–460 ms，是做这类分类的更便宜位置。另外每条 `web_fetch`/`web_search` 结果未经筛查就进入模型上下文：不受信的外部文本与 harness 指令之间，模型无法可靠地区分，此前对注入的防御只是系统提示词里的请求，而不是可执行的策略。

对照 `Anil-matcha/awesome-jev-by-typesafe`（2026-09-19 快照）确认：DSH 已实现该目录九条编码 agent 模式——浏览器步骤、工具风险、模型路由／级联、检索过滤、agent-trace 判定、循环控制、推送前检查项分诊、failover 仲裁与 compaction 评分。技能选择与输入把关是目录携带而 DSH 缺失的两条。

## 决策

在 `@deepseek-ai/dsh-decision-consumers` 中新增两个分类消费方，沿袭本包"导出为作用于 `ctx.decision` 的纯函数"的既有形态；不新增 seam 面，不新增插件：

- `selectSkill`（`src/skill-selector.ts`）——一次扇出 `needs_skill`（Boolean）、`skill`（在提供的目录加 `none` 上的 Choice）、`risk`（三级 Score）；代码门：目录为空时本地判定 `direct_answer`，不调用提供方；needs-skill 概率低于阈值返回 `direct_answer`；选到 `none` 或获胜概率低于 `minSkillConfidence`（默认 0.65）返回 `clarify`；risk 评分达到 `approvalRiskThreshold`（默认 1.5）返回 `load_skill_with_approval`。
- `guardFetchedContent`（`src/fetch-guardrail.ts`）——对抓取的 URL、标题与长度受限的正文提出两个 Boolean 问题（`prompt_injection`、`credential_harvest`）；判定把较大的危害概率映射到 `pass`/`review`/`block` 区间（默认 0.30／0.70），并报告 `thresholdMargin`，即与所在区间门槛的距离。

## 已考虑的替代方案

- **在 `dsh-llm` 内做 prompt 级别的注入筛查**——否决：筛查应发生在引入不受信文本的那个能力上，而 decision 答案让策略在代码中可分支，而不是一句模型可见的字符串。
- **用一个 Choice 同时选 guardrail 判定**——否决：一个 Choice 把两个独立危害折叠进同一个分布；分开的 Noul 才能让策略区分"仅注入（剥离后继续）"与"凭据收集（直接阻断）"。
- **把技能选择塞进 `dsh-llm` 的提示词指导**——否决：decision seam 的存在正是为了让横切分类不变成一次模型轮次，且通向 `clarify` 的置信度门槛必须由代码持有。
- **增加 `other`／`unknown` 技能选项或校准锁文件（仓库的 `jevcal` 模式）**——延期：`none` 已覆盖目录不完整的情况，阈值校准是覆盖整个 decision seam 的仓库级工作，不属于这两个消费方。

## 后果

- 技能目录路由可以跑在 decision 延迟上（实测一次三问题、三候选调用约 1.1–1.2 s，含进程预热；两 Boolean 的把关调用 334–435 ms），而不是一个完整的 agent 轮次。
- 抓取的网页内容在进入模型上下文前得到可机器分支的注入与凭据收集筛查；`block` 判定的处理由调用方持有。
- `selectSkill` 保持为库函数，直到技能路由归属方提供有界候选短名单。`guardFetchedContent` 通过可选的 `fetchGuardrail` 配置接入 `dsh-tool-web`；web profile 启用它，没有 decision 提供方的 profile 保留默认的无筛查行为。
- 真实提供方基准将 DSH 提议与 11 个技能的有界目录比较，把分歧扩展为 DSH 提议加 JEV 高概率技能，并将伴随技能分类为立即并行或后续深入。五个已标注案例连续运行三次，主技能准确率从 60% 提升到 100%，平均真实值路由分数为 1.0，总 decision 延迟中位数为 782 ms（范围 348–1150 ms）。
- 限制已写入包 README：默认阈值是针对 TypeSafe Jev 校准的，正文截断（8000 字符）是提供方上下文边界，不是语义保证。
