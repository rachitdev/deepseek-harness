# dsh-rs parity plan

## Goal

dsh-rs (RS) serves every endpoint, unary and stream, that the TypeScript DeepSeek Harness (TS) serves to its Web GUI, and hosts every TS server-side plugin. No endpoint is excluded.

Baseline: TS checkout `~/code/deepseek-harness` at tag `dsh-v0.2.2-alpha.2` (199 endpoints: 178 unary, 21 stream). `tools/parity-report.py` prints the current counts.

## Terms

- **DeepSeek Harness**: an agent harness whose features are plugins on a shared `Context` (the Cordis kernel). TS is the reference implementation; RS is this crate.
- **Endpoint**: one named remote call the Web GUI makes to the host, for example `session/prompt`. **Unary** returns one reply; **stream** keeps a WebSocket stream open and pushes frames.
- **Gateway**: the host's HTTP/WebSocket entry (`src/gui/server.rs`, `Gateway::route`) that checks the token, checks argument fields, and calls `Router::dispatch`.
- **Slice**: one group of endpoints built, tested and committed together.
- **Plugin**: a TS package loaded into the TS host. It cannot run in the Rust kernel; see slice 11.

## Source of truth

- `docs/gui-endpoints.json`: the endpoint inventory, generated from each TS package's `lib/typert.remote-client.js` descriptor. Regenerate: `cd <ts checkout> && node <dsh-rs>/tools/gui-endpoints.mjs --root . --out <dsh-rs>/docs/gui-endpoints.json`. The absolute `--root` form fails; run from the TS checkout with `--root .`.
- `docs/parity-missing.txt`: endpoints RS does not serve. `tests/parity.rs` fails when this list and `Router::endpoints()` plus `STREAM_ENDPOINTS` disagree, so a slice cannot land without shrinking the list and an upstream addition cannot pass unnoticed. When TS moves, regenerate the inventory and add the new names to the list in the same commit.
- `src/gui/descriptor.rs`: the inventory also makes the gateway refuse unknown argument fields on every endpoint, served or not.

## What the TypeScript Web host mounts

`tests/oracle/mounted.mjs` asks a running TS Web host (default composition) which inventory endpoints it answers; an unmounted endpoint gets a bare HTTP 404 `not found`. Result for the endpoints RS did not yet serve (`docs/parity-mounted.txt`, 2026-10-29): 47 unary endpoints are mounted, 53 are not, and 13 are streams (not probed).

- **Mounted, built first**: schedule (4), terminal (6), account (9), pluginManager (15), dynamicCordisRunner (10), officeToPdf (2), directoryPicker/pick.
- **Not mounted on the Web host**: cotTranslation, productAnalytics, speech, pluginMarket, pluginRecommendations, cuaAccess, forgePanel, liveshareShares, actionCards, claudeCodeMods. They belong to experimental or Desktop compositions. They are still in scope (no exclusions), but a Web-composition RS already matches TS by not serving them; they come after the mounted ones, and each needs its plugin composed to be testable against TS.
- A difference to close later: an unmounted endpoint answers TS `404 not found` (plain text), RS answers `200` with `gateway/lookup-not-found`.

## Verification layers

All five are required for a slice to be done.

1. **Gateway tests**: `tests/<slice>.rs` calls the endpoint through `Gateway::route`, covering the success path and every failure code.
2. **Ratchet**: `cargo test --test parity` is green with the slice's names removed from `docs/parity-missing.txt`.
3. **Differential run**: start TS (`pnpm dsh web`, port 3080) and RS (`dsh serve --profile <file>`, port 3081); send the same request sequence to both; replies must match with object key order ignored. Each difference is recorded in `docs/STATUS.md` under Known limits. Request sequences live in `tests/oracle/`.
4. **Harness2 rollout**: the Harness2 MCP tool `harness_rollout` runs `dsh run --prompt` in an isolated `DSH_HOME`. Set `dsh_bin` to the RS binary through a wrapper that supplies a model route (`/tmp/dsh-rs-egm/dsh-wrap.sh`: sets `DSH_CONFIG`, drops `--print-json`). The rollout must succeed with the slice's plugin composed.
5. **EGM review**: the Elephant-Goldfish Model review (`egm_review_run` in `skills4Harness/egm-mcp`) over the slice's design note, using the same wrapper as `DSH_BIN`. Every finding of severity high is fixed or dismissed with a written reason in the note before the slice is marked done.

## Slices

Order: slices that reuse existing RS machinery first, then slices that need a new store, then slices that need new processes or platform integrations. Slice 0 is a design gate and runs first because slices 10, 11 and 13 depend on it.

Counts are `missing of total` from the inventory.

| # | Slice | Missing endpoints | Count | Needs | Depends on | Status |
|---|-------|-------------------|-------|-------|------------|--------|
| 0 | Sidecar design note | none | 0 | decision: protocol, trust boundary, lifecycle, Node version | none | not started |
| 1 | session read API | session/state, events, wait; observeState, observeEvent, observeProjections (stream) | 0 of 25 (was 6) | existing log and inbox projections | none | done |
| 2 | schedule | schedule/delete, deleteInactive, history, update (+ tools, driver) | 0 of 6 (was 4) | durable reminder store, timer driver | none | done |
| 3 | credentials write | credentials/set, unset | 0 of 3 (was 2) | credential document | none | done |
| 4 | subagents | subagents/prompt, interruptByParent | 0 of 2 (was 2) | existing subagent plugin | none | done |
| 5 | jobs | job/list, job/follow (streams) | 2 of 3 | background job registry | none | not started |
| 6 | analytics and translation | productAnalytics enabled, report, watchPolicy (stream); cotTranslation limits, translate | 5 of 5 | policy setting, HTTP translators | none | not started |
| 7 | terminal | create, list, close, rename, shells, environment, attach (stream), retain (stream) | 8 of 8 | pty crate | none | not started |
| 8 | picker and file watch | directoryPicker/pick; workspaceFiles/changes (stream) | 2 of 8 | native chooser, fs watcher | none | not started |
| 9 | account | 9 served (signed-out host); `startSignIn` remains | 1 of 11 | Platform OAuth sign-in | none | partly done |
| 10 | plugin manager, market, recommendations | pluginManager 15, pluginMarket 4, pluginRecommendations 1 | 20 of 20 | package installer, registry client | slice 0 | not started |
| 11 | dynamicCordisRunner | 10 endpoints | 10 of 12 | Node sidecar host for TS plugins | slice 0 | not started |
| 12 | speech and officeToPdf | speech 6 (one stream), officeToPdf 2 | 8 of 8 | speech provider, document converter | none | not started |
| 13 | platform integrations | cuaAccess 9, forgePanel 18, liveshareShares 12, actionCards 1, claudeCodeMods 2, toolResults 1, userQuestions/attachWait 1 | 44 of 44 | each integration's service; may reuse the sidecar | slice 0 | not started |

Total missing: 123 (`tools/parity-report.py`). Slices 1 to 13 sum to 123.

`schedule/list` and `schedule/catalog` are already served as honest empty lists; slice 2 replaces them with real data.

## Definition of done per slice

- All five verification layers pass.
- The slice's endpoint names left `docs/parity-missing.txt`.
- `docs/STATUS.md` records the slice, its differences from TS, and its test counts.
- `cargo test` is green.
