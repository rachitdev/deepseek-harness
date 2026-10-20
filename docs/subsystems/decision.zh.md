# Decision

[English](decision.md) | 中文

decision [seam](../glossary.zh.md#capability-seam) 在 `ctx.decision` 服务上只承载**一个操作**（`evaluate`），由 Service Definition（[dsh-decision](../../packages/decision/decision)，即 `ctx.decision` 加上提供方注册表）、Service Provider（[dsh-decision-jev](../../packages/decision/decision-jev)、[dsh-decision-laya](../../packages/decision/decision-laya)、[dsh-decision-llm](../../packages/decision/decision-llm)）与带类型的[消费方](../../packages/decision/decision-consumers)共同组成。decision 是**一个可选能力**，不属于 agent loop 主干，因此其词汇记录在本页而非 [core.zh.md](core.zh.md)。更换提供方不会改变消费方提问的方式。

Source: [`packages/decision/decision/src/types.ts`](../../packages/decision/decision/src/types.ts)

## 该能力的用途

一次 decision 是对非结构化状态的判断，其答案是选择、评分或真值概率——绝不是文本。「这些页面元素中哪一个推进任务」、「这次工具调用破坏性有多大」、「这篇检索到的文档是否相关」。这类问题模型可以回答，但不需要模型写出任何内容，而 harness 自身无法计算它们。

把这些判断留在 `LlmService` 之外并非风格上的拆分。随仓库发布的提供方并不共用同一传输方式：`dsh-decision-jev` 向 TypeSafe 的 System One 端点发起请求，该端点不接受消息也不返回文本；`dsh-decision-laya` 通过仅在主要提供方失败后启动的本地 Laya 检查点回答同样的带类型问题；`dsh-decision-llm` 则把普通对话模型约束到相同的答案格式上。针对该 seam 编写的消费方在它们上都能运行。反过来同样成立：散文、命名与提交信息留在 `LlmService`，因为 decision 答案按构造不携带任何字符串。

## 问题与答案

一个请求携带一个 `state` 字符串与一张具名问题表，所有问题在同一次往返中得到回答。把多个问题一起提出才是预期用法——计费按调用而非按问题计算——因此已经在为一次 decision 付费的消费方应捎带上其他问题，而不是发起第二次调用。

```ts type-equiv
/**
 * One question put to a decision provider. A CLOSED discriminated union owned by `dsh-decision`:
 * providers translate every arm and consumers `switch` on `type` ending in `assertNever`, so a new
 * arm is a coordinated change across known packages rather than a plugin extension.
 */
type DecisionQuestion =
  | DecisionChoiceQuestion
  | DecisionScoreQuestion
  | DecisionBooleanQuestion
```

```ts type-equiv
/**
 * Pick one option from a named set. `criteria` maps each option id to the description the model
 * judges against; option ids are the consumer's own vocabulary and come back verbatim in the answer.
 */
interface DecisionChoiceQuestion {
  readonly type: 'choice'
  /** What the model is being asked to decide. */
  readonly instructions: string
  /** Option id to its description. At least two entries; a one-option choice has no decision to make. */
  readonly criteria: Readonly<Record<string, string>>
}
```

```ts type-equiv
/**
 * Place the state on an ordered rubric. `criteria` lists level descriptions from lowest to highest;
 * the answer may land between levels, so a score is continuous rather than an index.
 */
interface DecisionScoreQuestion {
  readonly type: 'score'
  /** What the model is being asked to rate. */
  readonly instructions: string
  /** Ordered level descriptions, lowest first. Between 2 and 10 entries. */
  readonly criteria: readonly string[]
}
```

```ts type-equiv
/** Judge whether one statement holds. The answer is the probability that it is true. */
interface DecisionBooleanQuestion {
  readonly type: 'boolean'
  /** The statement whose truth is being judged. */
  readonly instructions: string
}
```

```ts type-equiv
/**
 * One answer from a decision provider, keyed identically to its question. A CLOSED discriminated
 * union: the `type` always matches the question's `type`, so a consumer that asked a `choice` may
 * narrow on that arm.
 */
type DecisionAnswer =
  | DecisionChoiceAnswer
  | DecisionScoreAnswer
  | DecisionBooleanAnswer
```

```ts type-equiv
/** The selected option plus the full distribution that produced it. */
interface DecisionChoiceAnswer {
  readonly type: 'choice'
  /** The winning option id, always a key of the question's `criteria`. */
  readonly choice: string
  /** Probability per option id; keys match the question's `criteria`, values sum to approximately 1. */
  readonly probabilities: Readonly<Record<string, number>>
}
```

```ts type-equiv
/** A position on the rubric plus the distribution over levels. */
interface DecisionScoreAnswer {
  readonly type: 'score'
  /** Continuous position, `0` to `criteria.length - 1`; may fall between levels. */
  readonly score: number
  /** Probability per level index, keyed by its stringified index. */
  readonly probabilities: Readonly<Record<string, number>>
}
```

```ts type-equiv
/** The probability that the statement holds. */
interface DecisionBooleanAnswer {
  readonly type: 'boolean'
  /** Probability in `[0, 1]` that the statement is true. */
  readonly probability: number
}
```

```ts type-equiv
/**
 * What a decision provider is asked to evaluate: one state, and every question asked about it.
 * Questions are evaluated together because a provider prices and paces one call, not one question —
 * an extra question costs tokens but little additional time.
 */
interface DecisionRequest {
  /** The situation being judged, as plain text the provider reads verbatim. */
  readonly state: string
  /** Questions keyed by a consumer-chosen name; answers come back under the same keys. */
  readonly questions: Readonly<Record<string, DecisionQuestion>>
}
```

```ts type-equiv
/** The answers to one {@link DecisionRequest}, keyed identically to its `questions`. */
interface DecisionResult {
  /** One answer per question, under the request's own keys. */
  readonly answers: Readonly<Record<string, DecisionAnswer>>
  /** Id of the provider that answered, for session-log attribution. */
  readonly providerId: string
}
```

```ts type-equiv
/**
 * A decision-capable backend. Registered with `ctx.decision.registerProvider`. `id` is a stable
 * string, unique within the seam.
 */
interface DecisionProvider {
  readonly id: string
  /** Cheap local usability check; must not make network calls. */
  available(): boolean
  /** Answer every question in one request; honor `signal` for cancellation. */
  evaluate(request: DecisionRequest, signal?: AbortSignal): Promise<DecisionResult>
}
```

对 `choice` 而言，`criteria` 携带选项 id → 描述的映射；对 `score` 而言，它按从低到高的顺序列出等级描述；`boolean` 不需要它。`score` 答案在 `0..criteria.length - 1` 上连续，因此可以落在等级之间，其 `probabilities` 以等级索引为键。

每个答案都携带一个分布。这正是该 seam 的要点，而非实现细节：在置信度阈值之上行动、低于阈值则升权的消费方限定了自身的最坏情况，因为「自信但错误」才是需要设计的失败模式，而分布让它变得可见。

## 请求校验

服务在选定提供方之前先校验请求，因此无论是否挂载了提供方，畸形请求都以相同方式失败，调用方的 bug 也绝不会表现为可用性问题。规则是结构性的：一个请求至少携带一个问题；`choice` 至少提供两个选项，因为单选项的选择不构成一次 decision；`score` 的评分标准包含 2 到 10 个等级，因为更细的评分标准要求的精度是答案无法承载的。任何违例都抛出 `DECISION_REQUEST_INVALID`。

## 提供方可用性与选择

提供方依据可在本地检查的状态报告 `available()`——可解析的凭据、可解析的端点、非空的模型 id——而不执行 I/O。

选择是显式的或唯一的，绝不按位置。设置服务的 `provider` 字段后，该提供方必须已注册且可用，否则调用失败（`DECISION_PROVIDER_CONFIGURED_MISSING`、`DECISION_PROVIDER_CONFIGURED_UNAVAILABLE`），除非同时配置了可用的 `fallbackProvider`。`fallbackProvider` 仅在主要提供方无法作答后才调用——缺失、不可用、被限流或失败——主要调用成功时绝不会咨询它。取消（`DECISION_ABORTED`）不会触发后备。未固定任何提供方时，恰好一个可用的非后备提供方会自动选中；零个抛出 `DECISION_PROVIDER_UNAVAILABLE`；多于一个抛出 `DECISION_PROVIDER_AMBIGUOUS`，而不是按注册顺序解析。

## 错误

| 代码 | 抛出条件 |
|---|---|
| `DECISION_REQUEST_INVALID` | 请求违反上述任一结构性规则 |
| `DECISION_DUPLICATE_PROVIDER` | 某个提供方 id 被注册两次 |
| `DECISION_PROVIDER_CONFIGURED_MISSING` | 被固定的提供方未注册 |
| `DECISION_PROVIDER_CONFIGURED_UNAVAILABLE` | 被固定的提供方已注册但不可用 |
| `DECISION_PROVIDER_UNAVAILABLE` | 没有已注册的可用提供方 |
| `DECISION_PROVIDER_AMBIGUOUS` | 有多个可用提供方且未固定任何一个 |
| `DECISION_PROVIDER_ERROR` | 提供方失败，或返回的答案不满足所提的问题 |
| `DECISION_RATE_LIMITED` | 提供方配额耗尽；消费方可以回退 |
| `DECISION_ABORTED` | 调用方的 `AbortSignal` 已触发 |

返回的答案缺失其分布，或漏答被提出的问题时，提供方会抛出 `DECISION_PROVIDER_ERROR`，而不是让缺口被填补。在消费方的置信度检查处，伪造的概率与经过校准的概率无法区分，而这恰恰是该 seam 存在所要支持的那项检查。

## 消费方

`dsh-decision-consumers` 拥有可复用的分类函数与一个可选的 Agent 恢复绑定。该绑定处理已配置的 `agent/request-error` 代码，并发探测已授权的提供方／模型路由，并请求选定的 decision 提供方在健康 worker 中做出选择。接管要求判决方把获胜者的代码有效性评为高于 `broken-or-hallucinated`；获胜选项概率作为遥测上报而不作为门槛，因为它衡量的是判决方在探测已证明可达的各路由之间区分得有多强。选择结果会覆盖下一次 `agent/request` 并返回 `{ kind: 'retry' }`；普通的 `request/header` 记录所用的路由。无法得出结论的仲裁会委派给下一个恢复监听器，通常是通用的同路由重试。

该绑定在存在已启用的 `subagent-model-selection.allowedModels` 设置时读取它，否则使用其配置的回退候选。探测收到有界的最近一条持久 user 消息。取消与插件 dispose 会中止并排空每一个活动探测，接管后失败的路由在该 Agent 步骤内保持排除。每个失败路由集合只仲裁一轮，因此一个步骤内重复的同路由失败会复用已结算的结果。[包参考](../../packages/decision/decision-consumers/README.zh.md#live-model-failover)拥有配置与模型成本细节。

## 信任与不可信状态

`state` 常由 harness 无法控制的内容构建——抓取的页面、工具结果、文件。消费方必须依据自己的快照构建选项集合，而绝不依据来源提供的文本，这样内容就无法扩展或重命名被评判的选项。该 seam 不强制这一点：它无法区分消费方的选项 id 与页面建议的 id，因此这项义务落在每个消费方身上。

<!-- BEGIN GENERATED cordis-surface (gen-cordis-catalog.ts) — do not edit between markers -->

<a id="cordis-surface"></a>

## Cordis API

Generated from source by `scripts/gen-cordis-catalog.ts` (verified fresh by `pnpm run verify-cordis-catalog` in doc-sync; regenerate with `pnpm run gen-cordis-catalog`) — the language sides differ only in locale-specific paired document paths. Signature blocks use a `ts cordis-catalog` fence and keep the original source JSDoc; dispatch modes are defined in the [primer](../cordis-primer.zh.md#dispatch-modes), and the framework-inherited `ctx` API lives in [cordis-api/inherited.md](../cordis-api/inherited.md).

<a id="ctxdecision--decisionruntime"></a>

### `ctx.decision` — `DecisionRuntime`

The decision service. Registered as `ctx.decision` (one instance per context).

Selection semantics (resolved at execution time, never order-dependent):

- A configured id that is registered and `available()` → that provider.
- A configured id not registered → `DECISION_PROVIDER_CONFIGURED_MISSING`, unless `fallbackProvider` is registered and usable.
- A configured id registered but unavailable → `DECISION_PROVIDER_CONFIGURED_UNAVAILABLE`, unless `fallbackProvider` is registered and usable.
- A configured fallback id is invoked only after the primary cannot answer, never on a successful primary call.
- No id configured, exactly one registered usable non-fallback provider → that provider.
- No id configured, multiple usable non-fallback providers → `DECISION_PROVIDER_AMBIGUOUS`.
- No id configured, no usable provider → `DECISION_PROVIDER_UNAVAILABLE`.

```ts cordis-catalog
/**
 * Register a decision provider.
 * @param provider - the backend to register; its `id` must be unique within the seam.
 * @returns a disposer that removes this registration.
 */
registerProvider(provider: DecisionProvider): () => void

/**
 * Answer every question in one request through the selected provider, or trigger the fallback
 * provider on-demand if the primary fails to respond.
 * @param request - the state and the questions asked about it.
 * @param signal - optional cancellation for the provider call.
 * @returns the answers, keyed identically to `request.questions`.
 */
async evaluate(request: DecisionRequest, signal?: AbortSignal): Promise<DecisionResult>
```

Source: [`packages/decision/decision/src/index.ts`](../../packages/decision/decision/src/index.ts)
<!-- END GENERATED cordis-surface -->
