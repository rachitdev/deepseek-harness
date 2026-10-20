# Agent Note：compaction 批处理、指令偏离建议与请求拒绝路由

Status: implemented

[English](2026-09-21-jev-ecosystem-scale-and-rejection-routing.md) | 中文

## Problem

把六个独立的 TypeSafe Jev 项目（`jev-review`、`foreman`、`typesafe-computer-use`、`jev-search`、`jev-ultrafast`、`fast-jev-compaction`）与本仓库的 decision seam 做对照扫描后，发现三个缺陷和一个缺失的消费方。该 seam 已经实现了这些项目展示的十一个模式中的九个，因此价值在于补齐缺口，而不是引入新能力。

从 Noida 对 `api.typesafe.ai` 的实测确立了这些修复所依赖的性质：回答延迟取决于请求数量，而不是问题数量。一个问题和四十个问题都在约 0.95 秒内返回（1 个问题 486 个输入 token，40 个问题 1257 个）。额外的问题头基本免费，额外的请求则不是。

1. **`scoreCompactionItems` 恰恰在 compaction 存在意义的规模上失败。** 它把每个条目的两个问题放进一个无上限的请求。针对真实端点实测，200 个条目在 2.85 秒内完成，而 400 个条目返回 HTTP 400 `max_tokens_exceeded`，完全得不到计划——恰恰在 transcript 大到需要压缩时硬失败。

2. **被拒绝的请求会唤醒回退提供方。** `DecisionRuntime.evaluate` 在主提供方抛出任何非 abort 错误时都会回退。TypeSafe 对不可用的密钥返回 401，对格式错误的请求体和超长的 `state` 都返回 400；换一个提供方都不会改变结果。在配置了 `dsh-decision-laya` 时，每一次这样的失败都会启动一个本地 PyTorch worker（60 秒启动预算），最终得到同样的失败。

3. **没有任何消费方依据仓库自身的指令来判断工作**，尽管本仓库在根目录、`packages/` 与 `docs/` 都带有具约束力的 `AGENTS.md`。

## Decision

**为 compaction 评分做适配与批处理。** `scoreCompactionItems` 在四个截断档位中选出能放进 `maxStateTokens`（25000）的最宽档位来渲染状态，然后把问题拆分到受 `maxRequestTokens`（30000，低于端点的 32768）约束的并发请求中。每个条目在每个档位都保留在状态里——一个不在状态中的条目会被拿去与一份根本没提到它的 transcript 比较——因此降低的是细节，而不是丢弃条目。每个批次都重发完全相同的状态。

**把 4xx 归类为请求拒绝。** Jev 提供方把非 429 的 4xx 映射为 `DECISION_REQUEST_REJECTED`；`DecisionRuntime.evaluate` 上报该错误码而不回退。429 保留 `DECISION_RATE_LIMITED`，5xx 保留 `DECISION_PROVIDER_ERROR`，因此配额耗尽与服务端故障仍会回退。该分类归提供方所有，因为只有它们能看到后端状态码。

**新增 `assessInstructionDrift` 作为第十二个消费方**，返回一个 Boolean 概率及其与阈值的距离。它是建议性的：绝不阻断、重试或引导工作。

## Measured results

真实提供方，`tests/compaction-scale.e2e.ts`，400 个带标注条目：2.94 秒，全部条目都进入计划，40/40 条长期有效的用户约束被保留，307/308 条样板读取被丢弃，裁剪 1,330,380 字节，保留 22,020 字节。改动前的代码路径在同样输入上返回 HTTP 400。

批处理不改变答案。对 200 个条目分别用两种方式评分，决策一致率为 200/200，平均 |Δp| 为 0.013，最大 0.050。

指令偏离在两个数据集共十六个带标注场景上：在 0.90 默认阈值下 16/16 正确，零误报零漏报。foreman 公布的 0.80 阈值在此为 15/16，会把合规工作报成偏离；0.95 为 12/16，漏掉四个真实违规。

## Alternatives considered

- **foreman 的 0.80 偏离阈值**——作为默认值被否决。一个硬编码的 PBKDF2 迭代次数（本仓库指令明确豁免："安全不变量保持固定"）在隔离场景下五次运行均得 0.90（波动 ±0.01），在上下文中为 0.82。该失败是稳定的而非噪声，因此阈值设在该区间之上，并把这一豁免局限记录进包 README。
- **把偏离做成阻断式 gate**——否决。对带豁免条款的规则出现稳定误报会打断正确的工作，而该 seam 不返回任何文本用于解释这种打断。
- **为浏览器驱动做投机式 fan-out**（`jev-ultrafast`）——仅 fan-out 机制本身已经实现：实测相对顺序的两请求循环降低 48.6% 延迟且 5/5 决策一致，而 `decideEgoStep` 已经在一个请求中同时询问元素、完成、受阻与风险。但 fan-out 并非该项目的主要成果，其动态动作空间在本仓库中并不存在；`decideBrowserAction` 在 [2026-09-21-dynamic-browser-action-policy.zh.md](2026-09-21-dynamic-browser-action-policy.zh.md) 中补齐了这一点。
- **对被丢弃的条目做摘要**——否决，与 `fast-jev-compaction` 一致：摘要可能丢掉一个路径、一条确切报错或一条约束。保留的条目保持原文，只整条丢弃。
- **在服务层解析提供方错误文本**来识别确定性失败——否决；状态码在提供方处已经已知，而对厂商消息做字符串匹配算不上分类。

## Consequences

- 大于单个提供方请求的 transcript 现在会产出计划而不是报错；增长的是请求数量，而不是条目数量。
- 确定性的拒绝会直接呈现给调用方，不再触发回退冷启动。此前依赖旧行为掩盖坏凭据的部署，现在会看到 401。
- `DECISION_REQUEST_REJECTED` 是一个新的开放字符串错误码；消费方本就必须容忍提供方特有的错误码。
- `assessInstructionDrift` 是一个库函数，目前没有任何归属方把它接入 loop。负责展示其判断的上层留作待办，与 `selectSkill` 的情况相同。
- `maxStateTokens` 与 `maxRequestTokens` 都是消费方选项而非硬编码常量，因为上限属于提供方，并会随其变动。
