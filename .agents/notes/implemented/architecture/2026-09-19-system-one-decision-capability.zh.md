# Agent Note: System One decision 能力

Status: implemented

[English](2026-09-19-system-one-decision-capability.md) | 中文

## 问题

DeepSeek Harness 内部的各种判断历来要花掉一整次 LLM 轮次：选择下一个浏览器动作、判断某次 shell 调用是否危险、为某个轮次挑选模型、在 compaction 时判断哪些工具结果仍然重要、以及给检索到的记忆打分，全都是分类问题，但 harness 过去用前沿模型生成文本来回答它们。每一次都要花掉数秒延迟和数分钱的 token，而答案的全部信息量只是一个标签和一份置信度。

代价在循环里最为沉重。一个 ego-browser 会话点击二十个页面，就要花二十次模型轮次来决定点哪个元素——而这项工作的产出只是 harness 早已持有的某个列表里的一个索引。

TypeSafe 的 Jev 是一个 System One 模型：它接收一个 state 加上带类型的问题，返回带校准概率的带类型答案，不生成任何字符串。它回答的是「这些选项里选哪个、有多强、有多确定」。把它挂到 `LlmService` 上是不可行的，因为两个随仓库发布的提供方并不共用传输方式：TypeSafe 的 System One 端点不接受消息、不返回文本，因此无法表达为 chat completion；而 LLM 回退则把普通 completion 约束到相同的答案格式上。

## 决策

我们新增了一条专用的 **decision 能力 seam**——`packages/decision/`——通过五个包完整覆盖能力 seam 的全部三种角色：

1. **Service Definition**（`@deepseek-ai/dsh-decision`）：持有 `ctx.decision`、问题校验、提供方选择与错误词汇。它定义封闭的问题联合（`choice`、`score`、`boolean`）与答案联合，在选择提供方之前校验请求，并在出现歧义或不可用时明确失败。
2. **Service Provider —— Jev**（`@deepseek-ai/dsh-decision-jev`）：通过 Bearer 凭据引用 `TYPESAFE_API_KEY` 访问 TypeSafe 的直接 System One 端点（`https://api.typesafe.ai/v1/systemone`），并在请求体中携带模型 `jev-latest`。`jev-latest` 是会移动的别名，而不是固定版本；配置字段未设置时 `$TYPESAFE_MODEL` 可以覆盖它。它把 seam 的 `boolean` 问题转换为 TypeSafe 的 `noul` 词汇，再把校准后的答案映射回来。
3. **Service Provider —— Laya 后备**（`@deepseek-ai/dsh-decision-laya`）：仅在主要提供方无法作答后才启动缓存的本地 Laya `typed-decisions` 检查点。Worker 绑定回环端口 `58085`，以 `local_files_only` 加载权重，并在空闲超时后退出，因此健康的 Jev 路径不占用 Laya 内存。
4. **Service Provider —— LLM 回退**（`@deepseek-ai/dsh-decision-llm`）：把已注册的 `LlmService` 对话模型约束为带类型的 JSON 回复，并在模型输出边界校验每个字段。它不需要任何 decision 专用凭据，从而确保该 seam 绝不是单一厂商依赖。
5. **消费方**（`@deepseek-ai/dsh-decision-consumers`）：九个由 decision 驱动的操作与绑定，消费 `ctx.decision`：
   - `decideEgoStep`：自主浏览器导航，评估候选动作、检测完成／受阻，并在点击或升权之前进行校准过的置信度把关。
   - `assessToolRisk`：批准前的工具调用风险评分（`read-only`、`safe-workspace-mutation`、`destructive-or-external`、`system-critical`）与权限建议（`allow`、`prompt`、`deny`）。
   - `routeTurnModel`：轮次级别的复杂度路由，在快速／低价档（`deepseek-chat`）与前沿档（`deepseek-reasoner`）之间选择。
   - `filterRetrievalCandidates`：在一次 decision 调用中并行评估记忆与文档检索分片的相关性评分与降噪。
   - `scoreCompactionItems`：transcript 保留优先级，把条目划分为保留、丢弃与摘要三组并计入字节数。
   - `detectLoopStall`：基于近期动作序列的概率化循环停滞与重复检测，建议 `continue`、`warn` 或 `interrupt`。
   - `evaluateSubagent`：对委派的 subagent 执行进行评估，给出带类型的判定（`accept`、`retry`、`escalate`）与可执行的反馈建议。
   - `selectVerificationGates`：推送前 git diff 分诊，把变更文件映射到最小充分检查套件（`doc_only`、`lint_and_unit`、`full_coverage`、`full_with_e2e`）。
   - 实时模型 failover：拦截已配置的 `agent/request-error` 失败，并发探测已授权的提供方／模型路由，请求 JEV 评判代码有效性与 token 效率、延迟，覆盖同一步骤的下一次 `agent/request`，并返回 `{ kind: 'retry' }`。随之产生的 `request/header` 就是持久的路由记录。接管后失败的路由在该步骤内保持排除；取消与 dispose 会中止并排空进行中的探测。

`dsh-llm-retry` 在安排同路由退避之前，先委托给下游的专门恢复。这使通用重试插件无论监听器注册顺序如何都保持为回退：仲裁不可用、没有健康探测、或获胜者被判决方评为代码损坏时，都会走到普通重试，而不是终止该轮次。

### 评估过的提供方路由

| 路由 | 提供方 | 端点 | 成本／限制 | 状态 |
|---|---|---|---|---|
| **TypeSafe 直连** | TypeSafe AI | `https://api.typesafe.ai/v1/systemone` | 账户限流；直接 Bearer 凭据 | **本地已选用**；实时 e2e 为 1100 ms，8/8 突发调用成功 |
| **本地 Laya 后备** | ConvAI Laya | 回环 `http://127.0.0.1:58085/v1/systemone` | 零单次费用；约 20 秒冷加载，空闲退出 | Jev 失败后按需启动；`typed-decisions` 检查点 |
| **本地 LLM 回退** | 任意已注册的 `llm` 路由 | harness 内 `ctx.llm.stream` | 零新增凭据；可移植性下限 | 已实现并测试；base 组合未挂载 |
| **Vercel AI Gateway** | TypeSafe 经 Vercel | `https://ai-gateway.vercel.sh/v4/ai/evaluation-model` | 输入 $0.042/M，输出免费；免费层受限流 | TypeSafe 直连开放后替换的引导路由 |
| **Cloudflare AI Gateway** | Cloudflare | 通用网关 | 需要 Workers Paid 与账户余额 | 已否决 |

### 实测情况

本机访问 TypeSafe 直接端点时，首次调用耗时 1243 ms，之后的往返为 345–460 ms；连续八次成功调用的中位数为 406 ms。真实端点 e2e 用时 1100 ms，八次突发调用未出现 HTTP 429。这些数字建立本机基线，并不保证账户的持续速率上限。

提供方把 TypeSafe 直接端点的 HTTP 429 映射为 `DECISION_RATE_LIMITED`，从而允许消费方回退。本机 Web 组合把 `dsh-decision-laya` 挂载为 `fallbackProvider`，并且不挂载 `dsh-decision-llm`，因此 JEV 仲裁不可用时，会先尝试按需 Laya worker，然后再把模型恢复委托给 `dsh-llm-retry`。

## 曾考虑的替代方案

**把 decision 折叠进 LlmService，作为一个辅助模型用途。** 一个适配器或诸如 `purpose: 'decision'` 的用途标记就能让既有路由承担 decision。该方案被否决，因为 TypeSafe 的 System One 端点在结构上不是 chat-completions 形态：它不接受对话消息、不做流式、也不产生文本。把评估调用强行塞进 LLM 适配器契约，会需要伪造的 `Message` 封装与打桩的 `StreamChunk` 生成器，从而违反适配器协议。

**通过 TypeSafe JavaScript SDK 调用直接端点。** 该 SDK 提供带类型的客户端与自动重试行为。此方案在运行时代码中被否决，因为直接端点只需要一次带认证的 JSON 请求，一份小型原生 fetch 提供方即可实现协议，无需给核心运行时包增加 SDK 依赖。

**在 TypeSafe 前保留中间网关。** Vercel 与 Cloudflare 都能代理第三方模型流量，但中间层会增加一套凭据、协议转换、配额与网络跳点。本机已经具备 TypeSafe 直连权限，因此原生端点是更小的运营路径。

**把模型 failover 放进 `dsh-llm-retry` 内部。** 重试插件持有的是提供方策略退避，而不是候选授权、canary 生成或 decision 提供方选择。把接管折叠进去会让通用传输恢复依赖于可选的 decision 能力。改为使用 request-error waterfall，让重试监听器先委托出去，并保持为同路由回退。

## 后果

这带来了什么：
- **可移植性与韧性：** decision 与任何单一提供方解耦。harness 对 Jev（快速、经过校准）、按需本地 Laya 后备（冷加载后零单次费用）或 LLM 回退（无需密钥、通用）的行为完全一致。
- **经过校准的把关：** 消费方可以设置量化的置信度阈值（例如 `probability > 0.85`），而不是做二元猜测，并在置信度低时升权为完整的 LLM 轮次，从而限定下行风险。
- **延迟下降：** 本机对 TypeSafe 的热调用实测为 345–460 ms，而首次调用与隔离 e2e 仍约为 1.1–1.2 秒；两者都避免了为分类工作支付完整的 3–5 秒 chat completion。
- **十一条已实现的消费路径：** 十个带类型函数覆盖浏览器自动化、工具批准、模型路由、记忆检索、上下文 compaction、循环防护、subagent、推送前验证、技能目录路由与网页内容安全；实时模型 failover 是唯一具备自动运行时绑定的消费方。

这付出了什么：
- **需要维护五个包：** `@deepseek-ai/dsh-decision`、`@deepseek-ai/dsh-decision-jev`、`@deepseek-ai/dsh-decision-laya`、`@deepseek-ai/dsh-decision-llm` 与 `@deepseek-ai/dsh-decision-consumers`。
- **直连账户限流：** 高频率循环仍会受 TypeSafe HTTP 429 约束，需要足够的账户容量或已挂载的回退；八次成功调用并不能证明持续速率上限。
- **严格限定为分类：** 不能用于文本生成、代码编辑或命名。
- **Failover 扇出：** 一次符合条件的请求失败可能发出若干个有界的 canary 调用。因此候选授权同时决定了哪些提供方可能收到最近的用户任务。

## 验证

- `packages/decision/decision/tests/decision.spec.ts`：25 个单元测试，覆盖提供方注册、disposer 幂等、替换后的稳定性、重复拒绝、显式与环境变量配置的选择、歧义选择失败、不可用失败、问题校验与按需后备。
- `packages/decision/decision-jev/tests/decision-jev.spec.ts`：41 个单元测试，覆盖问题／答案转换、criteria 映射、可用性检查、fetch 请求头组装、限流错误映射、取消转发、凭据解析与 `$TYPESAFE_MODEL` 覆盖。
- `packages/decision/decision-laya/tests/decision-laya.spec.ts`：12 个单元测试，覆盖问题／答案转换、健康 worker 评估、HTTP 错误、取消映射与 fiber 释放。
- `packages/decision/decision-llm/tests/decision-llm.spec.ts`：27 个单元测试，覆盖提示词渲染、响应解析、围栏代码块剥离、schema 校验、选项／评分标准边界强制与流式组装。
- `packages/decision/decision-consumers/tests/*.spec.ts`：九个消费方的单元覆盖，包括全部探测失败、代码损坏判定、仍然接管的近乎均等排名、失败路由排除、allowed-model 取值来源，以及取消／dispose 的静默退出。
- `packages/decision/decision-consumers/tests/loader.spec.ts`：一次真实的 app/boot 与 Cordis Loader 组合，证明模型 A 上的 429 会产生并行探测、一次 decision 调用、同一 Agent 步骤内对模型 B 的重试，以及记录在持久 `request/header` 与 assistant 消息 `model` 字段中的模型 B。
- `packages/llm/llm-retry/tests/retry.spec.ts`：把专门恢复挂载在通用重试之后，证明下游的 `{ kind: 'retry' }` 在没有同路由重试事件的情况下获胜。
- `packages/decision/decision-consumers/tests/spike-10-step-browser.spec.ts`：10 步自主导航轨迹，逐步记录元素选择、置信概率、风险评分、完成情况与升权行为（平均置信度 86.7%）。
- `packages/decision/decision-jev/tests/gateway.e2e.ts`：针对 TypeSafe 直接 System One 端点的实时集成测试；缺少 `TYPESAFE_API_KEY` 时自行跳过；本机以 HTTP 200 在 1100 ms 通过。
