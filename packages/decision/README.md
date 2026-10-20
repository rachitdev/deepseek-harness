---
description: "Package map for the decision capability family: the typed-judgement service and the provider backends that answer its questions."
kind: "package-group"
---

# decision/ — decision capability family

English | [中文](README.zh.md)

## Summary

The `decision/` group lets any plugin ask a typed question about unstructured state (pick an option, score against a rubric, test a statement) through one provider-neutral service, `ctx.decision`. Answers carry calibrated probabilities and no text, so consumers gate on confidence instead of parsing prose. `decision/` owns validation, primary/fallback provider selection, and errors. `decision-jev/`, `decision-laya/`, and `decision-llm/` are the backends: Jev over TypeSafe's API, a local Laya checkpoint started only after primary failure, and a chat model constrained to the same format. `decision-consumers/` packages the consumers. Text generation stays with the LLM capability.

## Table of Contents

- [Packages](#packages)
- [Related documentation](#related-documentation)
- [Dev Note](#dev-note)

-----

<a id="packages"></a>
## Packages

Five packages play the decision roles; the [subsystem reference](../../docs/subsystems/decision.md) owns the exhaustive vocabulary and contracts.

| Package | Role | ctx key |
|---|---|---|
| [`decision/`](decision/README.md) | Typed-judgement service: answer choice, score, and boolean questions through interchangeable backends, one validation, selection, and error policy | `ctx.decision` |
| [`decision-jev/`](decision-jev/README.md) | Answers through Jev on TypeSafe's direct System One API | registers on `ctx.decision` |
| [`decision-laya/`](decision-laya/README.md) | Starts a cached local Laya typed-decision checkpoint only after primary provider failure | registers on `ctx.decision` |
| [`decision-llm/`](decision-llm/README.md) | Answers by constraining a registered chat model to a typed JSON reply | registers on `ctx.decision` |
| [`decision-consumers/`](decision-consumers/README.md) | Suite of 11 decision consumers (browser stepping, tool gating, routing, filtering, compaction, stall detection, subagent verdicts, gate selection, skill selection, fetch guarding) | consumes `ctx.decision` |

<a id="related-documentation"></a>
## Related documentation

- [Decision subsystem reference](../../docs/subsystems/decision.md) — question and answer types, validation rules, selection policy, and the error table.
- [Capability seam](../../docs/glossary.md#capability-seam) — the Service Definition / Provider / Consumer split this group follows.

<a id="dev-note"></a>
## Dev Note

The backends do not share a transport, and that is why this capability is a seam rather than another `LlmService` route: TypeSafe and local Laya accept typed questions and return no text, while the LLM backend answers the same questions through an ordinary completion. Keeping all three behind one service lets a consumer be written once and use a remote primary, an on-demand local fallback, or a chat-model substitute.
