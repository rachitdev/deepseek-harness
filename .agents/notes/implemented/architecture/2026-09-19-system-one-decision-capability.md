# Agent Note: System One decision capability

Status: implemented

English | [中文](2026-09-19-system-one-decision-capability.zh.md)

## Problem

Judgements inside DeepSeek Harness historically cost a full LLM turn: choosing the next browser action, deciding whether a shell call is dangerous, picking a model for a turn, judging which tool results still matter at compaction time, and scoring retrieved memories are all classification problems, but the harness answered them by generating text with a frontier model. Each one spent seconds of latency and cents of tokens for an answer whose entire information content is one label and a confidence.

The cost landed hardest in loops. An ego-browser session clicking through twenty pages spent twenty model turns deciding which element to click — work whose output is an index into a list the harness already had.

TypeSafe's Jev is a System One model: it takes one state plus typed questions and returns typed answers with calibrated probabilities, generating no strings. It answers "which of these, how strongly, and how sure". Bolting it onto `LlmService` was infeasible because the two shipped providers do not share a transport: TypeSafe's System One endpoint accepts no messages and returns no text, so it cannot be expressed as a chat completion, while an LLM fallback constrains an ordinary completion to the same answer format.

## Decision

We added a dedicated **decision capability seam** — `packages/decision/` — complete in all three capability-seam roles across five packages:

1. **Service Definition** (`@deepseek-ai/dsh-decision`): owns `ctx.decision`, question validation, provider selection, and the error vocabulary. It defines a closed question union (`choice`, `score`, `boolean`) and answer union, validates requests before provider selection, and fails loud on ambiguity or unavailability.
2. **Service Provider — Jev** (`@deepseek-ai/dsh-decision-jev`): reaches Jev on TypeSafe's direct System One endpoint (`https://api.typesafe.ai/v1/systemone`) with bearer credential reference `TYPESAFE_API_KEY` and model `jev-latest` in the request body. `jev-latest` is a moving alias, not a pinned version; `$TYPESAFE_MODEL` can override it when the config field is unset. It translates the seam's `boolean` questions to TypeSafe's `noul` vocabulary and maps calibrated answers back.
3. **Service Provider — Laya Fallback** (`@deepseek-ai/dsh-decision-laya`): starts a cached local Laya `typed-decisions` checkpoint only after the primary provider cannot answer. The worker binds loopback port `58085`, loads weights with `local_files_only`, and exits after an idle timeout so a healthy Jev path holds no Laya RAM.
4. **Service Provider — LLM Fallback** (`@deepseek-ai/dsh-decision-llm`): constrains a registered `LlmService` chat model to a typed JSON reply, validating every field at the model-output boundary. It requires no decision-specific credential, ensuring the seam is never a single-vendor dependency.
5. **Consumers** (`@deepseek-ai/dsh-decision-consumers`): nine decision-driven operations and bindings that consume `ctx.decision`:
   - `decideEgoStep`: autonomous browser navigation with candidate action evaluation, completion/blocked detection, and calibrated confidence gating before clicking or escalating.
   - `assessToolRisk`: pre-approval tool call risk scoring (`read-only`, `safe-workspace-mutation`, `destructive-or-external`, `system-critical`) and permission recommendation (`allow`, `prompt`, `deny`).
   - `routeTurnModel`: turn-level complexity routing between fast/cheap (`deepseek-chat`) and frontier (`deepseek-reasoner`) model tiers.
   - `filterRetrievalCandidates`: relevance scoring and noise reduction for memory and doc search chunks evaluated in parallel in a single decision pass.
   - `scoreCompactionItems`: transcript retention prioritization partitioning items into keep, drop, and summarize sets with byte accounting.
   - `detectLoopStall`: probabilistic loop stall and repetition detection over recent action sequences, recommending `continue`, `warn`, or `interrupt`.
   - `evaluateSubagent`: delegated subagent execution evaluation issuing typed verdicts (`accept`, `retry`, `escalate`) with actionable feedback recommendations.
   - `selectVerificationGates`: pre-push git diff triage mapping changed files to minimal sufficient gate suites (`doc_only`, `lint_and_unit`, `full_coverage`, `full_with_e2e`).
   - live model failover: intercepts configured `agent/request-error` failures, probes authorized provider/model routes concurrently, asks JEV to judge code validity, token efficiency, and latency, overrides the same step's next `agent/request`, and returns `{ kind: 'retry' }`. The resulting `request/header` is the durable route record. Failed takeover routes remain excluded for that step; cancellation and disposal abort and drain active probes.

`dsh-llm-retry` delegates to downstream specialized recovery before it schedules same-route backoff. This keeps the generic retry plugin as the fallback regardless of listener registration order: unavailable arbitration, no healthy probe, or a winner the judge rated as broken code reaches ordinary retry instead of terminating the turn.

### Provider routes evaluated

| Route | Provider | Endpoint | Cost / limits | Status |
|---|---|---|---|---|
| **TypeSafe direct** | TypeSafe AI | `https://api.typesafe.ai/v1/systemone` | Account rate limits; direct bearer credential | **Selected locally**; live e2e verified at 1100 ms and 8/8 burst calls succeeded |
| **Local Laya fallback** | ConvAI Laya | loopback `http://127.0.0.1:58085/v1/systemone` | Zero per-call cost; ~20 s cold load, idle exit | On-demand after Jev failure; `typed-decisions` checkpoint |
| **Local LLM fallback** | Any registered `llm` route | In-harness `ctx.llm.stream` | Zero new credentials; portability floor | Implemented and tested; not mounted in the base composition |
| **Vercel AI Gateway** | TypeSafe via Vercel | `https://ai-gateway.vercel.sh/v4/ai/evaluation-model` | $0.042/M in, out free; rate-limited on free tier | Bootstrap route replaced when direct TypeSafe access became available |
| **Cloudflare AI Gateway** | Cloudflare | Universal gateway | Requires Workers Paid plus account balance | Rejected |

### Measured reality

Direct TypeSafe round trips from this host measured 345–460 ms after the first call, with a median of 406 ms across eight consecutive successful calls; the first call took 1243 ms. The real endpoint e2e completed in 1100 ms, and the eight-call burst produced no HTTP 429. These measurements establish a local baseline, not an account-limit guarantee.

The provider maps a direct TypeSafe HTTP 429 to `DECISION_RATE_LIMITED` so a consumer can fall back. The local Web composition mounts `dsh-decision-laya` as `fallbackProvider` and does not mount `dsh-decision-llm`, so an unavailable JEV arbiter first tries the on-demand Laya worker and only then delegates model recovery to `dsh-llm-retry`.

## Alternatives considered

**Fold decision into LlmService as an auxiliary model purpose.** An adapter or purpose flag like `purpose: 'decision'` would let existing routes serve decisions. This was rejected because TypeSafe's System One endpoint is structurally not chat-completions shaped: it takes no conversation messages, does no streaming, and produces no text. Forcing evaluation calls into the LLM adapter contract would require fake `Message` envelopes and stubbed `StreamChunk` generators that violate the adapter protocol.

**Call TypeSafe through its JavaScript SDK.** The SDK provides typed clients and automatic retry behavior. This was rejected for runtime code because the direct endpoint needs one authenticated JSON request and a small native fetch provider implements that protocol without adding an SDK dependency to a core runtime package.

**Keep an intermediary gateway in front of TypeSafe.** Vercel and Cloudflare can broker third-party model traffic, but an intermediary adds another credential, protocol translation, quota, and network hop. Direct TypeSafe access is available on this deployment, so the native endpoint is the smaller operating path.

**Put model failover inside `dsh-llm-retry`.** The retry plugin owns provider-policy backoff, not candidate authorization, canary generation, or decision-provider selection. Folding takeover into it would make generic transport recovery depend on the optional decision capability. The request-error waterfall instead lets the retry listener delegate first and remain the same-route fallback.

## Consequences

What this buys:
- **Portability and resilience:** Decisions are decoupled from any single provider. The harness works identically against Jev (fast, calibrated), an on-demand local Laya fallback (zero per-call cost after cold load), or an LLM fallback (keyless, universal).
- **Calibrated gating:** Consumers can make quantitative confidence gates (e.g. `probability > 0.85`) rather than binary guesses, bounding downside risk by escalating to full LLM turns when confidence is low.
- **Latency reduction:** Warm direct TypeSafe decisions measured 345–460 ms on this host, while the first call and isolated e2e remained near 1.1–1.2 s; either avoids a full 3–5 s chat completion for classification work.
- **Eleven implemented consumer paths:** Ten typed functions cover browser automation, tool approval, model routing, memory retrieval, context compaction, loop guards, subagents, pre-push verification, skill catalog routing, and fetched-page safety; live model failover is the only consumer with an automatic runtime binding.

What this costs:
- **Five packages to maintain:** `@deepseek-ai/dsh-decision`, `@deepseek-ai/dsh-decision-jev`, `@deepseek-ai/dsh-decision-laya`, `@deepseek-ai/dsh-decision-llm`, and `@deepseek-ai/dsh-decision-consumers`.
- **Direct account rate limits:** High-cadence loops remain subject to TypeSafe HTTP 429 responses and require adequate account capacity or a mounted fallback; eight successful calls do not establish a sustained-rate guarantee.
- **Strictly classification-only:** Cannot be used for text generation, code editing, or naming.
- **Failover fan-out:** One eligible request failure can issue several bounded canary calls. Candidate authorization therefore also governs which providers may receive the latest user task.

## Verification

- `packages/decision/decision/tests/decision.spec.ts`: 25 unit tests covering provider registration, disposer idempotency, replacement stability, duplicate rejection, explicit and env-configured selection, ambiguous selection failure, unavailability failure, question validation, and on-demand fallback.
- `packages/decision/decision-jev/tests/decision-jev.spec.ts`: 41 unit tests covering question/answer translation, criteria mapping, availability checks, fetch header composition, rate-limit error mapping, cancellation forwarding, credential resolution, and `$TYPESAFE_MODEL` override.
- `packages/decision/decision-laya/tests/decision-laya.spec.ts`: 12 unit tests covering question/answer translation, healthy-worker evaluation, HTTP errors, abort mapping, and fiber disposal.
- `packages/decision/decision-llm/tests/decision-llm.spec.ts`: 27 unit tests covering prompt rendering, response parsing, code-fence stripping, schema validation, option/rubric bounds enforcement, and stream assembly.
- `packages/decision/decision-consumers/tests/*.spec.ts`: unit coverage for all eleven consumers, including all-probe failure, broken-code verdicts, near-uniform rankings that still take over, failed-route exclusion, allowed-model sourcing, empty-catalog local skill decisions, and cancellation/disposal quiescence.
- `packages/decision/decision-consumers/tests/loader.spec.ts`: a real app/boot and Cordis Loader composition proves a 429 on model A produces parallel probes, one decision call, a model-B retry in the same Agent step, and model B recorded in the durable `request/header` and in the assistant message's `model` field.
- `packages/llm/llm-retry/tests/retry.spec.ts`: mounts specialized recovery after generic retry and proves downstream `{ kind: 'retry' }` wins without a same-route retry event.
- `packages/decision/decision-consumers/tests/spike-10-step-browser.spec.ts`: 10-step autonomous navigation trajectory logging per-step element choices, confidence probabilities, risk scores, completion, and escalation behavior (86.7% average confidence).
- `packages/decision/decision-jev/tests/gateway.e2e.ts`: live integration test against TypeSafe's direct System One endpoint; self-skips when `TYPESAFE_API_KEY` is absent; passed with HTTP 200 in 1100 ms on this host.
