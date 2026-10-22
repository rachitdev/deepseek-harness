---
description: "记录持久化类型更改及其兼容性确认。"
kind: persistence-change
---

# 2026-10-22-delegation-tool-name

[English](2026-10-22-delegation-tool-name.md) | 中文

## 概述

新增 `subagent-delegation` 消息来源，记录子级会话、摘要，以及其委派固定策略拒绝该子级请求的工具。整个来源分支都是新增的：没有任何已发布会话格式包含它，2026-10-13 基线中也没有。

## 目录

- [声明](#declaration)
- [兼容性](#compatibility)
- [验证](#verification)
- [开发备注](#dev-note)

<a id="declaration"></a>
## 声明

```yaml persistence-change
schemaVersion: 1
id: 2026-10-22-delegation-tool-name
baseline: false
changes:
  - root: "event:agent/inbox/spliced"
    previous: "2026-10-13-user-rpc-principal"
    after: "88992ac20b18c962fb8c70a8f0226d9d2cb403abedfa90e92fb726c50e19fd80"
    decision: same-version
  - root: "event:developer/message"
    previous: "2026-10-13-user-rpc-principal"
    after: "2ca77a7c10dcdb65df3ecca451c1923d60209950ebca9e5f2aab189d7ef731e2"
    decision: same-version
  - root: "event:session/title-llm-request"
    previous: "2026-10-13-user-rpc-principal"
    after: "6689b4bc09d4e8877b2e71b0e8e35b34e258faafafbfb3c2a6b6787df9dac25e"
    decision: same-version
  - root: "event:user/message"
    previous: "2026-10-13-user-rpc-principal"
    after: "ccb9500b8d3eb89682d4dd62122cded55cb9bfce21339787138ce9db166e0fb4"
    decision: same-version
```

<a id="compatibility"></a>
## 兼容性

已有记录不可能包含该来源类型，因此两个读取方向都是安全的。已发布的读取方只要求 `kind` 为非空字符串，按消息来源类型分支的消费方对未知类型会落入默认分支；通知本身只从 `summary` 渲染。本次变更新增的是来源类型，而不是在已发布类型上添加字段，因此不引入字段缺失的情况，头部版本保持不变。

<a id="verification"></a>
## 验证

pnpm exec vitest run packages/subagent/subagent/tests/delegation-notice.spec.ts：7 个测试通过。`git tag --contains HEAD` 无输出。将本记录的 schema 快照与 2026-10-13 记录做字面量比对，四个受影响根中 `subagent-delegation` 是唯一新增字面量。

<a id="dev-note"></a>
## 开发备注

无。
