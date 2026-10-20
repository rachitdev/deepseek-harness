# Agent Note: 模型 failover 漏掉了实时 OmniRoute 配额与计费错误

Status: implemented

[English](2026-09-20-model-failover-missed-omniroute-errors.md) | 中文

## 问题

2026-09-20 的会话 `647801a3` 为 Gemini 3.8 记录了九次 `RATE_LIMIT` 结束事件（文案为 “All antigravity accounts have exhausted their quota”），随后在同一条 `omniroute` 路由上产生八次 `llm/retry`。之后 Codex Sol 以 `PI_AI_ERROR`（“credentials … cooling down”）失败，OpenCode Go 以 `PI_AI_ERROR`（HTTP 402 `payment_required`）失败。没有任何 `request/header` 变更，也没有 JEV/Laya 仲裁。用户只能手动改选 Sol、Omen，再改选 Grok。

三段彼此独立的缺口造成了这份记录：

1. `isQuotaExceededError` 无法匹配 OmniRoute 文案（`exhausted their quota`、`cooling down`、HTTP 402 / `payment_required` / `billing_error`），因此可恢复的死路由失败要么留在 `RATE_LIMIT`，要么落入 `PI_AI_ERROR`。
2. failover 默认 `triggerCodes` 省略了 `AUTH` 与 `PI_AI_ERROR`，单元测试还把 `AUTH` 当成非触发码。未分类的提供方错误因此永远不会进入并行探测。
3. `useAllowedModels: true` 会用 `subagent-model-selection.allowedModels` **替换** `failover.candidates`。实时设置列表是三条显式的 Gemini/Grok 固定项，并不包含 `omniroute/auto/best-coding`，所以 JEV 从未探测 OmniRoute 的编码阶梯。

## 决策

- 扩展 `isQuotaExceededError` 以覆盖上述 OmniRoute 短语。`classifyPiAiError` 在 429 分支之前把 HTTP 402 映射为 `QUOTA`，因此欠费路由不会被当成短暂限流来重试。
- 把 `AUTH` 与 `PI_AI_ERROR` 纳入 failover 的默认触发集合。同路由 `llm-retry` 仍然不会重试 `PI_AI_ERROR` 或 `QUOTA`；failover 要么切换，要么该轮次失败。
- 在设置启用时把 `allowedModels` 与 `candidates` 做并集，使 `omniroute/auto/best-coding` 作为探测目标保留在显式固定项旁边。
- 不改动 OmniRoute 网关、组合与凭据。

## 曾考虑的替代方案

**把 `PI_AI_ERROR` 留在 failover 之外，只分类更多字符串。** 已否决：实时 Sol/Omen 失败已经落入该桶；未来未列出的 OmniRoute 短语会再次既不重试也不切换。

**只用 `auto/best-coding` 替换 `allowedModels`。** 已否决：当组合本身不健康时，显式的 Gemini/Grok 固定项仍是有用的 canary；并集两者都保留。

**从本会话重启实时 `dsh web` 宿主。** 已否决：另起一个服务器不会更新 `http://127.0.0.1:3080`。web profile 的实时 patch 重载会应用配置；操作步骤是刷新该 URL。

## 后果

- OmniRoute 的配额耗尽与计费错误变为 `QUOTA`，并启动 JEV/Laya 仲裁，而不是八次同路由重试。
- 未分类的提供方错误走同一条并行探测路径。
- 探测顺序为 allowed-models 再加配置的 candidates，受 `maxCandidates` 限制。
- 在 pi-ai 转发 `code`/`cause` 之前，分类仍依赖消息文本匹配。
