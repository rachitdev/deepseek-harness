---
description: "decision 服务（ctx.decision）：插件如何通过可替换的提供方，就某一情境提出带类型、携带概率的问题，并共用一套选择策略与错误词汇。"
kind: "package-reference"
---

# @deepseek-ai/dsh-decision

[English](README.md) | 中文

## 概述

任何插件都可以通过 `dsh-decision`（`ctx.decision`）就某个情境提出带类型的问题（选一个选项、按评分标准打分、判断陈述真假），并得到校准概率而不是散文。服务每次调用选定一个可用提供方，因此调用方无需追踪由哪个模型作答。当代码必须依据自身无法计算的判断来分支时选择它，例如某次工具调用是否有风险。该服务不发起网络调用，也不注册面向模型的工具，因此必须先挂载提供方；所有问题类型共用一套选择策略、错误词汇与配置。

## 目录

- [使用本包](#use-this-package)
- [理解实现](#understand-the-implementation)
- [延伸阅读](#further-exploration)
- [模型体验](#model-experience)
- [已知限制与待办工作](#known-limitations-and-deferred-work)
- [开发备注](#dev-note)

-----

<a id="use-this-package"></a>
## 使用本包

需要带类型判断的组合加载 `dsh-decision` 服务并挂载至少一个后端，插件作者随后直接调用 `ctx.decision.evaluate()`。服务为每次调用解析后端，因此除非调用方自行配置，否则它看不到提供方 id。

### 何时选择它

当代码必须依据对非结构化状态的判断来分支，且答案是选择、评分或真值概率而非文本时，选择该服务。不要用它来生成散文、命名或提交信息：decision 答案不携带任何字符串。当分支可计算时你同样不需要它——路径比较或 schema 检查更便宜且精确。

该 seam 之所以存在、而不是再调用一次 `LlmService`，原因在于它的两个随仓库发布的提供方并不共用传输方式。`dsh-decision-jev` 向一个不接受消息、不返回文本的评估端点发起请求，因此它无法表达为 chat completion；`dsh-decision-llm` 则把普通对话模型约束到相同的答案格式上。通过该 seam 提问的调用方在两者上都能工作。

### 最小配置

加载服务，让唯一挂载的后端自动选中，或用 `provider` 固定一个提供方 id。环境变量 `$DSH_DECISION_PROVIDER` 输入同一字段，它不是另一条独立的优先级链。

```yaml
- name: '@deepseek-ai/dsh-decision'
- name: '@deepseek-ai/dsh-decision-jev'
```

| 字段 | 默认值 | 含义 |
|---|---|---|
| `provider` | （未设置） | 被固定的提供方 id；未设置时，恰好一个可用则自动选中 |
| `fallbackProvider` | （未设置） | 仅在主要提供方无法作答后才调用的提供方 id；主要调用成功时绝不会咨询它 |

生成的[配置目录](../../../docs/config-catalog.zh.md#deepseek-aidsh-decision)是每个可接受字段及其 JSDoc 的详尽真源。

### 提问

`evaluate()` 接收一个描述情境的 `state` 字符串与一张具名问题表，并在同一次往返中回答所有问题。每个答案都带有一个概率分布，因此调用方可以依据置信度设置阈值，而不是接受一个光秃秃的判定。

```text
const result = await ctx.decision.evaluate({
  state: snapshotOfThePage,
  questions: {
    element: { type: 'choice', instructions: 'Which element advances the task?', criteria: { '1': 'Buy now', '2': 'Add to cart' } },
    risk: { type: 'score', instructions: 'How destructive is this action?', criteria: ['safe', 'caution', 'destructive'] },
    done: { type: 'boolean', instructions: 'Is the task already complete?' },
  },
})
```

三种问题类型覆盖该 seam 支持的全部判断：

| 类型 | `criteria` | 答案 |
|---|---|---|
| `choice` | 选项 id → 描述映射（2 个或更多） | 获胜的 `choice`，外加覆盖每个选项的 `probabilities` |
| `score` | 按从低到高排列的等级描述（2–10） | 从 `0` 到 `criteria.length - 1` 的连续 `score`，外加覆盖各等级索引的 `probabilities` |
| `boolean` | （无） | 该陈述为真的 `probability` |

把多个问题一起提出才是预期用法：一次调用无论携带多少问题都只花一次往返，因此已经在为一次 decision 付费的消费方应捎带上其他问题，而不是发起第二次调用。

### 依据置信度设置阈值

`choice` 答案的 `probabilities` 正是使用该 seam 而非抛硬币的理由。只在阈值之上行动、低于则升权的消费方限定了自身的最坏情况：自信但错误才是需要设计的失败模式，而分布让它变得可见。

<a id="understand-the-implementation"></a>
## 理解实现

`DecisionRuntime` 是注册为 `ctx.decision` 的 Cordis 服务。它持有一个以 id 为键的提供方注册表，在接触任何提供方之前校验每个请求，并为每次调用选定一个提供方。

**校验先于选择。** 无论是否挂载了提供方，畸形请求都以 `DECISION_REQUEST_INVALID` 失败，因此调用方的 bug 绝不会表现为可用性问题。规则是结构性的：至少一个问题；`choice` 至少需要两个选项，因为单选项的选择不构成一次 decision；`score` 的评分标准包含 2 到 10 个等级，因为更细的评分标准要求的精度是答案无法承载的。

**选择是显式的或唯一的。** 配置了 `provider` 时，该提供方必须已注册且可用，否则调用会明确失败（`DECISION_PROVIDER_CONFIGURED_MISSING` / `_CONFIGURED_UNAVAILABLE`），除非同时配置了可用的 `fallbackProvider`。`fallbackProvider` 不是静默替代：服务仅在主要提供方无法作答后才调用它，主要调用成功时绝不会咨询它。未配置任何提供方时，恰好一个可用的非后备提供方会自动选中；零个以 `DECISION_PROVIDER_UNAVAILABLE` 失败，多于一个以 `DECISION_PROVIDER_AMBIGUOUS` 失败，而不是按注册顺序挑选。

**只有当主提供方无法服务一个本可回答的请求时，回退才会运行。** 主提供方抛出 `DECISION_REQUEST_REJECTED` 意味着它拒绝的是请求本身——不可用的凭据、格式错误的请求体，或超出该提供方 token 上限的状态——而回退会收到完全相同的请求，并得到完全相同的拒绝。服务改为直接上报它，使确定性的失败不必付出两次代价；当回退需要加载本地模型来作答时，这一点最为关键。该分类由提供方负责，因为只有它们能看到后端的状态码。

**注册是一次 effect。** `registerProvider()` 返回一个 disposer，并拒绝重复的 id（`DECISION_DUPLICATE_PROVIDER`）。来自被替换注册的陈旧 disposer 不会移除其后继者。

<a id="further-exploration"></a>
## 延伸阅读

- [`@deepseek-ai/dsh-decision-jev`](../decision-jev/README.zh.md) —— TypeSafe Jev 提供方，适用于持有 TypeSafe 凭据的部署。
- [`@deepseek-ai/dsh-decision-laya`](../decision-laya/README.zh.md) —— 按需本地 Laya 后备，仅在主要提供方失败后启动。
- [`@deepseek-ai/dsh-decision-llm`](../decision-llm/README.zh.md) —— 无需密钥的提供方，它约束已有的对话模型，因此该 seam 绝不是单一厂商依赖。
- [能力 seam](../../../docs/glossary.zh.md#capability-seam) —— 本包所属分组遵循的 Service Definition / Provider / Consumer 划分。

<a id="dev-note"></a>
## 开发备注

提供方选择在两个方向上都刻意保持明确。存在歧义的自动选择是错误而不是首个匹配，因为替代方案会让注册顺序悄悄变成配置面——而由一次无关重排导致的行为变化，远比一条同时点名两个候选的错误信息更难诊断。

<a id="model-experience"></a>
## 模型体验

None, as this typed-judgement service delegates all prompt construction and execution to registered providers and their callers.

#### KV Cache 影响

No direct effect; consumers that branch on decision results own any subsequent prompt changes.


<a id="known-limitations-and-deferred-work"></a>
## 已知限制与待办工作

- **没有答案会生成文本。** 每个分支按构造都返回选择、数值或概率。散文、命名与提交信息留在 `LlmService`；这是该 seam 的性质，而非待补的缺口。
- **跨调用没有缓存或批处理。** 两个消费方在同一轮次就同一 state 提问就要付两次往返。把额外问题捎带在已经运行的一次调用上，是目前唯一的共享机制。
- **没有跨提供方的校准保证。** 一个提供方给出的 `0.8` 与另一个提供方给出的 `0.8` 并不被承诺含义相同，因此针对一个提供方调好的阈值在切换后应重新检查。
- **选择是按调用而非按问题。** 一个请求无法把它的 `boolean` 问题路由到一个提供方、把 `choice` 问题路由到另一个。
