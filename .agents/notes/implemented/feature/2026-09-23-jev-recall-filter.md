# Agent Note: Jev recall filter — relevance and stored-injection screening of memory recall

Status: implemented

English | [中文](2026-09-23-jev-recall-filter.zh.md)

## Problem

The OpenViking memory plugin appends a `<openviking-context>` recall block to every Agent step. Its vector ranking scores almost every item 0.77–0.93, so the block carries whatever is nearest in embedding space: on 40 recorded recalls from this workstation, TypeSafe Jev rated 255 of 419 items (54% of recalled characters) below "tangential" for the request they were attached to. Recall is also an unscreened channel for stored prompt injection: text captured into memory from a fetched page or tool output returns in later sessions as trusted-looking context. `filterRetrievalCandidates` existed but had no caller, and only `web_fetch` results were screened for injection.

## Decision

Add a live consumer to `@deepseek-ai/dsh-decision-consumers`, enabled by `recallFilter.enabled`:

- `screenRecallItems` asks one `rel_<n>` Score question (0..3 rubric `irrelevant`/`tangential`/`useful`/`essential`) and, when `screenInjection` is on, one `inj_<n>` Boolean question per item, in concurrent batches of `batchSize`. State holds the user request, the previous assistant reply (`maxContextChars`), and each item's uri plus bounded body.
- `installRecallFilter` registers an `agent/pre-step` listener that matches recall messages by `source.plugin`/`source.form`, parses `<memory uri>` elements, and replaces the message with a re-rendered block: items below `minRelevance` are omitted with one count note, items at or above `injectionThreshold` become `screened="withheld: …"` stubs that keep only the uri.
- Any provider failure, malformed answer, or `timeoutMs` expiry returns the original recall and logs a warning; recall is never lost because Jev is slow or down.

Choices pinned by measurement against live TypeSafe:

- **No `keep` Boolean.** On 151 items it correlated 0.96 with the Score and never kept an item the Score dropped, but it dropped 15 items the Score rated useful or better. The Score alone decides.
- **Defaults `minRelevance` 0.5, `injectionThreshold` 0.7.** Planted attacks scored 0.75–0.99; benign security notes and all 419 recorded items scored at most 0.16.
- **Previous-reply context.** A quarter of recorded queries are follow-ups such as "go ahead". With the previous reply in state, 11 of 233 items on short queries that the request alone rated irrelevant were rescued (for example a code-signing preference for "go ahead test it out"), so context is on by default.
- **Rejection split.** TypeSafe's gateway firewall answers HTTP 403 for state containing attack-shaped text such as `curl … | sh`, surfaced as `DECISION_REQUEST_REJECTED`, which fails the whole batch. A rejected batch is retried item by item, and an item refused alone is withheld by default (`withholdUnscreenable`): refusal was only observed on planted attacks.
- **Listener order.** OpenViking appends recall after its own `next()` in a `prepend` listener, so the filter must be outermost. The Loader applies entries with `Promise.allSettled`, so registration order across bundles is not deterministic; the filter re-registers its `prepend` listener on every `agent/session-start`, which runs after every boot-time registration. Removing that re-registration fails the ordering tests.

## Alternatives considered

- **Patch the vendored OpenViking plugin to call the filter** — rejected: vendor packages are not patched locally, and the filter would then serve only one memory source.
- **Raise OpenViking's `scoreThreshold`** — rejected: vector similarity is flat across relevant and irrelevant items (0.77–0.93), so no threshold separates them; it also does nothing for injection.
- **Fail closed on Jev errors** — rejected as the default: a Laya cold start or network error would strip all memory from a turn. Injection is still caught on the success path, which is the measured common case.
- **A second decision request for injection** — rejected: extra questions cost no latency while extra round trips do, so both questions share one request.

## Consequences

- Each step with recall adds one decision round trip (measured p50 0.43 s, p95 0.53 s through the TypeScript path; up to 0.85 s p95 on larger recalls) and removes about half of recalled characters from the model request.
- The model sees the filtered block, which is also what the session log records; withheld items remain discoverable by uri through memory search.
- The profile message (`form: instructions`) is not screened.

## Verification

- Unit and live-listener tests in `tests/recall-block.spec.ts`, `tests/recall-screen.spec.ts`, and `tests/recall-filter.spec.ts` cover parsing, batching, rejection split, fail-open, timeout, disabled mode, disposal, and context; the three source files hold 100% statement and branch coverage.
- `tests/recall-filter-loader.spec.ts` boots a Loader composition with the recall source listed after `decision-consumers` and asserts the durable session log holds the screened block.
- A live A/B through `JevDecisionProvider` on 40 recorded recalls with 20 planted attacks and 20 benign notes: 18 attacks withheld by score, 2 refused by the gateway and withheld, 0 benign or recorded items withheld, 0 failed calls.
- An isolated `jev-recall` profile run with live OpenViking and TypeSafe returned the correct answer with 3 kept items and 12 omitted.
