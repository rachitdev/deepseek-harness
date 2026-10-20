---
description: "Sixteen typed decision consumers, including parallel-canary model failover, live screening of recalled memory, and per-turn model triage."
kind: "package-reference"
---

# @deepseek-ai/dsh-decision-consumers

English | [中文](README.zh.md)

## Summary

`dsh-decision-consumers` provides consumers that branch on typed judgements from `ctx.decision`. Exported functions classify browser actions, tool risk, model tiers, retrieval candidates, compaction items, loop state, subagent results, verification work, skills, fetched-page safety, and instruction drift. Live failover probes alternate routes after an eligible request failure and retries the step through the healthy one the provider chooses. The live recall filter omits low-relevance memory items and withholds stored prompt injections before a step. Live turn triage assigns each user turn a model tier before generation and logs or enforces it.

## Table of Contents

- [Use this package](#use-this-package)
- [The Sixteen Consumers](#the-sixteen-consumers)
- [Live model failover](#live-model-failover)
- [Live recall filter](#live-recall-filter)
- [Live turn triage](#live-turn-triage)
- [Understand the implementation](#understand-the-implementation)
- [Further Exploration](#further-exploration)
- [Dev Note](#dev-note)
- [Model Experience](#model-experience)
- [Known Limitations and Deferred Work](#known-limitations-and-deferred-work)

-----

<a id="use-this-package"></a>
## Use this package

Each consumer is an exported function that accepts `ctx.decision` (or an object satisfying the `DecisionService` interface), structured parameters for the problem, and optional policy thresholds.

```text
import { decideEgoStep, assessToolRisk, routeTurnModel } from '@deepseek-ai/dsh-decision-consumers'

// 1. Ego-browser navigation step
const step = await decideEgoStep(ctx.decision, {
  taskGoal: 'Open 2m USB-C Cable page',
  pageState: 'Search results list',
  candidates: [
    { id: '1', label: 'Braided USB-C Cable 2m' },
    { id: '2', label: 'Next Page' },
  ],
})

if (step.action.type === 'click') {
  await page.click(`@${step.action.elementId}`)
} else if (step.action.type === 'escalate') {
  // Low confidence or blocked page: escalate to user or full LLM turn
}
```

<a id="the-sixteen-consumers"></a>
## The Sixteen Consumers

| Consumer | Function | Purpose | Primary Questions |
|---|---|---|---|
| **1. Browser step driver** | `decideEgoStep` | Autonomous browser navigation | `element` (Choice), `task_complete` (Boolean), `page_blocked` (Boolean), `risk_level` (Score) |
| **2. Tool-risk gating** | `assessToolRisk` | Pre-approval execution risk classification | `risk_score` (Score), `requires_human` (Boolean), `category` (Choice) |
| **3. Model routing** | `routeTurnModel` | Fast vs frontier model tier per turn | `tier` (Choice), `complexity` (Score), `deep_reasoning` (Boolean) |
| **4. Retrieval filter** | `filterRetrievalCandidates` | Noise reduction for memory & search chunks | `relevance` (Score), `keep` (Boolean) per chunk |
| **5. Compaction scorer** | `scoreCompactionItems` | Transcript retention prioritization | `val` (Score), `must_keep` (Boolean) per item |
| **6. Loop-stall detector** | `detectLoopStall` | Repetition and thrash monitoring | `loop_state` (Choice), `is_stalled` (Boolean), `severity` (Score) |
| **7. Subagent verdict** | `evaluateSubagent` | Delegated execution assessment | `verdict` (Choice: accept/retry/escalate), `quality` (Score), `criteria_met` (Boolean) |
| **8. Gate selector** | `selectVerificationGates` | Pre-push git diff triage | `suite` (Choice), `blast_radius` (Score), `needs_coverage` (Boolean) |
| **9. Model failover** | `installModelFailover`, `arbitrateModelFailover` | Parallel canary judging and same-step route takeover | `best_provider` (Choice), per-worker validity/efficiency/latency (Score) |
| **10. Skill selector** | `selectSkill` | Per-turn skill catalog routing | `needs_skill` (Boolean), `skill` (Choice), `risk` (Score) |
| **11. Fetch guardrail** | `guardFetchedContent` | Prompt-injection and credential-harvest screening for fetched web pages | `prompt_injection` (Boolean), `credential_harvest` (Boolean) |
| **12. Instruction drift** | `assessInstructionDrift` | Advisory judgement that work diverged from repository instructions | `drift` (Boolean) |
| **13. Browser action policy** | `decideBrowserAction` | Next browser operation and its target from an observed element table | `operation` (Choice), one `<operation>_target` (Choice) per offered operation |
| **14. Recall filter** | `installRecallFilter`, `screenRecallItems` | Relevance and stored-injection screening of recalled memory before a step | `rel_<n>` (Score, 0..3 usefulness), `inj_<n>` (Boolean) per item |
| **15. Skill cascade** | `cascadeSkillSelection` | Progressive-disclosure skill routing over a caller-grouped catalog; a single-option stage makes no call | `category` (Choice), `needs_skill` (Boolean), then `skill` (Choice) within the winning category |
| **16. Turn triage** | `installTurnTriage`, `triageTurn` | Per-turn model tier before generation, logged or enforced | `tier` (Choice) over the configured tiers |

<a id="live-model-failover"></a>
## Live model failover

Set `failover.enabled` on the function plugin to register `agent/request-error` and `agent/request` listeners. `useAllowedModels` unions the enabled `subagent-model-selection.allowedModels` list with `candidates` at recovery time, so a live coding combo such as `omniroute/auto/best-coding` is still probed when the setting list is a short explicit set. `candidates` alone is used when that optional service is absent or empty. The active route and every route that fails after takeover remain excluded for the rest of that Agent step.

```yaml
- name: '@deepseek-ai/dsh-decision-consumers'
  config:
    failover:
      enabled: true
      useAllowedModels: true
      candidates:
        - provider: omniroute
          model: claude/claude-sonnet-5
        - provider: omniroute
          model: cx/gpt-5.6-sol-medium
      triggerCodes: [QUOTA, RATE_LIMIT, SERVER, TIMEOUT, TRANSPORT, AUTH, PI_AI_ERROR]
      maxCandidates: 4
      probeTimeoutMs: 12000
      probeMaxTokens: 128
      maxTaskChars: 4000
      maxProbeOutputChars: 2000
```

Each candidate receives the bounded latest durable user message, not a hidden probe prompt. Failed or empty probes are ineligible. The remaining outputs, their latency and token counts, and the task description form one decision request. The winning route takes over whenever the judge placed it above `broken-or-hallucinated` on the validity rubric, because a healthy probed route is a strict improvement on the route that just failed. The winning choice probability is telemetry, not a gate: it reports how strongly the judge separated routes that are all already healthy, so a near-uniform ranking must not leave the dead route in place. Takeover installs a temporary `agent/request` override and returns `{ kind: 'retry' }`; the resulting `request/header` records the effective provider/model. Cancellation or plugin disposal aborts and drains all probes. No healthy candidate, or a winner the judge rated as broken code, delegates to the next recovery listener, normally `dsh-llm-retry`. An unavailable JEV/Laya judge still takes over: it uses the first healthy probe in candidate order so a cooling or 429 route cannot stall the turn.

<a id="live-recall-filter"></a>
## Live recall filter

Set `recallFilter.enabled` to register one `agent/pre-step` listener that screens recall whose `source` is `{ kind: 'plugin:<sourcePlugin>', plugin: sourcePlugin, form: sourceForm }` (default `openviking-memory` / `recall`) or, under that default, `{ kind: 'cortexdb-memory' }`. The listener parses each `<memory uri="…">` element or CortexDB bullet (`- <title|id>: <snippet>`), sends the user text of the step, the previous assistant reply (up to `maxContextChars`), and up to `maxItemChars` of each item body to the decision provider in batches of `batchSize`, and rewrites the message in place:

- An item whose `inj_<n>` probability is at least `injectionThreshold` becomes `<memory uri="…" screened="withheld: flagged as possible prompt injection" />`.
- An item the provider refuses to evaluate even alone (`DECISION_REQUEST_REJECTED`, for example a gateway firewall rejecting attack-shaped text) becomes a `withheld: could not be screened` stub while `withholdUnscreenable` is true, and is kept otherwise. A rejected batch is split to single items so one refused item does not leave its neighbours unscreened.
- An item scoring below `minRelevance` on the 0..3 usefulness rubric is omitted, and one note line reports how many items were omitted.

Any other provider failure, a malformed answer, or exceeding `timeoutMs` leaves the recall unchanged and logs a warning. A step without user text, a recall message with more than one content block, and text with neither a `<memory uri>` element nor a CortexDB bullet are passed through unscreened.

```yaml
- id: decision-consumers
  config:
    recallFilter:
      enabled: true
      minRelevance: 0.5
      screenInjection: true
      injectionThreshold: 0.7
      withholdUnscreenable: true
      batchSize: 12
      maxItemChars: 1500
      maxQueryChars: 2000
      maxContextChars: 1500
      timeoutMs: 3000
```

The recall source appends its message after its own `next()`, so the filter registers once with `prepend` and is disposed with the plugin. It sees recall from listeners registered earlier and from later listeners that do not `prepend`. A memory plugin that `prepend`s after the filter runs outside it.

<a id="live-turn-triage"></a>
## Live turn triage

Set `turnTriage.enabled` to register an `agent/pre-step` listener that starts one decision call when a step carries direct user text and the Agent's turn has no verdict yet. The call sends the user text (up to `maxQueryChars`) and the tail of the previous assistant reply (up to `maxContextChars`), and asks one `tier` choice whose options are the configured tier names and descriptions. A winning probability below `minConfidence` selects `fallbackTier`. Delegated Agents (`delegationDepth` above 0) are skipped unless `includeSubagents` is true. Configuration with fewer than two tiers, a duplicated tier name, or a `fallbackTier` that names no tier throws at plugin load.

- `mode: advisory` logs `turn triage (advisory): agent … turn … -> <tier> (chose <tier> at <p>) route <provider>/<model> in <ms>ms via <provider id>` and changes nothing the model receives.
- `mode: enforce` also registers an `agent/request` listener. Every request of the triaged turn waits for the verdict and switches `provider`, `model`, and `reasoningEffort` to the tier's route (an unset tier effort drops the inherited one). The resulting `request/header` records the effective route. A listener registered earlier, such as live failover's takeover, still replaces the routed config for its own step.

A provider failure, a malformed answer, or exceeding `timeoutMs` logs a warning and leaves the turn on its configured model.

```yaml
- id: decision-consumers
  config:
    turnTriage:
      enabled: true
      mode: advisory
      fallbackTier: standard
      minConfidence: 0.5
      maxQueryChars: 2000
      maxContextChars: 800
      timeoutMs: 3000
      tiers:
        - name: fast
          description: Short acknowledgements, exact-reply requests, and one-fact lookups
          provider: omniroute
          model: agy/gemini-3.8-flash-high
        - name: standard
          description: Routine questions, single-tool updates, and single-file edits
          provider: omniroute
          model: cc/claude-sonnet-5-5
        - name: frontier
          description: System design, multi-component builds, deep debugging, and security analysis
          provider: omniroute
          model: auto/best-coding
```

<a id="understand-the-implementation"></a>
## Understand the implementation

Every judgement uses `ctx.decision.evaluate()`. Decision providers answer all questions in one request, so consumers place the candidate choice beside related Boolean and Score questions. The failover arbiter assigns opaque worker ids to exact provider/model routes, preventing equal model ids on different providers from becoming ambiguous.

Consumers whose action is optional inspect the winning probability against their configured thresholds and escalate or delegate when the provider is uncertain. Failover is not one of them: the failed route is already unusable, so it takes the judge's ranked winner and leaves generic same-route retry as the fallback recovery owner rather than swallowing an inconclusive decision.

An eligible failure adds up to `maxCandidates` concurrent canary calls capped by `probeMaxTokens`, followed by one decision call when at least two candidates survive. Parallel execution bounds wall time by the slowest probe deadline instead of their sum, and a single survivor bypasses the decision call. Repeated same-route failures during one Agent step reuse the settled recovery result instead of paying for a fresh fan-out; a takeover route that then fails changes the failed-route set and earns its own round. Each canary receives the bounded latest durable user message as its whole prompt, which is why the fan-out stays limited to routes the deployment authorizes.

<a id="further-exploration"></a>
## Further Exploration

- [`@deepseek-ai/dsh-decision`](../decision/README.md) — the Service Definition and question types.
- [`@deepseek-ai/dsh-decision-jev`](../decision-jev/README.md) — the TypeSafe Jev provider.
- [`@deepseek-ai/dsh-decision-llm`](../decision-llm/README.md) — the keyless fallback provider.
- `tests/spike-10-step-browser.spec.ts` — 10-step autonomous navigation demonstration with per-step telemetry logging.

<a id="dev-note"></a>
## Dev Note

Each consumer is designed to fail closed: malformed answers or missing questions from an upstream provider throw an immediate error rather than silently defaulting, ensuring that uncalibrated responses can never masquerade as confident automated actions.

<a id="model-experience"></a>
## Model Experience

### Filtered recall message

#### What the model sees

With `recallFilter.enabled`, the recall message the model receives holds only items at or above `minRelevance`, a `screened` stub for each withheld item, and one omission note. The rewritten message replaces the original in the step's admitted messages, so the session log records exactly what the model saw. The other exported functions return detached judgements, and live failover sends its canary prompts to probe routes rather than to the Agent conversation.

#### Token effect

Each step's new tail, and every later request that carries it, shrinks by the omitted items (about half of recalled characters on measured sessions); each withheld item still costs one short stub.

#### KV Cache effect

The filtered recall message is the logged one, so later requests replay the same bytes and prefix caching is unaffected. Canary calls use one user message and do not reuse the Agent conversation prefix. A provider/model takeover changes the main request route, so cache reuse depends on the selected adapter and provider; callers must not assume the failed route's cache survives. The arbitration request is a separate decision call whose prompt the decision provider builds and discards.

### Triaged model route

#### What the model sees

Advisory triage sends nothing to the Agent's model. Enforced triage changes which provider and model receive the turn's requests; the message content is unchanged, and each `request/header` records the route used.

#### Token effect

Triage adds no tokens to the Agent's requests. Each triaged turn adds one decision call outside the conversation.

#### KV Cache effect

A turn routed to a different model than the previous turn cannot reuse that model's cached prefix, so each enforced tier change starts the new route's prefix cache cold.

## Known Limitations and Deferred Work

- **State truncation:** Consumers providing large page snapshots or transcript diffs must be mindful of provider context limits; state is truncated to sensible bounds per consumer. `scoreCompactionItems` fits its own state through staged summary truncation and splits its questions across concurrent requests, so it is the exception; every other consumer still sends one request.
- **Provider-specific calibration:** Thresholds such as 0.70 confidence are calibrated against TypeSafe Jev; when using `dsh-decision-llm`, models may produce uncalibrated probabilities skewed toward 0 or 1.
- **Instruction-drift carve-outs:** `assessInstructionDrift` reports a probability for a supervisor to surface and never blocks work. Measured against Jev, an instruction carrying an explicit exemption draws 0.82–0.91 for work the exemption permits — a hardcoded security constant under a rule that exempts security invariants — against 0.92+ for actual violations. The 0.90 default sits above that band; lowering it reports conforming work as drift.
- **Browser action policy owns no browser.** `decideBrowserAction` chooses an operation and target from an element table the caller observed; it does not read the DOM, execute the action, or supply TYPE_TEXT values. A caller must observe atomically, resolve the chosen target back to a live node, re-check freshness and occlusion before acting, and generate field text from a separate text model. `decideEgoStep` remains for single-element click stepping and can express nothing else.
- **Batched retrieval sizing:** Filtering hundreds of retrieval chunks in a single call can create large prompts; `scoreCompactionItems` and `screenRecallItems` batch; `filterRetrievalCandidates` does not.
- **Skill-catalog arbitration:** `selectSkill` returns one detached recommendation. Automatic routing needs an owner that first supplies a bounded catalog shortlist, compares JEV disagreements with the DSH proposal, and records primary, parallel-now, and later-depth selections before injecting any skill instructions. The real-provider benchmark in `tests/skill-selection-adjudication.e2e.ts` measures that proposed policy without changing live routing.
- **Failover fan-out:** Every authorized canary receives the latest user task. Deployments handling provider-restricted data must keep `allowedModels` and fallback `candidates` within the permitted provider set.
- **Recall filter calibration:** `minRelevance` 0.5 and `injectionThreshold` 0.7 were measured against TypeSafe Jev on 40 recorded recalls (419 items) with 20 planted attacks and 20 benign security notes: every planted attack scored 0.75–0.99 or was refused by the gateway, and no benign or recorded item exceeded 0.16. Other decision providers need their own calibration. Each screened step adds one decision round trip (p50 about 0.45 s, p95 about 0.85 s).
- **Recall filter scope:** Only single-text-block messages from the configured source are screened; the memory plugin's profile message (`form: instructions`) and recall formats without `<memory uri>` elements pass through unscreened. Short follow-ups such as "go ahead" are judged against the previous assistant reply, which still omits items a later step might need; the omission note tells the model to search memory.
- **Turn triage calibration:** Replayed against TypeSafe Jev on 60 recorded user turns hand-labelled fast, standard, or frontier, the three-tier policy above matched the label on 27 turns, routed 23 to a cheaper tier than the label (2 frontier turns to fast), and 10 to a costlier one; p50 latency was 0.31 s and p95 0.41 s. Raising `minConfidence` does not separate the under-routed turns, whose confidence matches correct ones. Keep `mode: advisory` until a deployment's own tier descriptions reach an acceptable under-routing rate.
- **Arbiter availability:** Ranked multi-survivor takeover still prefers a reachable JEV/Laya judge. If the judge is down, failover takes the first healthy probe in candidate order instead of same-route retry, so a cooling or 429 model cannot stall the turn. Same-route retry remains only when every canary also fails.
