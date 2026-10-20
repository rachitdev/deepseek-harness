---
description: "The LLM-backed decision provider: answers ctx.decision questions by constraining an existing chat model to a typed JSON reply, so the decision seam needs no dedicated credential."
kind: "package-reference"
---

# @deepseek-ai/dsh-decision-llm

English | [中文](README.zh.md)

## Summary

`dsh-decision-llm` answers [`dsh-decision`](../decision/README.md) questions with a model the composition already has, by constraining an ordinary chat model to a typed JSON reply and validating what comes back. It needs no decision-specific credential, so every consumer built on the decision seam works on a plain harness install and the seam is never a single-vendor dependency. Choose it as the default provider, as a fallback beside [`dsh-decision-jev`](../decision-jev/README.md), or as the way to develop a decision consumer before any gateway credential exists.

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

Mount the plugin next to `dsh-decision` and point it at a registered LLM route. It registers one provider under the id `llm`.

### When to choose it

Choose it whenever the deployment has a working LLM route and no dedicated decision credential, and pair it with a cheap, fast model rather than the conversation's main one: the questions are small and the reply is a few dozen tokens, so a premium model buys little here. Mounting it alongside `dsh-decision-jev` requires pinning one with the service's `provider` field, because the decision service refuses to guess between two usable providers.

### Minimal configuration

The route is required rather than guessed, because which model serves decisions varies per deployment and a wrong guess would fail at call time instead of at load.

```yaml
- name: '@deepseek-ai/dsh-decision'
- name: '@deepseek-ai/dsh-decision-llm'
  config:
    provider: deepseek
    model: deepseek-chat
```

| Field | Default | Meaning |
|---|---|---|
| `provider` | (required) | Registered LLM provider route serving decisions |
| `model` | (required) | Exact model id on that route |
| `maxTokens` | `1024` | Upper bound on the JSON reply |

The generated [configuration catalog](../../../docs/config-catalog.md#deepseek-aidsh-decision-llm) is the exhaustive source for every accepted field and its JSDoc.

<a id="understand-the-implementation"></a>
## Understand the implementation

The provider renders each request as a system prompt stating the required JSON format plus a user turn describing the state, every question, and — for a `choice` — the exact option ids and — for a `score` — the numbered levels the answer must use. It then streams one model call and assembles the text.

**The reply is a model-output boundary, so every field is validated.** A chat model can emit anything, so the parser checks each answer against the question that produced it: a `choice` must name an option the question actually offered, a `score` must fall within `0..criteria.length - 1`, every probability must be a finite number in `[0, 1]`, and a distribution must cover every option or level. Anything else fails as `DECISION_PROVIDER_ERROR` rather than reaching a consumer's confidence gate as though it were calibrated. A Markdown code fence around the JSON is tolerated, including an unterminated one, because models add them routinely and stripping one costs nothing.

<a id="further-exploration"></a>
## Further Exploration

- [`@deepseek-ai/dsh-decision`](../decision/README.md) — the Service Definition, question types, and selection policy.
- [`@deepseek-ai/dsh-decision-jev`](../decision-jev/README.md) — the faster, cheaper provider for deployments with a TypeSafe credential.
- [`@deepseek-ai/dsh-decision-laya`](../decision-laya/README.md) — the on-demand local Laya fallback, started only after the primary fails.

<a id="dev-note"></a>
## Dev Note

Validation deliberately refuses to repair a near-miss — an out-of-range score is not clamped and a missing option is not filled in with zero. Repairing would convert a model that misunderstood the question into a confident-looking answer, which is the one failure this seam's consumers cannot detect downstream.

<a id="model-experience"></a>
## Model Experience

None, as this decision provider runs auxiliary model calls whose prompts are built and discarded locally.

#### KV Cache effect

No effect; the auxiliary call runs on its own message array and does not touch the conversation prefix cache.


## Known Limitations and Deferred Work

- **Probabilities are stated, not calibrated.** A chat model reports numbers that read like probabilities without any guarantee they behave like them, and models tend to round toward `0` and `1`. A confidence threshold tuned against this provider should be re-checked after switching to a calibrated one, and vice versa.
- **No retry on a malformed reply.** A model that returns unparseable JSON fails the call rather than being asked again; a consumer that wants a second attempt must implement it.
- **Latency and cost track the chosen model.** This provider is the portability floor, not the fast path: the same question answered here costs a full model turn.
- **The reply must fit `maxTokens`.** A request carrying many questions with large option maps can exceed the bound and truncate into unparseable JSON; raise the bound or split the request.
