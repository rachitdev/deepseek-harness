---
description: "Jev decision 提供方：通过 TypeSafe 的直接 System One API 回答 ctx.decision 问题，返回经过校准的概率而不生成文本。"
kind: "package-reference"
---

# @deepseek-ai/dsh-decision-jev

[English](README.md) | 中文

## 概述

`dsh-decision-jev` 通过 TypeSafe 的直接 System One API 上的 Jev 回答 [`dsh-decision`](../decision/README.zh.md) 的问题。Jev 是为带类型的判断而非对话构建的：它返回带校准分布的选择、评分或概率，完全不生成文本。当部署持有 TypeSafe 凭据、并希望以比对话模型更快更省的方式得到 decision 答案时选择它。把它与 decision 服务一起挂载；它不注册面向模型的工具，只回答消费方提出的问题。

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

把插件挂载到 `dsh-decision` 旁边，并为它提供 TypeSafe 凭据。它注册一个 id 为 `jev` 的提供方，当它是唯一已挂载的可用提供方时会自动选中。

### 何时选择它

当组合持有 TypeSafe 密钥、且其 decision 对延迟或成本敏感时选择它。它比把同一个问题经由前沿对话模型路由明显更快也更省，因为该端点不做文本生成。对于无法持有 TypeSafe 凭据的部署，不要把它作为唯一提供方——改为挂载 [`dsh-decision-llm`](../decision-llm/README.zh.md)，或同时挂载两者，并用服务的 `provider` 字段固定其中一个。

### 最小配置

密钥通过凭据*引用*读取，因此它是被命名的而非内嵌的，轮换后的密钥无需重启即可在下次调用生效。

```yaml
- name: '@deepseek-ai/dsh-decision'
- name: '@deepseek-ai/dsh-decision-jev'
```

| 字段 | 默认值 | 含义 |
|---|---|---|
| `apiKeyEnv` | `TYPESAFE_API_KEY` | 指定 TypeSafe 密钥的环境变量名 |
| `apiKey` | （未设置） | 字面密钥；优先于 `apiKeyEnv`，仅供一次性本地运行使用 |
| `baseURL` | `$TYPESAFE_BASE_URL`，否则 `https://api.typesafe.ai` | TypeSafe API 端点基址 |
| `model` | `jev-latest` | TypeSafe 直接模型别名；字段未设置时 `$TYPESAFE_MODEL` 可以覆盖它 |

生成的[配置目录](../../../docs/config-catalog.zh.md#deepseek-aidsh-decision-jev)是每个可接受字段及其 JSDoc 的详尽真源。

密钥解析先取字面 `apiKey`，其次取已挂载的 `credentials` seam，最后取启动环境。上述三者中任一可能提供密钥时，提供方即报告自身可用；若在调用时三者实际都无法提供，则以 `DECISION_PROVIDER_ERROR` 明确失败——在挂载与调用之间被吊销的密钥会表现为错误，而绝不会表现为一次未认证的请求。

<a id="understand-the-implementation"></a>
## 理解实现

TypeSafe 的 System One 端点不是 chat-completions 形态的，这正是该能力是一个独立 seam、而不是又一条 `LlmService` 路由的具体原因。请求使用 Bearer 认证发往 `/v1/systemone`；JSON 请求体携带 `model`、`state` 与 `questions`。

**厂商词汇留在此包内部。** TypeSafe 用同一个键 `criteria` 命名 choice 的选项映射与 score 的有序等级，并把「为真概率」这一分支称为 `noul`。`src/types.ts` 描述该协议格式，`provider.ts` 在 `boolean` 与 `noul` 之间双向转换，因此 seam 的词汇绝不会把厂商命名泄漏给消费方。

**缺失字段是错误，而不是默认值。** 当 TypeSafe 返回的答案缺少其分布，或漏答被提出的问题时，调用失败。伪造的概率在消费方的置信度检查处与经过校准的概率无法区分，而这恰恰是该 seam 存在所要支持的那项检查。

**限流会被单独上报。** HTTP 429 抛出 `DECISION_RATE_LIMITED` 而非通用的提供方错误，因此消费方可以识别出配额耗尽并回退，而不是把该 decision 视为无法回答。取消会在每个 await 点抛出 `DECISION_ABORTED`，包括请求体读取过程中，因此被中止的调用绝不会被误报为提供方故障。

**被拒绝的请求与故障的提供方相区分。** 其余任何 4xx 都抛出 `DECISION_REQUEST_REJECTED`：不可用的密钥（401）、格式错误的请求体，或超出该端点 token 上限的 `state`（400 `max_tokens_exceeded`），都是请求自身的属性，因此 decision 服务会把它上报出来，而不是把同一个请求重试到回退提供方。5xx 仍保留 `DECISION_PROVIDER_ERROR`，因此服务端故障依然会回退。这一点在配置了 `dsh-decision-laya` 时尤其重要：无意义的回退会先加载一个本地模型，然后得到同样的拒绝。

<a id="further-exploration"></a>
## 延伸阅读

- [`@deepseek-ai/dsh-decision`](../decision/README.zh.md) —— Service Definition、问题类型与选择策略。
- [`@deepseek-ai/dsh-decision-laya`](../decision-laya/README.zh.md) —— 按需本地 Laya 后备，仅在该提供方失败后启动。
- [`@deepseek-ai/dsh-decision-llm`](../decision-llm/README.zh.md) —— 无需密钥的替代方案，可用作回退。
- `tests/gateway.e2e.ts` —— TypeSafe 直接覆盖；在缺少 `$TYPESAFE_API_KEY` 时自行跳过。

<a id="dev-note"></a>
## 开发备注

单元测试套件对 `fetch` 打桩，因此无法捕获请求格式的漂移。`tests/gateway.e2e.ts` 会调用 TypeSafe 直接服务，并在缺少密钥时自行跳过，因此 `/v1/systemone` 路径、Bearer 认证、请求体中的模型 id 与 `noul` 响应分支保有真实检查，同时默认套件不要求凭据。

<a id="model-experience"></a>
## 模型体验

None, as this decision provider calls an external evaluation endpoint that contributes nothing to the conversation model context.

#### KV Cache 影响

No effect; the evaluation call runs outside the conversation context and invalidates no prefix cache.


<a id="known-limitations-and-deferred-work"></a>
## 已知限制与待办工作

- **账户限流仍是一个独立依赖。** TypeSafe 在账户超过限额时返回 HTTP 429；提供方抛出 `DECISION_RATE_LIMITED`。在循环中逐步做 decision 的消费方需要足够的 TypeSafe 直接容量或已挂载的回退。
- **实测延迟取决于网络距离。** 应以部署所在区域对 TypeSafe 直接端点的实测值为准，并在更改端点、账户层级或模型后重新测量，而不是沿用 Vercel 网关的数据。
- **不假定其准确性与前沿模型相同。** 独立测试显示 Jev 遗漏过一个前沿模型捕获到的缺陷。消费方应依据返回的概率设置阈值并在低于阈值时升权，而不是把任何单一答案当作权威。
- **答案不携带文本**，因此该提供方永远无法解释一次 decision。当消费方需要向用户给出理由时，必须另想办法产生。
- **来源页面派生的 state 是不可信输入。** 把抓取内容送入 `state` 的消费方必须依据自己的快照构建选项集合，绝不依据页面提供的文本，否则页面就可能影响它被评判时所用的选项列表。
