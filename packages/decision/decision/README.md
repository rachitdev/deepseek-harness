---
description: "The decision service (ctx.decision): how plugins ask typed, probability-carrying questions about a situation through interchangeable providers, with one selection policy and error vocabulary."
kind: "package-reference"
---

# @deepseek-ai/dsh-decision

English | [中文](README.zh.md)

## Summary

Any plugin can ask a typed question about a situation (pick an option, score against a rubric, test a statement) through `dsh-decision` (`ctx.decision`) and get back calibrated probabilities instead of prose. The service picks one usable provider per call, so callers never track which model answers. Choose it when code must branch on a judgement it cannot compute, such as whether a tool call is risky. The service makes no network calls and registers no model-facing tool, so a provider must be mounted first; every question type shares one selection policy, error vocabulary, and configuration.

## Table of Contents

- [Use this package](#use-this-package)
- [Understand the implementation](#understand-the-implementation)
- [Further Exploration](#further-exploration)
- [Model Experience](#model-experience)
- [Known Limitations and Deferred Work](#known-limitations-and-deferred-work)
- [Dev Note](#dev-note)

-----

<a id="use-this-package"></a>
## Use this package

A composition that needs typed judgements loads the `dsh-decision` service and mounts at least one backend, and plugin authors then call `ctx.decision.evaluate()` directly. The service resolves the backend for each call, so callers never see provider ids unless they configured one.

### When to choose it

Choose the service when code must branch on a judgement about unstructured state and the answer is a selection, a rating, or a truth probability rather than text. Do not choose it to generate prose, names, or commit messages: a decision answer carries no strings. You also do not need it when the branch is computable — a path comparison or a schema check is cheaper and exact.

The reason this seam exists rather than another `LlmService` call is that its two shipped providers do not share a transport. `dsh-decision-jev` posts to an evaluation endpoint that takes no messages and returns no text, so it cannot be expressed as a chat completion; `dsh-decision-llm` constrains an ordinary chat model to the same answer format. Callers that ask a question through this seam work against either.

### Minimal configuration

Load the service and let a single mounted backend auto-select, or pin a provider id with `provider`. The environment variable `$DSH_DECISION_PROVIDER` feeds the same field and is not a separate priority chain.

```yaml
- name: '@deepseek-ai/dsh-decision'
- name: '@deepseek-ai/dsh-decision-jev'
```

| Field | Default | Meaning |
|---|---|---|
| `provider` | (unset) | Pinned provider id; unset auto-selects when exactly one is usable |
| `fallbackProvider` | (unset) | Provider id invoked only after the primary cannot answer; never consulted on a successful primary call |

The generated [configuration catalog](../../../docs/config-catalog.md#deepseek-aidsh-decision) is the exhaustive source for every accepted field and its JSDoc.

### Asking a question

`evaluate()` takes a `state` string describing the situation and a map of named questions, and answers every question in one round trip. Each answer arrives with a probability distribution, so a caller can gate on confidence rather than accept a bare verdict.

```text
const result = await ctx.decision.evaluate({
  state: snapshotOfThePage,
  questions: {
    element: { type: 'choice', instructions: 'Which element advances the task?', criteria: { '1': 'Buy now', '2': 'Add to cart' } },
    risk: { type: 'score', instructions: 'How destructive is this action?', criteria: ['safe', 'caution', 'destructive'] },
    done: { type: 'boolean', instructions: 'Is the task already complete?' },
  },
})
```

Three question types cover every judgement this seam supports:

| Type | `criteria` | Answer |
|---|---|---|
| `choice` | option id → description map (2 or more) | winning `choice` plus `probabilities` over every option |
| `score` | ordered level descriptions, lowest first (2–10) | continuous `score` from `0` to `criteria.length - 1`, plus `probabilities` over level indices |
| `boolean` | (none) | `probability` that the statement is true |

Asking several questions together is the intended use: one call costs one round trip regardless of how many questions it carries, so a consumer that already pays for a decision should ride its other questions along rather than issue a second call.

### Gating on confidence

A `choice` answer's `probabilities` are the reason to use this seam instead of a coin flip. A consumer that acts only above a threshold and escalates below it bounds its own worst case: a wrong-but-confident answer is the failure mode to design against, and the distribution is what makes it visible.

<a id="understand-the-implementation"></a>
## Understand the implementation

`DecisionRuntime` is a Cordis service registered as `ctx.decision`. It holds a provider registry keyed by id, validates every request before touching a provider, and selects one provider per call.

**Validation precedes selection.** A malformed request fails as `DECISION_REQUEST_INVALID` whether or not any provider is mounted, so a caller's bug never presents as an availability problem. The rules are structural: at least one question; a `choice` needs at least two options, because a one-option choice is not a decision; a `score` rubric holds 2 to 10 levels, because a finer rubric asks for a precision the answer cannot carry.

**Selection is explicit or unambiguous.** With `provider` configured, that provider must be registered and available, or the call fails loudly (`DECISION_PROVIDER_CONFIGURED_MISSING` / `_CONFIGURED_UNAVAILABLE`) unless `fallbackProvider` is also configured and usable. `fallbackProvider` is not a silent substitute: the service invokes it only after the primary cannot answer, and never on a successful primary call. With nothing configured, exactly one usable non-fallback provider auto-selects; zero fails as `DECISION_PROVIDER_UNAVAILABLE` and more than one fails as `DECISION_PROVIDER_AMBIGUOUS` rather than picking by registration order.

**A fallback runs only when the primary failed to serve an answerable request.** A primary raising `DECISION_REQUEST_REJECTED` rejected the request itself — an unusable credential, a malformed body, or a state above that provider's token ceiling — and the fallback would receive the identical request and reach the identical rejection. The service surfaces it instead, so a deterministic failure is not paid for twice, which matters most when the fallback loads a local model to answer. Providers own that classification, because only they see the backend's status.

**Registration is an effect.** `registerProvider()` returns a disposer and rejects a duplicate id (`DECISION_DUPLICATE_PROVIDER`). A stale disposer from a replaced registration does not remove its successor.

<a id="further-exploration"></a>
## Further Exploration

- [`@deepseek-ai/dsh-decision-jev`](../decision-jev/README.md) — the TypeSafe Jev provider, for deployments with a TypeSafe credential.
- [`@deepseek-ai/dsh-decision-laya`](../decision-laya/README.md) — the on-demand local Laya fallback, started only after the primary fails.
- [`@deepseek-ai/dsh-decision-llm`](../decision-llm/README.md) — the keyless provider that constrains an existing chat model, so the seam is never a single-vendor dependency.
- [Capability seam](../../../docs/glossary.md#capability-seam) — the Service Definition / Provider / Consumer split this package's group follows.

<a id="dev-note"></a>
## Dev Note

Provider selection is deliberately loud in both directions. An ambiguous auto-selection is an error rather than a first-match, because the alternative silently makes registration order a configuration surface — and the resulting behavior change on an unrelated reorder is far harder to diagnose than a message naming both candidates.

<a id="model-experience"></a>
## Model Experience

None, as this typed-judgement service delegates all prompt construction and execution to registered providers and their callers.

#### KV Cache effect

No direct effect; consumers that branch on decision results own any subsequent prompt changes.


## Known Limitations and Deferred Work

- **No answer generates text.** Every arm returns a selection, a number, or a probability by construction. Prose, names, and commit messages stay with `LlmService`; that is a property of the seam, not a gap to close.
- **No caching or batching across calls.** Two consumers asking about the same state in one turn pay two round trips. Riding extra questions along a call that already runs is currently the only sharing mechanism.
- **No cross-provider calibration guarantee.** A `0.8` from one provider and a `0.8` from another are not promised to mean the same thing, so a threshold tuned against one provider should be re-checked after switching.
- **Selection is per-call, not per-question.** A request cannot route its `boolean` questions to one provider and its `choice` questions to another.
