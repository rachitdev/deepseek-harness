# Decision

English | [中文](decision.zh.md)

The decision seam — a [capability seam](../glossary.md#capability-seam) spanning **one operation** (`evaluate`) on a `ctx.decision` service — is split across a Service Definition ([dsh-decision](../../packages/decision/decision), `ctx.decision` plus the provider registry), Service Providers ([dsh-decision-jev](../../packages/decision/decision-jev), [dsh-decision-laya](../../packages/decision/decision-laya), [dsh-decision-llm](../../packages/decision/decision-llm)), and typed [consumers](../../packages/decision/decision-consumers). Decision is **one optional capability**, not part of the agent-loop spine, so its vocabulary lives here rather than in [core.md](core.md). A provider swap does not change how a consumer asks its question.

Source: [`packages/decision/decision/src/types.ts`](../../packages/decision/decision/src/types.ts)

## What this capability is for

A decision is a judgement about unstructured state whose answer is a selection, a rating, or a truth probability — never text. "Which of these page elements advances the task", "how destructive is this tool call", "is this retrieved document relevant". Such questions are answerable by a model but do not need one to write anything, and the harness cannot compute them itself.

Keeping them off `LlmService` is not a stylistic split. The shipped providers do not share a transport: `dsh-decision-jev` posts to TypeSafe's System One endpoint, which accepts no messages and returns no text; `dsh-decision-laya` answers the same typed questions through a local Laya checkpoint started only after the primary fails; `dsh-decision-llm` constrains an ordinary chat model to the same answer format. A consumer written against this seam runs on any of them. The inverse also holds: prose, names, and commit messages stay with `LlmService`, because a decision answer carries no strings by construction.

## Questions and answers

One request carries a `state` string and a map of named questions, and every question is answered in the same round trip. Asking several together is the intended use — the cost is per call, not per question — so a consumer already paying for a decision should ride its other questions along rather than issue a second call.

```ts type-equiv
/**
 * One question put to a decision provider. A CLOSED discriminated union owned by `dsh-decision`:
 * providers translate every arm and consumers `switch` on `type` ending in `assertNever`, so a new
 * arm is a coordinated change across known packages rather than a plugin extension.
 */
type DecisionQuestion =
  | DecisionChoiceQuestion
  | DecisionScoreQuestion
  | DecisionBooleanQuestion
```

```ts type-equiv
/**
 * Pick one option from a named set. `criteria` maps each option id to the description the model
 * judges against; option ids are the consumer's own vocabulary and come back verbatim in the answer.
 */
interface DecisionChoiceQuestion {
  readonly type: 'choice'
  /** What the model is being asked to decide. */
  readonly instructions: string
  /** Option id to its description. At least two entries; a one-option choice has no decision to make. */
  readonly criteria: Readonly<Record<string, string>>
}
```

```ts type-equiv
/**
 * Place the state on an ordered rubric. `criteria` lists level descriptions from lowest to highest;
 * the answer may land between levels, so a score is continuous rather than an index.
 */
interface DecisionScoreQuestion {
  readonly type: 'score'
  /** What the model is being asked to rate. */
  readonly instructions: string
  /** Ordered level descriptions, lowest first. Between 2 and 10 entries. */
  readonly criteria: readonly string[]
}
```

```ts type-equiv
/** Judge whether one statement holds. The answer is the probability that it is true. */
interface DecisionBooleanQuestion {
  readonly type: 'boolean'
  /** The statement whose truth is being judged. */
  readonly instructions: string
}
```

```ts type-equiv
/**
 * One answer from a decision provider, keyed identically to its question. A CLOSED discriminated
 * union: the `type` always matches the question's `type`, so a consumer that asked a `choice` may
 * narrow on that arm.
 */
type DecisionAnswer =
  | DecisionChoiceAnswer
  | DecisionScoreAnswer
  | DecisionBooleanAnswer
```

```ts type-equiv
/** The selected option plus the full distribution that produced it. */
interface DecisionChoiceAnswer {
  readonly type: 'choice'
  /** The winning option id, always a key of the question's `criteria`. */
  readonly choice: string
  /** Probability per option id; keys match the question's `criteria`, values sum to approximately 1. */
  readonly probabilities: Readonly<Record<string, number>>
}
```

```ts type-equiv
/** A position on the rubric plus the distribution over levels. */
interface DecisionScoreAnswer {
  readonly type: 'score'
  /** Continuous position, `0` to `criteria.length - 1`; may fall between levels. */
  readonly score: number
  /** Probability per level index, keyed by its stringified index. */
  readonly probabilities: Readonly<Record<string, number>>
}
```

```ts type-equiv
/** The probability that the statement holds. */
interface DecisionBooleanAnswer {
  readonly type: 'boolean'
  /** Probability in `[0, 1]` that the statement is true. */
  readonly probability: number
}
```

```ts type-equiv
/**
 * What a decision provider is asked to evaluate: one state, and every question asked about it.
 * Questions are evaluated together because a provider prices and paces one call, not one question —
 * an extra question costs tokens but little additional time.
 */
interface DecisionRequest {
  /** The situation being judged, as plain text the provider reads verbatim. */
  readonly state: string
  /** Questions keyed by a consumer-chosen name; answers come back under the same keys. */
  readonly questions: Readonly<Record<string, DecisionQuestion>>
}
```

```ts type-equiv
/** The answers to one {@link DecisionRequest}, keyed identically to its `questions`. */
interface DecisionResult {
  /** One answer per question, under the request's own keys. */
  readonly answers: Readonly<Record<string, DecisionAnswer>>
  /** Id of the provider that answered, for session-log attribution. */
  readonly providerId: string
}
```

```ts type-equiv
/**
 * A decision-capable backend. Registered with `ctx.decision.registerProvider`. `id` is a stable
 * string, unique within the seam.
 */
interface DecisionProvider {
  readonly id: string
  /** Cheap local usability check; must not make network calls. */
  available(): boolean
  /** Answer every question in one request; honor `signal` for cancellation. */
  evaluate(request: DecisionRequest, signal?: AbortSignal): Promise<DecisionResult>
}
```

`criteria` carries the option id -> description map for a `choice` and the ordered level descriptions, lowest first, for a `score`; a `boolean` needs none. A `score` answer is continuous over `0..criteria.length - 1`, so it can land between levels, and its `probabilities` are keyed by level index.

Every answer carries a distribution. That is the point of the seam rather than an implementation detail: a consumer that acts above a confidence threshold and escalates below it bounds its own worst case, because a wrong-but-confident answer is the failure mode to design against and the distribution is what makes it visible.

## Request validation

The service validates before it selects a provider, so a malformed request fails identically whether or not a provider is mounted and a caller's bug never presents as an availability problem. The rules are structural: a request carries at least one question; a `choice` offers at least two options, because a one-option choice is not a decision; a `score` rubric holds 2 to 10 levels, because a finer rubric asks for a precision the answer cannot carry. Any violation raises `DECISION_REQUEST_INVALID`.

## Provider availability and selection

A provider reports `available()` from locally checkable state — a resolvable credential, a parseable endpoint, a non-empty model id — without performing I/O.

Selection is explicit or unambiguous, never positional. With the service's `provider` field set, that provider must be registered and available, or the call fails (`DECISION_PROVIDER_CONFIGURED_MISSING`, `DECISION_PROVIDER_CONFIGURED_UNAVAILABLE`) unless `fallbackProvider` is also configured and usable. `fallbackProvider` is invoked only after the primary cannot answer — a missing, unavailable, rate-limited, or failed primary — and never on a successful primary call. Cancellation (`DECISION_ABORTED`) does not trigger fallback. With nothing pinned, exactly one usable non-fallback provider auto-selects, zero raises `DECISION_PROVIDER_UNAVAILABLE`, and more than one raises `DECISION_PROVIDER_AMBIGUOUS` rather than resolving by registration order.

## Errors

| Code | Raised when |
|---|---|
| `DECISION_REQUEST_INVALID` | The request violates a structural rule above |
| `DECISION_DUPLICATE_PROVIDER` | A provider id is registered twice |
| `DECISION_PROVIDER_CONFIGURED_MISSING` | The pinned provider is not registered |
| `DECISION_PROVIDER_CONFIGURED_UNAVAILABLE` | The pinned provider is registered but not usable |
| `DECISION_PROVIDER_UNAVAILABLE` | No usable provider is registered |
| `DECISION_PROVIDER_AMBIGUOUS` | Several providers are usable and none is pinned |
| `DECISION_PROVIDER_ERROR` | A provider failed, or returned an answer that does not satisfy the question asked |
| `DECISION_RATE_LIMITED` | A provider's quota is exhausted; a consumer may fall back |
| `DECISION_ABORTED` | The caller's `AbortSignal` fired |

A provider that returns an answer missing its distribution, or omits a question that was asked, raises `DECISION_PROVIDER_ERROR` rather than having the gap filled in. A fabricated probability is indistinguishable from a calibrated one at a consumer's confidence gate, which is precisely the check the seam exists to support.

## Consumers

`dsh-decision-consumers` owns reusable classification functions and one optional Agent recovery binding. The binding handles configured `agent/request-error` codes, probes authorized provider/model routes concurrently, and asks the selected decision provider to choose among healthy workers. Takeover requires the judge to rate the winner's code validity above `broken-or-hallucinated`; the winning choice probability is reported as telemetry rather than gated on, because it measures how strongly the judge separated routes that the probes already proved reachable. Selection overrides the next `agent/request` and returns `{ kind: 'retry' }`; the ordinary `request/header` records the route used. Inconclusive arbitration delegates to the next recovery listener, normally the generic same-route retry.

The binding reads the enabled `subagent-model-selection.allowedModels` setting when present and otherwise uses its configured fallback candidates. Probes receive the bounded latest durable user message. Cancellation and plugin disposal abort and drain every active probe, and failed takeover routes stay excluded for that Agent step. One round is arbitrated per failed-route set, so repeated same-route failures during a step reuse the settled result. The [package reference](../../packages/decision/decision-consumers/README.md#live-model-failover) owns configuration and model-cost details.

## Trust and untrusted state

`state` is frequently built from content the harness does not control — a fetched page, a tool result, a file. A consumer must build its option set from its own snapshot and never from text the source supplied, so that content cannot extend or rename the options it is judged against. The seam does not enforce this: it cannot distinguish a consumer's option ids from ones a page suggested, so the obligation sits with each consumer.

<!-- BEGIN GENERATED cordis-surface (gen-cordis-catalog.ts) — do not edit between markers -->

<a id="cordis-surface"></a>

## Cordis API

Generated from source by `scripts/gen-cordis-catalog.ts` (verified fresh by `pnpm run verify-cordis-catalog` in doc-sync; regenerate with `pnpm run gen-cordis-catalog`) — the language sides differ only in locale-specific paired document paths. Signature blocks use a `ts cordis-catalog` fence and keep the original source JSDoc; dispatch modes are defined in the [primer](../cordis-primer.md#dispatch-modes), and the framework-inherited `ctx` API lives in [cordis-api/inherited.md](../cordis-api/inherited.md).

<a id="ctxdecision--decisionruntime"></a>

### `ctx.decision` — `DecisionRuntime`

The decision service. Registered as `ctx.decision` (one instance per context).

Selection semantics (resolved at execution time, never order-dependent):

- A configured id that is registered and `available()` → that provider.
- A configured id not registered → `DECISION_PROVIDER_CONFIGURED_MISSING`, unless `fallbackProvider` is registered and usable.
- A configured id registered but unavailable → `DECISION_PROVIDER_CONFIGURED_UNAVAILABLE`, unless `fallbackProvider` is registered and usable.
- A configured fallback id is invoked only after the primary cannot answer, never on a successful primary call.
- No id configured, exactly one registered usable non-fallback provider → that provider.
- No id configured, multiple usable non-fallback providers → `DECISION_PROVIDER_AMBIGUOUS`.
- No id configured, no usable provider → `DECISION_PROVIDER_UNAVAILABLE`.

```ts cordis-catalog
/**
 * Register a decision provider.
 * @param provider - the backend to register; its `id` must be unique within the seam.
 * @returns a disposer that removes this registration.
 */
registerProvider(provider: DecisionProvider): () => void

/**
 * Answer every question in one request through the selected provider, or trigger the fallback
 * provider on-demand if the primary fails to respond.
 * @param request - the state and the questions asked about it.
 * @param signal - optional cancellation for the provider call.
 * @returns the answers, keyed identically to `request.questions`.
 */
async evaluate(request: DecisionRequest, signal?: AbortSignal): Promise<DecisionResult>
```

Source: [`packages/decision/decision/src/index.ts`](../../packages/decision/decision/src/index.ts)
<!-- END GENERATED cordis-surface -->
