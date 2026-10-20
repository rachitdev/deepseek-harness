# Agent Note: JEV skill selection and fetch guardrail decision consumers

Status: implemented

English | [中文](2026-09-19-jev-skill-selection-and-fetch-guardrail-consumers.zh.md)

## Problem

Two high-frequency judgments in the harness still cost a full LLM turn or nothing at all. Skill catalog routing means the agent either burns a reasoning turn deciding whether a skill applies, or relies on the loop's implicit choice; the TypeSafe JEV decision seam, measured at 345–460 ms warm on this machine, is the cheaper place for that classification. And every `web_fetch`/`web_search` result enters model context unscreened: untrusted external text can carry instructions that the model cannot reliably distinguish from harness instructions, so the guard against injection was a system-prompt plea rather than an executable check.

The gap analysis against `Anil-matcha/awesome-jev-by-typesafe` (September 19, 2026 snapshot) confirmed DSH already implements nine of the catalog's coding-agent patterns — browser stepping, tool risk, model routing/cascade, retrieval filtering, agent-trace verdicts, loop control, pre-push gate triage, failover arbitration, and compaction scoring. Skill selection and input guardrails were the two patterns the catalog carries that DSH did not.

## Decision

Add two classification consumers to `@deepseek-ai/dsh-decision-consumers`, following the package's existing shape of exported pure functions over `ctx.decision`; no new seam surface, no new plugin:

- `selectSkill` (`src/skill-selector.ts`) — fan-out of `needs_skill` (Boolean), `skill` (Choice over the offered catalog plus `none`), and `risk` (three-level Score), with code-owned gates: catalog-empty decides locally as `direct_answer` without a provider call; a sub-threshold needs-skill probability returns `direct_answer`; `none` or a winning skill probability below `minSkillConfidence` (default 0.65) returns `clarify`; a risk score at or above `approvalRiskThreshold` (default 1.5) returns `load_skill_with_approval`.
- `guardFetchedContent` (`src/fetch-guardrail.ts`) — two Boolean questions (`prompt_injection`, `credential_harvest`) over the fetched URL, title, and a length-bounded excerpt; the verdict maps the larger hazard probability onto `pass`/`review`/`block` bands (defaults 0.30/0.70, individually configurable) and reports `thresholdMargin`, the distance from the band it was judged against.

## Alternatives considered

- **KV-cache style injection screening inside the LLM provider** — rejected: screening belongs at the capability that introduces untrusted text, and a decision answer keeps the policy branchable in code instead of a model-visible string.
- **A single Choice over guardrail verdicts** — rejected: a Choice collapses two independent hazards into one distribution; separate Nouls let a policy distinguish injection-only (strip and continue) from credential-harvest (block outright).
- **Skill selection inside `dsh-llm` as prompt guidance** — rejected: the decision seam exists precisely so cross-cutting classification does not become a model turn, and the confidence gate to `clarify` must be code-owned.
- **Detecting `other`/`unknown` skill options or calibration lock files (repo's `jevcal` pattern)** — deferred: the existing `none` option covers the incomplete-catalog case, and threshold calibration is repo-wide decision-seam work, not this consumer.

## Consequences

- Skill catalog routing can run at decision latency (measured live at ~1.1–1.2 s for a three-question, three-candidate call including process warm-up; a two-Boolean guardrail call ran 334–435 ms) instead of a full agent turn.
- Fetched web content gets a machine-checkable injection and credential-harvest screen before entering model context; callers own what a `block` verdict does.
- `selectSkill` remains a library function until a skill-routing owner supplies a bounded candidate shortlist. `guardFetchedContent` is wired into `dsh-tool-web` behind the opt-in `fetchGuardrail` config; the web profile enables it, while profiles without a decision provider retain the default unguarded behavior.
- The real-provider benchmark compares a DSH proposal against an 11-skill bounded catalog, expands disagreements to DSH's proposal plus JEV's high-probability skills, and classifies companion skills as parallel-now or later-depth. Three runs over five labeled cases improved primary accuracy from 60% to 100%, achieved a mean ground-truth routing score of 1.0, and measured 782 ms median total decision latency (348–1150 ms range).
- Known limitations carried into the package README: provider-relative calibration of the defaults, and the bounded excerpt bound (8000 characters) heeds the provider's context limits, not a semantic guarantee.
