# Agent Note: Always-filled context budget and on-demand capability gating

Status: proposed

English | [中文](2026-08-30-always-filled-context-budget.zh.md)

## Problem

A fresh standard-profile session in this repository starts its first model request at 26,988 input tokens against a 262,144-token window — about 10.3% of the context is filled before the user types a word. The measurement comes from this session's own log (session `8be52d15…`): the first `assistant/message` usage record and the logged `request/header` with its assembled system prompt and 48 tool schemas, calibrated at 4.45 chars/token against the reported usage.

Measured composition of that baseline:

| Component | ~Tokens | Share |
|---|---:|---:|
| Tool schemas (48 tools) | 10,900 | 40% |
| System prompt (identity, persona, tool policies) | 5,600 | 21% |
| Skill catalog (62 skills, full descriptions) | 5,400 | 20% |
| Workspace instructions (repo + home AGENTS.md) | 4,300 | 16% |
| Memory preview, runtime snapshot, user message | 790 | 3% |

Tool families within the schemas: core file/edit/web/bash 2.0k, meta (ask/plan/todo/skill) 0.8k, orchestration (agents/jobs/workflow/ralph) 2.8k, cordis dynamic-plugin 1.7k, goals 0.5k, OpenViking MCP 3.1k. Roughly 0.7k of the system prompt restates the dynamic-plugin tutorial that the `cordis-plugin-development` skill already carries.

Provider prompt caching recovered ~96% of the prefix from the second step of the measured turn, so the direct API cost of the baseline is cache-rate after the first request. The remaining costs are real: subscription rate-limit windows meter input tokens, time-to-first-token scales with prompt length even on cache hits, and ~17k of standing boilerplate precedes every user message. No policy or measurement protocol owns these costs today; the always-filled context is the emergent sum of plugin composition.

The skill system already implements lazy loading (catalog at session start, full text on load), and dynamic Cordis plugins implement mid-session capability growth. What is missing is a budget the composition must meet and an on-demand path for capability groups that are hidden rather than loaded.

## Proposal

Adopt an always-filled context budget with measured acceptance criteria, delivered in three layers. This note owns the policy and the repo-side changes; the deployment preset and session plugin are out-of-repo operationalizations recorded here as context.

1. **Policy.** The standard profile's always-filled baseline targets ≤ 14.3k tokens (from the measured 27.0k), and per-request growth stays bounded by the per-family and per-step budgets below. The budget is enforced by measurement, not runtime rejection.
2. **Layer A — deployment preset (config-only, ~27.0k → ~16.7k).** A lean agent preset mounts the core loop, file/shell/web tools, the cordis family (so mid-session growth through `cordis_define` remains possible), and a curated ~12-skill root; it omits orchestration, goals, and the OpenViking MCP client.
3. **Layer B — session-scoped gate (dynamic plugin, ~27.0k → ~14.3k).** A waterfall listener on `system-prompt/assemble` filters visible tools to the core and meta families plus two meta-tools: `enable_tools(group)` makes the next request carry a gated family's schemas (sticky until disabled), and `find_skill(query)` performs OpenViking semantic search over skill descriptions in place of the 62-entry catalog, which collapses to a one-line index.
4. **Layer C — repository changes (upstream, ~27.0k → ~10.5k with A+B semantics shipped).** Three changes: a two-tier skill catalog (name plus ≤8-word tag in the catalog prompt; full descriptions through a describe/search mode of the skill tool), folding the dynamic-plugin tutorial out of the system prompt into the existing skill, and MCP per-server lazy tool exposure (one dispatcher tool per server instead of one schema per operation).

## Measurement protocol

- **Baseline.** Fresh session; read the first `assistant/message` `usage.inputTokens` and the `request/header` component sizes from the session log (zstd JSONL).
- **Growth.** Input-token delta per step over the same log.
- **Capability regression.** ~12 scripted tasks, half requiring gated families; gated success rate must equal ungated, and the count of under-summon retries is the reported metric.
- **Cache behavior.** Cache-read share across a 10-step session; group flips bounded at 2 per task.

## Alternatives considered

- **Do nothing; rely on prompt caching.** Caching already makes the baseline cheap per request, but the five-hour subscription windows meter input tokens, TTFT grows with prompt length on every request, and the baseline still costs full price on the first request of every session. Rejected as a policy because nothing owns the number.
- **Truncate workspace instructions globally.** The documentation standard keeps standing orders in root and subtree AGENTS.md as budgeted tiers, and the repo-specific block is absent in non-repo workspaces. Pruning it trades real guidance for ~4.3k tokens. Rejected.
- **Hide tools with no summon path.** A config-only tool removal without the `cordis_define` catch or `enable_tools` leaves the model unable to request capability it cannot see, failing heavy tasks without a diagnostic. Rejected; both layers keep a summon path.

## Acceptance criteria

1. A keyless recorded-session snapshot carries the shipped profile's first-request `inputTokens` and the catalog/tool character budget, and fails when a change pushes the always-filled baseline above the policy target.
2. After the two-tier catalog, the catalog prompt block stays ≤ 6k characters with all 62 skills present, and every full description is reachable through one skill-tool call.
3. The system prompt's dynamic-plugin material stays ≤ 1k characters, and the rest remains loadable from the skill catalog by name.
4. OpenViking MCP contributes one dispatcher tool per server; all 15 operations remain reachable through it.
5. The regression eval reports equal task success gated and ungated, and ≤ 1 under-summon retry per task on average.

## Risks

- **Under-summoning.** The model cannot ask for capability it cannot see; the standing system line enumerates gated groups, and the under-summon retry count is the measured counter.
- **Cache churn.** Each `enable_tools` flip rebuilds the prompt-cache prefix once; groups are sticky and flips bounded at 2 per task.
- **Discoverability regression.** Replacing the catalog with semantic recall loses serendipitous skill discovery; the one-line index keeps names visible and `find_skill` carries the recall path.
- **Model-visible ⟺ logged.** Any on-demand injection must land through logged mechanisms (spliced inbox events); gating must not bypass the session log.
- **Downstream updates.** The two-tier catalog and tutorial fold change model-visible text, so recorded snapshots, the Model Experience sections of package READMEs, and the [skill-system note](../../archived/feature/2026-07-05-skill-system.md) update in the same PRs, as does the [projected-token-usage note](../../implemented/architecture/2026-07-29-projected-token-usage-and-request-context.md) if the usage event shape moves.
