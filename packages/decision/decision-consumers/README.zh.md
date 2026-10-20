---
description: "十四个带类型的 decision 消费方，包括并行 canary 模型 failover 与召回记忆的实时筛查。"
kind: "package-reference"
---

# @deepseek-ai/dsh-decision-consumers

[English](README.md) | 中文

## 概述

`dsh-decision-consumers` 提供依据 `ctx.decision` 带类型判断进行分支的消费方。十二个导出函数分别对浏览器动作与目标、工具风险、模型档位、检索候选、compaction 条目、loop 状态、subagent 结果、验证工作、技能路由、网页内容安全以及与仓库指令的偏离进行分类。实时 failover 拦截符合条件的 Agent 请求失败，并发探测备选路由，由 decision 提供方选定一条健康路由，再通过它重试同一步骤。实时召回过滤在步骤运行前判断插件注入的每条记忆，省略低相关条目，并扣留存储型提示注入。

## 目录

- [使用本包](#use-this-package)
- [十四个消费方](#the-fourteen-consumers)
- [实时模型 failover](#live-model-failover)
- [实时召回过滤](#live-recall-filter)
- [理解实现](#understand-the-implementation)
- [延伸阅读](#further-exploration)
- [开发备注](#dev-note)
- [模型体验](#model-experience)
- [已知限制与待办工作](#known-limitations-and-deferred-work)

-----

<a id="use-this-package"></a>
## 使用本包

每个消费方都是一个导出函数，接受 `ctx.decision`（或一个满足 `DecisionService` 接口的对象）、针对该问题的结构化参数，以及可选的策略阈值。

```text
import { decideEgoStep, assessToolRisk, routeTurnModel } from '@deepseek-ai/dsh-decision-consumers'

// 1. Ego-browser navigation step
const step = await decideEgoStep(ctx.decision, {
  taskGoal: 'Open 2m USB-C Cable page',
  pageState: 'Search results list',
  candidates: [
    { id: '1', label: 'Braided USB-C Cable 2m' },
    { id: '2', label: 'Next Page' },
  ],
})

if (step.action.type === 'click') {
  await page.click(`@${step.action.elementId}`)
} else if (step.action.type === 'escalate') {
  // Low confidence or blocked page: escalate to user or full LLM turn
}
```

<a id="the-fourteen-consumers"></a>
## 十四个消费方

| 消费方 | 函数 | 用途 | 主要问题 |
|---|---|---|---|
| **1. 浏览器步骤驱动** | `decideEgoStep` | 自主浏览器导航 | `element`（Choice）、`task_complete`（Boolean）、`page_blocked`（Boolean）、`risk_level`（Score） |
| **2. 工具风险把关** | `assessToolRisk` | 批准前的执行风险分类 | `risk_score`（Score）、`requires_human`（Boolean）、`category`（Choice） |
| **3. 模型路由** | `routeTurnModel` | 每轮次的快速档与前沿档模型 | `tier`（Choice）、`complexity`（Score）、`deep_reasoning`（Boolean） |
| **4. 检索过滤** | `filterRetrievalCandidates` | 记忆与搜索分片的降噪 | 每个分片的 `relevance`（Score）、`keep`（Boolean） |
| **5. compaction 评分** | `scoreCompactionItems` | transcript 保留优先级 | 每个条目的 `val`（Score）、`must_keep`（Boolean） |
| **6. loop 停滞检测** | `detectLoopStall` | 重复与反复试探监控 | `loop_state`（Choice）、`is_stalled`（Boolean）、`severity`（Score） |
| **7. subagent 判定** | `evaluateSubagent` | 委派执行评估 | `verdict`（Choice：accept/retry/escalate）、`quality`（Score）、`criteria_met`（Boolean） |
| **8. 检查项选择** | `selectVerificationGates` | 推送前 git diff 分诊 | `suite`（Choice）、`blast_radius`（Score）、`needs_coverage`（Boolean） |
| **9. 模型 failover** | `installModelFailover`、`arbitrateModelFailover` | 并行 canary 判决与同步骤路由接管 | `best_provider`（Choice）、每个 worker 的有效性／效率／延迟（Score） |
| **10. 技能选择** | `selectSkill` | 每轮次的技能目录路由 | `needs_skill`（Boolean）、`skill`（Choice）、`risk`（Score） |
| **11. 网页内容把关** | `guardFetchedContent` | 对抓取网页的提示注入与凭据收集筛查 | `prompt_injection`（Boolean）、`credential_harvest`（Boolean） |
| **12. 指令偏离** | `assessInstructionDrift` | 对工作是否偏离仓库指令给出建议性判断 | `drift`（Boolean） |
| **13. 浏览器动作策略** | `decideBrowserAction` | 依据观测到的元素表选择下一个浏览器操作及其目标 | `operation`（Choice）、每个可用操作一个 `<operation>_target`（Choice） |
| **14. 召回过滤** | `installRecallFilter`、`screenRecallItems` | 在步骤前对召回记忆做相关性与存储型注入筛查 | 每个条目的 `rel_<n>`（Score，0..3 有用度）、`inj_<n>`（Boolean） |

<a id="live-model-failover"></a>
## 实时模型 failover

在函数插件上设置 `failover.enabled` 以注册 `agent/request-error` 与 `agent/request` 监听器。`useAllowedModels` 在恢复时把已启用的 `subagent-model-selection.allowedModels` 与 `candidates` 做并集，因此当设置列表是较短的显式集合时，仍会探测 `omniroute/auto/best-coding` 这类实时编码组合。该可选服务缺失或为空时，只使用 `candidates`。活动路由以及接管后失败过的每条路由，在该 Agent 步骤的剩余时间内保持排除。

```yaml
- name: '@deepseek-ai/dsh-decision-consumers'
  config:
    failover:
      enabled: true
      useAllowedModels: true
      candidates:
        - provider: omniroute
          model: claude/claude-sonnet-5
        - provider: omniroute
          model: cx/gpt-5.6-sol-medium
      triggerCodes: [QUOTA, RATE_LIMIT, SERVER, TIMEOUT, TRANSPORT, AUTH, PI_AI_ERROR]
      maxCandidates: 4
      probeTimeoutMs: 12000
      probeMaxTokens: 128
      maxTaskChars: 4000
      maxProbeOutputChars: 2000
```

每个候选收到的是有界的最近一条持久 user 消息，而不是隐藏的探测提示词。失败或为空的探测不具资格。其余输出、其延迟与 token 用量、以及任务描述共同构成一次 decision 请求。只要判决方把获胜路由的有效性评分置于 `broken-or-hallucinated` 之上，它就会接管，因为一条经过探测且健康的路由严格优于刚刚失败的那条路由。获胜选项概率是遥测而不是门槛：它报告判决方在本来都已健康的路由之间区分得有多强，因此近乎均等的排名绝不能把已死的路由留在原位。接管会安装一个临时的 `agent/request` 覆盖并返回 `{ kind: 'retry' }`；随之产生的 `request/header` 记录生效的提供方／模型。取消或插件 dispose 会中止并排空所有探测。没有健康候选、或获胜者被判决方评为代码损坏时，委派给下一个恢复监听器，通常是 `dsh-llm-retry`。JEV／Laya 判决方不可用时仍会接管：按候选顺序取第一条健康探测，以免冷却或 429 路由把本轮卡住。

<a id="live-recall-filter"></a>
## 实时召回过滤

设置 `recallFilter.enabled` 会注册一个 `agent/pre-step` 监听器，筛查 `source` 为 `{ kind: 'plugin', plugin: sourcePlugin, form: sourceForm }`（默认 `openviking-memory` / `recall`）的召回消息。监听器解析每个 `<memory uri="…">` 元素，把该步骤的用户文本、上一条 assistant 回复（最多 `maxContextChars`）以及每个条目正文的前 `maxItemChars` 个字符，按 `batchSize` 分批发送给 decision 提供方，并就地改写该消息：

- `inj_<n>` 概率不低于 `injectionThreshold` 的条目变为 `<memory uri="…" screened="withheld: flagged as possible prompt injection" />`。
- 提供方即使单独评估也拒绝的条目（`DECISION_REQUEST_REJECTED`，例如网关防火墙拒绝攻击形态的文本），在 `withholdUnscreenable` 为 true 时变为 `withheld: could not be screened` 存根，否则保留。被拒绝的批次会拆成单条请求，因此一个被拒条目不会让同批其他条目漏筛。
- 在 0..3 有用度量表上低于 `minRelevance` 的条目被省略，并用一行说明报告省略的条目数。

其他提供方失败、畸形答案或超过 `timeoutMs` 时，召回保持不变并记录一条警告。没有用户文本的步骤、含多个内容块的召回消息，以及不含 `<memory uri>` 元素的文本都不经筛查直接通过。

```yaml
- id: decision-consumers
  config:
    recallFilter:
      enabled: true
      minRelevance: 0.5
      screenInjection: true
      injectionThreshold: 0.7
      withholdUnscreenable: true
      batchSize: 12
      maxItemChars: 1500
      maxQueryChars: 2000
      maxContextChars: 1500
      timeoutMs: 3000
```

召回来源在自身 `next()` 之后追加消息，因此过滤器必须是最外层的 `agent/pre-step` 监听器。插件是并发应用的，配置顺序无法保证这一点；过滤器在每次 `agent/created` 时重新注册其 `prepend` 监听器，位于所有启动期注册之后。

<a id="understand-the-implementation"></a>
## 理解实现

每个判断都使用 `ctx.decision.evaluate()`。decision 提供方在一次请求中回答所有问题，因此消费方把候选选择与相关的 Boolean 和 Score 问题放在一起。failover 仲裁器为确切的提供方／模型路由分配不透明的 worker id，防止不同提供方上相同的模型 id 产生歧义。

动作可选的消费方会对照其配置的阈值检查获胜概率，并在提供方不确定时升权或委派。failover 不属于这一类：失败的路由已经不可用，因此它接受判决方排名最高的获胜者，并把通用的同路由重试留作回退恢复的归属方，而不是吞掉一个无法得出结论的 decision。

一次符合条件的失败最多增加 `maxCandidates` 个受 `probeMaxTokens` 限制的并发 canary 调用，并在至少两个候选存活时跟随一次 decision 调用。并行执行把墙钟时间约束在最慢探测的截止时间上，而不是它们的总和，且单一存活者会跳过 decision 调用。一个 Agent 步骤内重复的同路由失败会复用已结算的恢复结果，而不必重新付出一轮扇出；接管后失败的路由会改变失败路由集合，因而获得自己的一轮。每个 canary 收到有界的最近一条持久 user 消息作为其完整提示词，这正是扇出被限制在部署已授权路由上的原因。

<a id="further-exploration"></a>
## 延伸阅读

- [`@deepseek-ai/dsh-decision`](../decision/README.zh.md) —— Service Definition 与问题类型。
- [`@deepseek-ai/dsh-decision-jev`](../decision-jev/README.zh.md) —— TypeSafe Jev 提供方。
- [`@deepseek-ai/dsh-decision-llm`](../decision-llm/README.zh.md) —— 无需密钥的回退提供方。
- `tests/spike-10-step-browser.spec.ts` —— 带逐步遥测日志的 10 步自主导航演示。

<a id="dev-note"></a>
## 开发备注

每个消费方都被设计为快速失败：上游提供方返回的畸形答案或漏答问题会立刻抛出错误，而不是静默取默认值，从而确保未经校准的响应绝不会伪装成自信的自动化动作。

<a id="model-experience"></a>
## 模型体验

### 过滤后的召回消息

#### 模型看到什么

启用 `recallFilter.enabled` 后，模型收到的召回消息只包含不低于 `minRelevance` 的条目、每个被扣留条目的 `screened` 存根以及一行省略说明。改写后的消息在该步骤的准入消息中替换原消息，因此 session 日志记录的正是模型所见内容。其余导出函数返回分离的判断，实时 failover 把 canary 提示词发往探测路由，而不是 Agent 对话。

#### Token 影响

每个步骤的新尾部以及之后携带它的每个请求，都会少掉被省略的条目（在实测会话中约为召回字符的一半）；每个被扣留条目仍占用一个简短存根。

#### KV Cache 影响

过滤后的召回消息就是被记录的那一条，因此后续请求重放相同的字节，前缀缓存不受影响。Canary 调用只使用一条 user 消息，不复用 Agent 对话前缀。提供方／模型接管会改变主请求路由，因此缓存复用取决于所选适配器与提供方；调用方不得假定失败路由的缓存仍然有效。仲裁请求是一次独立的 decision 调用，其提示词由 decision 提供方构建并丢弃。

<a id="known-limitations-and-deferred-work"></a>
## 已知限制与待办工作

- **状态截断：** 提供大页面快照或 transcript diff 的消费方必须留意提供方的上下文限制；每个消费方都把状态截断到合理的边界。`scoreCompactionItems` 通过分级截断摘要来适配自身状态，并把问题拆分到多个并发请求中，因此它是例外；其余每个消费方仍只发送一个请求。
- **提供方相关的校准：** 诸如 0.70 置信度这类阈值是针对 TypeSafe Jev 校准的；使用 `dsh-decision-llm` 时，模型可能产生偏向 0 或 1 的未校准概率。
- **指令偏离的豁免条款：** `assessInstructionDrift` 只给出一个概率供上层展示，绝不阻断工作。针对 Jev 的实测表明，当某条指令带有明确豁免时（例如规则豁免安全不变量，而工作恰好是硬编码一个安全常量），被豁免的合规工作得到 0.82–0.91，而真正的违规为 0.92 以上。0.90 的默认阈值位于该区间之上；调低它会把合规工作报成偏离。
- **浏览器动作策略本身不拥有浏览器。** `decideBrowserAction` 依据调用方观测到的元素表选择操作与目标；它不读取 DOM、不执行动作，也不提供 TYPE_TEXT 的文本值。调用方必须原子地观测、把选定目标解析回活动节点、在执行前重新检查页面新鲜度与遮挡，并通过独立的文本模型生成字段文本。`decideEgoStep` 仍用于单元素点击步进，且只能表达点击。
- **批量检索规模：** 在一次调用中过滤数百个检索分片会生成很大的提示词；`scoreCompactionItems` 与 `screenRecallItems` 做了批处理；`filterRetrievalCandidates` 没有。
- **技能目录仲裁：** `selectSkill` 返回一个分离的单项建议。自动路由需要归属方先提供有界目录短名单、将 JEV 的分歧与 DSH 提议比较，并在注入任何技能指令前记录主技能、立即并行技能和后续深入技能。`tests/skill-selection-adjudication.e2e.ts` 中的真实提供方基准在不改变实时路由的前提下测量该候选策略。
- **Failover 扇出：** 每个已授权的 canary 都会收到最近一条 user 任务。处理提供方受限数据的部署必须把 `allowedModels` 与回退 `candidates` 保持在允许的提供方集合内。
- **召回过滤校准：** `minRelevance` 0.5 与 `injectionThreshold` 0.7 是在 40 条已记录召回（419 个条目）上、加入 20 条植入攻击与 20 条良性安全笔记后，针对 TypeSafe Jev 实测得出的：每条植入攻击得分 0.75–0.99 或被网关拒绝，良性条目与已记录条目均不超过 0.16。其他 decision 提供方需要各自校准。每个被筛查的步骤增加一次 decision 往返（p50 约 0.45 秒，p95 约 0.85 秒）。
- **召回过滤范围：** 只筛查来自已配置来源的单文本块消息；记忆插件的档案消息（`form: instructions`）以及不含 `<memory uri>` 元素的召回格式不经筛查直接通过。"go ahead" 这类简短跟进会结合上一条 assistant 回复来判断，但仍可能省略之后步骤所需的条目；省略说明会提示模型去搜索记忆。
- **仲裁器可用性：** 带排名的多存活者接管仍优先使用可触达的 JEV／Laya 判决方。判决方不可用时，failover 按候选顺序取第一条健康探测，而不是同路由重试，因此冷却或 429 模型无法卡住本轮。仅当所有 canary 也都失败时才回退到同路由重试。
