# Agent Note: Jev 召回过滤——对记忆召回做相关性与存储型注入筛查

Status: implemented

[English](2026-09-23-jev-recall-filter.md) | 中文

## 问题

OpenViking 记忆插件会在每个 Agent 步骤追加一个 `<openviking-context>` 召回块。它的向量排序几乎给每个条目都打出 0.77–0.93 的分数，因此召回块装的是嵌入空间里最近的内容：在本机 40 条已记录召回上，TypeSafe Jev 把 419 个条目中的 255 个（占召回字符的 54%）评为低于「tangential」。召回也是一条未经筛查的存储型提示注入通道：从抓取网页或工具输出捕获进记忆的文本，会在之后的会话中以看似可信的上下文返回。`filterRetrievalCandidates` 已存在但没有调用方，而只有 `web_fetch` 的结果会做注入筛查。

## 决策

在 `@deepseek-ai/dsh-decision-consumers` 中新增一个实时消费方，由 `recallFilter.enabled` 启用：

- `screenRecallItems` 为每个条目提出一个 `rel_<n>` Score 问题（0..3 量表 `irrelevant`/`tangential`/`useful`/`essential`），并在 `screenInjection` 开启时提出一个 `inj_<n>` Boolean 问题，按 `batchSize` 并发分批。state 包含用户请求、上一条 assistant 回复（`maxContextChars`），以及每个条目的 uri 与有界正文。
- `installRecallFilter` 注册一个 `agent/pre-step` 监听器，按 `source.plugin`/`source.form` 匹配召回消息，解析 `<memory uri>` 元素，并用重新渲染的块替换该消息：低于 `minRelevance` 的条目被省略并附一行计数说明，不低于 `injectionThreshold` 的条目变为只保留 uri 的 `screened="withheld: …"` 存根。
- 任何提供方失败、畸形答案或 `timeoutMs` 超时都返回原始召回并记录警告；召回绝不会因为 Jev 慢或不可用而丢失。

以下选择由针对实时 TypeSafe 的实测确定：

- **不使用 `keep` Boolean。** 在 151 个条目上它与 Score 的相关系数为 0.96，从未保留 Score 丢弃的条目，却丢弃了 15 个 Score 评为 useful 及以上的条目。只由 Score 决定。
- **默认 `minRelevance` 0.5、`injectionThreshold` 0.7。** 植入攻击得分 0.75–0.99；良性安全笔记与全部 419 个已记录条目最高 0.16。
- **上一条回复作为上下文。** 四分之一的已记录查询是「go ahead」这类跟进。把上一条回复放进 state 后，短查询上 233 个条目中有 11 个原本仅凭请求被评为无关的条目被挽回（例如「go ahead test it out」对应的代码签名偏好），因此默认开启上下文。
- **拒绝拆分。** TypeSafe 网关防火墙对包含 `curl … | sh` 这类攻击形态文本的 state 返回 HTTP 403，表现为 `DECISION_REQUEST_REJECTED`，使整批失败。被拒绝的批次逐条重试，单独仍被拒绝的条目默认被扣留（`withholdUnscreenable`）：实测只有植入攻击被拒。
- **监听器顺序。** OpenViking 在 `prepend` 监听器里、于自身 `next()` 之后追加召回，因此过滤器必须位于最外层。Loader 用 `Promise.allSettled` 应用条目，跨 bundle 的注册顺序不确定；过滤器在每次 `agent/session-start` 时重新注册其 `prepend` 监听器，此时所有启动期注册都已完成。去掉重新注册会使顺序测试失败。

## 备选方案

- **修改 vendored 的 OpenViking 插件去调用过滤器**——否决：vendor 包不在本地打补丁，而且过滤器会因此只服务一个记忆来源。
- **提高 OpenViking 的 `scoreThreshold`**——否决：向量相似度在相关与无关条目上同样平坦（0.77–0.93），任何阈值都无法区分它们；它对注入也毫无作用。
- **Jev 出错时失败即关闭**——不作为默认：Laya 冷启动或网络错误会让一个轮次失去全部记忆。成功路径（实测中的常见情况）仍会捕获注入。
- **为注入单独发一次 decision 请求**——否决：额外问题不增加延迟，额外往返才增加，因此两个问题共用一次请求。

## 影响

- 每个带召回的步骤增加一次 decision 往返（经 TypeScript 路径实测 p50 0.43 秒、p95 0.53 秒；较大召回的 p95 最高 0.85 秒），并从模型请求中去掉约一半的召回字符。
- 模型看到的是过滤后的块，session 日志记录的也是它；被扣留的条目仍可通过记忆搜索按 uri 找到。
- 档案消息（`form: instructions`）不经筛查。

## 验证

- `tests/recall-block.spec.ts`、`tests/recall-screen.spec.ts` 与 `tests/recall-filter.spec.ts` 中的单元与实时监听器测试覆盖解析、分批、拒绝拆分、失败放行、超时、禁用模式、dispose 与上下文；三个源文件的语句与分支覆盖率均为 100%。
- `tests/recall-filter-loader.spec.ts` 启动一个 Loader 组合，把召回来源列在 `decision-consumers` 之后，并断言持久 session 日志中是筛查后的块。
- 经 `JevDecisionProvider` 的实时 A/B：40 条已记录召回，加入 20 条植入攻击与 20 条良性笔记：18 条攻击按分数被扣留，2 条被网关拒绝并扣留，0 条良性或已记录条目被扣留，0 次调用失败。
- 在隔离的 `jev-recall` profile 中结合实时 OpenViking 与 TypeSafe 运行一次，得到正确答案，保留 3 个条目、省略 12 个。
