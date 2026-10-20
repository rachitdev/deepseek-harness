---
description: "decision 能力族的包地图：带类型判断的服务，以及回答其问题的提供方后端。"
kind: "package-group"
---

# decision/ —— decision 能力族

[English](README.md) | 中文

## 概述

`decision/` 组让任何插件通过一个提供方无关的服务 `ctx.decision`，就非结构化状态提出带类型的问题（选一个选项、按评分标准打分、判断陈述真假）。答案携带校准概率且不含文本，因此消费方依据置信度设阈值，而不是解析散文。`decision/` 负责问题校验、主要／后备提供方选择与错误。`decision-jev/`、`decision-laya/` 与 `decision-llm/` 是后端：经 TypeSafe API 访问的 Jev、仅在主要提供方失败后启动的本地 Laya 检查点，以及被约束为同一答案格式的对话模型。`decision-consumers/` 打包各个消费方。文本生成仍归 LLM 能力。

## 目录

- [包](#packages)
- [相关文档](#related-documentation)
- [开发备注](#dev-note)

-----

<a id="packages"></a>
## 包

五个包分别扮演 decision 的各类角色；[subsystem 参考](../../docs/subsystems/decision.zh.md)持有详尽的词汇与契约。

| 包 | 角色 | ctx 键 |
|---|---|---|
| [`decision/`](decision/README.zh.md) | 带类型判断的服务：通过可替换的后端回答 choice、score 与 boolean 问题，共用一套校验、选择与错误策略 | `ctx.decision` |
| [`decision-jev/`](decision-jev/README.zh.md) | 通过 TypeSafe 直接 System One API 上的 Jev 作答 | 注册到 `ctx.decision` |
| [`decision-laya/`](decision-laya/README.zh.md) | 仅在主要提供方失败后启动缓存的本地 Laya typed-decision 检查点 | 注册到 `ctx.decision` |
| [`decision-llm/`](decision-llm/README.zh.md) | 通过把已注册的对话模型约束为带类型 JSON 回复来作答 | 注册到 `ctx.decision` |
| [`decision-consumers/`](decision-consumers/README.zh.md) | 11 个 decision 消费方的套件（浏览器步骤驱动、工具把关、路由、过滤、compaction、停滞检测、subagent 判定、检查项选择、技能选择、网页内容把关） | 消费 `ctx.decision` |

<a id="related-documentation"></a>
## 相关文档

- [Decision subsystem 参考](../../docs/subsystems/decision.zh.md) —— 问题与答案类型、校验规则、选择策略与错误表。
- [能力 seam](../../docs/glossary.zh.md#capability-seam) —— 本组遵循的 Service Definition / Provider / Consumer 划分。

<a id="dev-note"></a>
## 开发备注

这些后端并不共用传输方式，这正是该能力是一条 seam、而不是又一条 `LlmService` 路由的原因：TypeSafe 与本地 Laya 接受带类型的问题且不返回文本，而 LLM 后端通过普通 completion 回答同样的问题。把三者放在同一个服务之后，才能让消费方写一次，就能使用远程主提供方、按需本地后备，或对话模型替代。
