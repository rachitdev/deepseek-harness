# Agent Note: Dynamic browser action policy over one decision request

Status: implemented

English | [中文](2026-09-21-dynamic-browser-action-policy.zh.md)

## Problem

`decideEgoStep` asks one fixed `element` choice question and can only ever return a click. A page needing text entry or a native dropdown selection is unreachable through it, so the consumer cannot drive an ordinary search form. It also has no callers, and this repository ships no browser package: the browser actually used here is the external `ego-browser` CLI, driven through the Bash tool.

An earlier scan recorded `jev-ultrafast`'s speculative fan-out as already implemented and stopped there, which mistook one mechanism for the project's result. Its measured gain comes mostly from elsewhere — median browser protocol calls fell 1,092 → 101 and median task time 9.450 s → 7.092 s, driven by atomic DOM snapshotting and bounded waits rather than by fan-out. The part missing from this repository is the **dynamic action space**: an element table rebuilt on every observation, where each operation carries its own set of valid targets.

## Decision

`decideBrowserAction` asks one operation question plus one target question per operation the page currently offers, in a single request. Only the head naming the chosen operation is read; the rest cannot execute an action. Measurement against this account shows question count is close to free while request count is not, so unused heads are the cheap half of the trade.

An operation reaches the provider only when an observed control supports it. A page with no editable field never offers `TYPE_TEXT`, so the provider is never asked to choose among actions that cannot run. When exactly one control supports the chosen operation, no target question is asked: a one-option choice is not a decision and the seam rejects it, and choosing the operation already determined the target.

`confidence` below `targetConfidenceThreshold` (0.70) returns `executable: false` rather than throwing, so a caller observes again instead of acting on a weak choice.

The consumer owns no browser. It takes an already-observed element table and returns an operation and target id. Observation, target resolution back to a live node, freshness and occlusion checks, and TYPE_TEXT value generation stay with the caller.

## Measured results

Real provider, `tests/browser-action-policy.e2e.ts`, four labeled steps: 4/4 correct operation and target, 361–1224 ms per step. Two of the four are steps `decideEgoStep` cannot express — typing into the empty destination field of a partly filled form, and a `SELECT`-only control. The `DONE` case returned 0.72, above the threshold but well below the 1.00 of the three action cases, which is the expected shape: completion is a judgement about visible evidence, not a target lookup.

The upstream Python implementation was also run end to end on this machine against the live web: 3/3 verified click-only navigation at a median 3.385 s, and one full TYPE_TEXT task in 6.973 s with the text helper served by the local OmniRoute gateway.

## Alternatives considered

- **Extending `decideEgoStep`** — rejected. Its return type is a single click action with an element id; widening it to an operation union would change every existing field's meaning for no current caller, while the click-only stepping it documents remains valid on its own.
- **Porting the browser runtime too** — rejected for this change. The snapshot, wait, and occlusion logic is where most of `jev-ultrafast`'s speed lives, but it belongs to a browser package this repository does not have, and `ego-browser` already owns browser control here. The decision policy is the part that fits the existing seam.
- **Asking operation and target sequentially** — rejected on measurement: two requests cost roughly twice one, and a 48.6% latency reduction with 5/5 identical decisions was measured for the single-request form.
- **Letting the policy supply TYPE_TEXT values** — rejected. Decision answers carry no text by construction, and the upstream implementation deliberately fails rather than inventing a field value. Text generation stays with a caller's text model.

## Consequences

- A caller can now drive forms, not only links, through the decision seam.
- `decideEgoStep` remains for single-element click stepping; both are exported and neither supersedes the other.
- A page offering one operation with one control now resolves without a target question, so callers must read `targetId` rather than assuming a target head was asked.
- The consumer is a library function with no owner wiring it into a loop, as with `selectSkill` and `assessInstructionDrift`.
