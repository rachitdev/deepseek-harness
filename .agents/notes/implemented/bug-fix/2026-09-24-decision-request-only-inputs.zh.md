# Agent Note：decision 提示词作为仅用于请求的输入

Status: implemented

[English](2026-09-24-decision-request-only-inputs.md) | 中文

## 问题

`dsh-decision-llm` 与 `dsh-decision-consumers` 中的实时 failover 把各自的辅助提示词作为持久 user 消息发送，并使用提供方来源类型 `dsh-decision-llm` 与 `dsh-decision-consumers`。这两个类型加入了 `MessageSourceMap`，于是持久化清单在 `user/message`、`developer/message`、`agent/inbox/spliced` 与 `session/title-llm-request` 上记录了两个新的 user 来源变体。`verify-persistence-changes` 将该变更判定为相对已接受的 Session 格式 4 基线的破坏性变更，尽管这两个提示词都从未写入 Session。

## 决策

- 两个提示词都是 `RequestUserInput` 值（`{ role: 'user', content }`，不含 `id`，不含 `source`），这是 `dsh-llm` 为仅用于单次请求的 user 输入提供的类型。
- 两个包都不声明 `MessageSourceMap` 成员，因此 Session 格式 4 清单保持不变，`verify-persistence-changes` 与已接受的历史一致。
- decision 提示词在提供方调用内部构建并丢弃。failover 探测文本是持久 `user/message` 事件的有界投影，因此每个探测输入都能从 Session 日志重建。

## 考虑过的替代方案

**保留来源类型并把 Session 格式提升到 5。** 格式提升表示存在一项实际并不存在的持久变更，并会迫使每个已存储的 Session 进行迁移。

**保留来源类型并把该变更确认为非破坏性。** 清单会记录没有任何写入方产生的变体，而该确认会错误描述 Session 格式。

## 验证

`pnpm exec vitest run packages/decision` 通过 250 个测试；`pnpm run verify-persistence-changes` 报告 62 个根与 6 条历史记录一致；`pnpm exec tsc -b tsconfig.host.json` 通过。

## 后果

今后必须持久化其提示词的 decision 生产方，需要同时声明自己的 `MessageSourceMap` 类型并配套 Session 格式变更。这两个包中仅用于请求的提示词不会出现在 Session 日志、`dsh sessions` 输出或持久化清单中。
