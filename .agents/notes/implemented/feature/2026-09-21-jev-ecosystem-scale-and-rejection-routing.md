# Agent Note: Compaction batching, instruction-drift advisory, and request-rejection routing

Status: implemented

English | [中文](2026-09-21-jev-ecosystem-scale-and-rejection-routing.zh.md)

## Problem

A scan of six independent TypeSafe Jev projects (`jev-review`, `foreman`, `typesafe-computer-use`, `jev-search`, `jev-ultrafast`, `fast-jev-compaction`) against this repository's decision seam found three defects and one absent consumer. The seam already implemented nine of the eleven patterns those projects demonstrate, so the value was in the gaps rather than in new capability.

Measurement from Noida against `api.typesafe.ai` established the property the fixes depend on: answer latency tracks the number of requests, not the number of questions. One question and forty questions both answered in ~0.95 s (1 → 486 input tokens, 40 → 1257). Extra question heads are effectively free; extra requests are not.

1. **`scoreCompactionItems` failed at the size compaction exists for.** It sent every item's two questions in one unbounded request. Measured against the real endpoint, 200 items answered in 2.85 s, and 400 items returned HTTP 400 `max_tokens_exceeded` with no plan at all — a hard failure precisely when a transcript is large enough to need compacting.

2. **A rejected request woke the fallback provider.** `DecisionRuntime.evaluate` fell back on any non-abort primary error. TypeSafe returns 401 for an unusable key and 400 for both a malformed body and an oversized `state`; none of these change under a different provider. With `dsh-decision-laya` configured, each one started a local PyTorch worker (60 s startup budget) to reach the same failure.

3. **No consumer judged work against the repository's own instructions**, though this repository carries binding `AGENTS.md` files at root, `packages/`, and `docs/`.

## Decision

**Fit and batch compaction scoring.** `scoreCompactionItems` renders its state at the widest of four truncation stages that fits `maxStateTokens` (25000), then splits questions across concurrent requests bounded by `maxRequestTokens` (30000, under the endpoint's 32768). Every item stays present in the state at every stage — an item absent from the state would be judged against a transcript that does not mention it — so detail is reduced rather than items dropped. The identical state is resent with every batch.

**Classify 4xx as a request rejection.** The Jev provider maps non-429 4xx to `DECISION_REQUEST_REJECTED`; `DecisionRuntime.evaluate` surfaces that code instead of falling back. 429 keeps `DECISION_RATE_LIMITED` and 5xx keeps `DECISION_PROVIDER_ERROR`, so quota exhaustion and server-side failure still fall back. Providers own the classification because only they see the backend status.

**Add `assessInstructionDrift` as consumer twelve**, reporting one Boolean probability and its distance from the threshold. It is advisory: it never blocks, retries, or steers.

## Measured results

Real provider, `tests/compaction-scale.e2e.ts`, 400 labeled items: 2.94 s, every item planned, 40/40 standing user constraints retained, 307/308 boilerplate reads dropped, 1,330,380 bytes pruned against 22,020 retained. The pre-change code path returns HTTP 400 on the same input.

Batching does not change the answers. Over 200 items scored both ways, decision agreement was 200/200 with mean |Δp| 0.013 and max 0.050.

Instruction drift over sixteen labeled scenarios across two datasets: 16/16 correct at the 0.90 default, with zero false positives and zero false negatives. Foreman's published 0.80 threshold scored 15/16 here, reporting conforming work as drift; 0.95 scored 12/16, missing four real violations.

## Alternatives considered

- **Foreman's 0.80 drift threshold** — rejected as the default. A hardcoded PBKDF2 iteration count, which this repository's instructions explicitly exempt ("security invariants stay fixed"), scored 0.90 in isolation across five runs (variance ±0.01) and 0.82 in context. The failure is stable rather than noisy, so the threshold sits above that band and the carve-out limitation is recorded in the package README.
- **A blocking drift gate** — rejected. A stable false positive on exemption-bearing rules would interrupt correct work, and the seam returns no text with which to explain an interruption.
- **Speculative fan-out for the browser driver** (`jev-ultrafast`) — the fan-out mechanism alone was already implemented: measured at 48.6% latency reduction over a sequential two-request loop with 5/5 identical decisions, but `decideEgoStep` already asks element, completion, blocked, and risk in one request. Fan-out is not that project's main result, and its dynamic action space was absent here; `decideBrowserAction` adds it in [2026-09-21-dynamic-browser-action-policy.md](2026-09-21-dynamic-browser-action-policy.md).
- **Summarizing dropped items** — rejected, following `fast-jev-compaction`: a summary can lose a path, an exact error, or a constraint. Retained items stay verbatim; only whole items are dropped.
- **Parsing provider error text in the service** to detect deterministic failures — rejected; the status code is already known at the provider, and string matching on a vendor message is not a classification.

## Consequences

- A transcript larger than one provider request now produces a plan instead of an error; the request count, not the item count, is what grows.
- A deterministic rejection surfaces at the caller without a fallback cold start. A deployment relying on the previous behavior to mask a bad credential now sees the 401.
- `DECISION_REQUEST_REJECTED` is a new open-string code; consumers already must tolerate provider-specific codes.
- `assessInstructionDrift` is a library function with no owner wiring it into the loop. A supervisor that surfaces its verdict is deferred work, as with `selectSkill`.
- Both `maxStateTokens` and `maxRequestTokens` are consumer options rather than hardcoded constants, because the ceiling belongs to the provider and moves with it.
