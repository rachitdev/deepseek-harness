# Agent Note: Model failover missed live OmniRoute quota and billing errors

Status: implemented

English | [中文](2026-09-20-model-failover-missed-omniroute-errors.zh.md)

## Problem

Session `647801a3` on 2026-09-20 logged nine Gemini 3.8 `RATE_LIMIT` finishes for "All antigravity accounts have exhausted their quota", then eight `llm/retry` events on the same `omniroute` route. Codex Sol later died as `PI_AI_ERROR` ("credentials … cooling down") and OpenCode Go as `PI_AI_ERROR` (HTTP 402 `payment_required`). No `request/header` change and no JEV/Laya arbitration occurred. The user had to pick Sol, then Omen, then Grok by hand.

Three independent gaps produced that transcript:

1. `isQuotaExceededError` did not match OmniRoute wording (`exhausted their quota`, `cooling down`, HTTP 402 / `payment_required` / `billing_error`), so recoverable dead-route failures stayed `RATE_LIMIT` or fell through to `PI_AI_ERROR`.
2. Default failover `triggerCodes` omitted `AUTH` and `PI_AI_ERROR`, and the unit suite treated `AUTH` as a non-trigger. An unclassified provider error therefore never entered parallel probing.
3. `useAllowedModels: true` *replaced* `failover.candidates` with `subagent-model-selection.allowedModels`. The live setting list is three explicit Gemini/Grok pins and does not include `omniroute/auto/best-coding`, so JEV never probed OmniRoute's coding ladder.

## Decision

- Extend `isQuotaExceededError` with those OmniRoute phrases. `classifyPiAiError` maps HTTP 402 to `QUOTA` before the 429 branch, so a billed-out route is not retried as a transient rate limit.
- Include `AUTH` and `PI_AI_ERROR` in failover's default trigger set. Same-route `llm-retry` still does not retry `PI_AI_ERROR` or `QUOTA`; failover either switches or the turn fails.
- Union `allowedModels` with `candidates` when the setting is enabled, preserving `omniroute/auto/best-coding` as a probe target beside the explicit pins.
- Keep OmniRoute's gateway, combos, and credentials unchanged.

## Alternatives considered

**Leave `PI_AI_ERROR` out of failover and only classify more strings.** Rejected: live Sol/Omen failures already landed in that bucket, and a future unlisted OmniRoute phrase would again retry nothing and switch nothing.

**Replace `allowedModels` with only `auto/best-coding`.** Rejected: the explicit Gemini/Grok pins remain useful canaries when the combo itself is unhealthy; the union keeps both.

**Restart the live `dsh web` host from this session.** Rejected: another server does not update `http://127.0.0.1:3080`. The web profile's live patch reload applies config; a page refresh of that URL is the operator step.

## Consequences

- Exhausted-quota and billing OmniRoute errors become `QUOTA` and start JEV/Laya arbitration instead of eight same-route retries.
- Unclassified provider errors start the same parallel probe path.
- Probe order is allowed-models then configured candidates, capped by `maxCandidates`.
- Classification remains message-text matching until pi-ai forwards `code`/`cause`.
