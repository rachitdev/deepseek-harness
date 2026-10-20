---
description: "The Jev decision provider: answers ctx.decision questions through TypeSafe's direct System One API, returning calibrated probabilities without generating text."
kind: "package-reference"
---

# @deepseek-ai/dsh-decision-jev

English | [中文](README.zh.md)

## Summary

`dsh-decision-jev` answers [`dsh-decision`](../decision/README.md) questions through Jev on TypeSafe's direct System One API. Jev is built for typed judgements rather than conversation: it returns a selection, a score, or a probability with a calibrated distribution, and generates no text at all. Choose it when a deployment has a TypeSafe credential and wants decisions answered faster and more cheaply than a chat model can. Mount it alongside the decision service; it registers no model-facing tool and answers only what a consumer asks.

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

Mount the plugin next to `dsh-decision` and give it a TypeSafe credential. It registers one provider under the id `jev`, which auto-selects when it is the only usable provider mounted.

### When to choose it

Choose it when the composition holds a TypeSafe key and its decisions are latency- or cost-sensitive. It is measurably faster and cheaper than routing the same question through a frontier chat model because the endpoint does no text generation. Do not choose it as a sole provider for a deployment that cannot hold a TypeSafe credential — mount [`dsh-decision-llm`](../decision-llm/README.md) instead or as well, and pin one with the service's `provider` field.

### Minimal configuration

The key is read through a credential *reference*, so it is named rather than embedded and a rotated key reaches the next call without a restart.

```yaml
- name: '@deepseek-ai/dsh-decision'
- name: '@deepseek-ai/dsh-decision-jev'
```

| Field | Default | Meaning |
|---|---|---|
| `apiKeyEnv` | `TYPESAFE_API_KEY` | Environment variable naming the TypeSafe key |
| `apiKey` | (unset) | Literal key; wins over `apiKeyEnv`, for throwaway local runs only |
| `baseURL` | `$TYPESAFE_BASE_URL`, else `https://api.typesafe.ai` | TypeSafe API endpoint base |
| `model` | `jev-latest` | Direct TypeSafe model alias; `$TYPESAFE_MODEL` can override it when the field is unset |

The generated [configuration catalog](../../../docs/config-catalog.md#deepseek-aidsh-decision-jev) is the exhaustive source for every accepted field and its JSDoc.

Key resolution takes a literal `apiKey` first, then the `credentials` seam when one is mounted, then the launch environment. The provider reports itself available when any of those three could supply a key, and fails loudly with `DECISION_PROVIDER_ERROR` if none actually does at call time — a key revoked between mount and call surfaces as an error, never as an unauthenticated request.

<a id="understand-the-implementation"></a>
## Understand the implementation

TypeSafe's System One endpoint is not chat-completions shaped, which is the concrete reason this capability is a separate seam rather than another `LlmService` route. Requests go to `/v1/systemone` with bearer authentication; the JSON body carries `model`, `state`, and `questions`.

**Vendor vocabulary stays inside this package.** TypeSafe names a choice's option map and a score's ordered levels with the same key, `criteria`, and calls the probability-of-truth arm `noul`. `src/types.ts` describes that wire format and `provider.ts` translates between `boolean` and `noul`, so the seam's vocabulary never leaks vendor naming to consumers.

**A missing field is an error, not a default.** When TypeSafe returns an answer without its distribution, or omits a question that was asked, the call fails. A fabricated probability would be indistinguishable from a calibrated one at a consumer's confidence gate, which is exactly the check this seam exists to support.

**Rate limiting is reported distinctly.** HTTP 429 raises `DECISION_RATE_LIMITED` rather than the generic provider error, so a consumer can recognize an exhausted quota and fall back rather than treat the decision as unanswerable. Cancellation raises `DECISION_ABORTED` from every await point, including mid-body reads, so an aborted call is never misreported as a provider failure.

**A rejected request is distinguished from a failed provider.** Any other 4xx raises `DECISION_REQUEST_REJECTED`: an unusable key (401), a malformed body, or a `state` above the endpoint's token ceiling (400 `max_tokens_exceeded`) is a property of the request, and the decision service therefore surfaces it instead of retrying the identical request on the fallback provider. 5xx keeps `DECISION_PROVIDER_ERROR`, so a server-side failure still falls back. This matters most with `dsh-decision-laya` configured, where a pointless fallback loads a local model before reaching the same rejection.

<a id="further-exploration"></a>
## Further Exploration

- [`@deepseek-ai/dsh-decision`](../decision/README.md) — the Service Definition, question types, and selection policy.
- [`@deepseek-ai/dsh-decision-laya`](../decision-laya/README.md) — the on-demand local Laya fallback, started only after this provider fails.
- [`@deepseek-ai/dsh-decision-llm`](../decision-llm/README.md) — the keyless alternative, usable as a fallback.
- `tests/gateway.e2e.ts` — direct TypeSafe coverage; self-skips without `$TYPESAFE_API_KEY`.

<a id="dev-note"></a>
## Dev Note

The unit suite stubs `fetch`, which cannot catch the request format drifting. `tests/gateway.e2e.ts` therefore calls the direct TypeSafe service and self-skips without a key, so the `/v1/systemone` path, bearer authentication, body-level model id, and `noul` response arm keep a real check without making the default suite require a credential.

<a id="model-experience"></a>
## Model Experience

None, as this decision provider calls an external evaluation endpoint that contributes nothing to the conversation model context.

#### KV Cache effect

No effect; the evaluation call runs outside the conversation context and invalidates no prefix cache.


## Known Limitations and Deferred Work

- **Account rate limits remain an independent dependency.** TypeSafe reports HTTP 429 when an account exceeds its limit; the provider raises `DECISION_RATE_LIMITED`. A consumer that decides per step in a loop needs enough direct TypeSafe capacity or a mounted fallback.
- **Measured latency depends on network distance.** Treat the direct TypeSafe measurement for a deployment's own region as authoritative and re-measure after changing endpoint, account tier, or model rather than carrying forward Vercel gateway figures.
- **Accuracy is not assumed equal to a frontier model.** Independent testing has shown Jev missing a defect that a frontier model caught. Consumers should gate on the returned probability and escalate below threshold rather than treat any single answer as authoritative.
- **Answers carry no text**, so this provider can never explain a decision. When a consumer needs a rationale for a user, it must produce one another way.
- **Page-derived state is untrusted input.** A consumer that feeds scraped content into `state` must build its option set from its own snapshot, never from text the page supplied, or a page can influence the option list it is judged against.
