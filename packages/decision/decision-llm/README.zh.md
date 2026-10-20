---
description: "由 LLM 支撑的 decision 提供方：把已有对话模型约束为带类型的 JSON 回复来回答 ctx.decision 问题，因此 decision seam 无需专用凭据。"
kind: "package-reference"
---

# @deepseek-ai/dsh-decision-llm

[English](README.md) | 中文

## 概述

`dsh-decision-llm` 用组合中已有的模型回答 [`dsh-decision`](../decision/README.zh.md) 的问题：把普通对话模型约束为带类型的 JSON 回复，并校验返回的内容。它不需要 decision 专用凭据，因此每个基于 decision seam 构建的消费方在普通的 harness 安装上都能工作，该 seam 也绝不会成为单一厂商依赖。把它用作默认提供方、作为 [`dsh-decision-jev`](../decision-jev/README.zh.md) 旁边的回退，或作为在任何网关凭据存在之前开发 decision 消费方的方式。

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

把插件挂载到 `dsh-decision` 旁边，并把它指向一条已注册的 LLM 路由。它注册一个 id 为 `llm` 的提供方。

### 何时选择它

当部署有一条可用的 LLM 路由、且没有专用 decision 凭据时选择它，并搭配一个便宜、快速的模型而非对话主模型：问题很小，回复也仅几十个 token，因此高级模型在这里收益甚微。把它与 `dsh-decision-jev` 一起挂载时，需要用服务的 `provider` 字段固定其中一个，因为 decision 服务拒绝在两个可用提供方之间猜测。

### 最小配置

该路由是必填而非猜测的，因为哪个模型承担 decision 因部署而异，猜错会在调用时失败而不是在加载时失败。

```yaml
- name: '@deepseek-ai/dsh-decision'
- name: '@deepseek-ai/dsh-decision-llm'
  config:
    provider: deepseek
    model: deepseek-chat
```

| 字段 | 默认值 | 含义 |
|---|---|---|
| `provider` | （必填） | 承担 decision 的已注册 LLM 提供方路由 |
| `model` | （必填） | 该路由上的确切模型 id |
| `maxTokens` | `1024` | JSON 回复的上界 |

生成的[配置目录](../../../docs/config-catalog.zh.md#deepseek-aidsh-decision-llm)是每个可接受字段及其 JSDoc 的详尽真源。

<a id="understand-the-implementation"></a>
## 理解实现

该提供方把每个请求渲染为一条陈述所需 JSON 格式的系统提示词，加上一个描述 state、每个问题、以及——对 `choice` 而言——确切选项 id、对 `score` 而言——答案必须使用的编号等级的 user 轮次。随后它流式执行一次模型调用并组装文本。

**回复是模型输出边界，因此每个字段都会被校验。** 对话模型可以输出任何内容，因此解析器会对照产生该答案的问题来检查每个答案：`choice` 必须给出该问题实际提供的某个选项，`score` 必须落在 `0..criteria.length - 1` 之内，每个概率都必须是 `[0, 1]` 内的有限数，分布必须覆盖每个选项或等级。其他任何情况都以 `DECISION_PROVIDER_ERROR` 失败，而不会像已经过校准那样抵达消费方的置信度检查。JSON 外层的 Markdown 围栏代码块是被容忍的，包括未闭合的围栏，因为模型经常加上它们，而剥掉一个不付出任何代价。

<a id="further-exploration"></a>
## 延伸阅读

- [`@deepseek-ai/dsh-decision`](../decision/README.zh.md) —— Service Definition、问题类型与选择策略。
- [`@deepseek-ai/dsh-decision-jev`](../decision-jev/README.zh.md) —— 为持有 TypeSafe 凭据的部署提供的更快、更省的提供方。
- [`@deepseek-ai/dsh-decision-laya`](../decision-laya/README.zh.md) —— 按需本地 Laya 后备，仅在主要提供方失败后启动。

<a id="dev-note"></a>
## 开发备注

校验刻意拒绝修复接近命中的情况——越界的评分不会被截断到范围内，缺失的选项也不会用零补齐。修复会把一个误解了问题的模型转变成看似自信的答案，而这正是该 seam 的消费方在下游无法检测的唯一失败。

<a id="model-experience"></a>
## 模型体验

None, as this decision provider runs auxiliary model calls whose prompts are built and discarded locally.

#### KV Cache 影响

No effect; the auxiliary call runs on its own message array and does not touch the conversation prefix cache.


<a id="known-limitations-and-deferred-work"></a>
## 已知限制与待办工作

- **概率是声明出来的，而非校准过的。** 对话模型报告的数值读起来像概率，却不保证它们的行为也像概率，而且模型倾向于向 `0` 与 `1` 取整。针对该提供方调好的置信度阈值，在切换到经过校准的提供方之后应重新检查，反之亦然。
- **畸形回复不重试。** 返回无法解析的 JSON 的模型会让调用失败，而不会被再问一次；希望重试的消费方必须自行实现。
- **延迟与成本跟随所选模型。** 该提供方是可移植性下限，而不是快速路径：在这里回答同一个问题要花费一次完整的模型轮次。
- **回复必须装进 `maxTokens`。** 携带许多问题、且选项映射很大的请求可能超出该上界并被截断成无法解析的 JSON；请提高上界或拆分请求。
