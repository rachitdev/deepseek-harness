# dsh-rs live status

Updated by the agent after each milestone. Local commits only; nothing is pushed.
Legend: [x] done and tested, [~] in progress, [ ] not started, [!] blocked.

Last verified 2026-10-27 14:55 against the running Rust GUI (port 3081): the official GUI
inventory holds 146 endpoints (131 unary, 15 stream). 64 of the 131 unary endpoints are served
and 67 are missing. Full `cargo test` = 660 passed, 0 failed (slash commands added `commands/execute` and a real `commands/list`; image attachments added `session/attachment`; file uploads added the raw `POST /api/session/uploadFileBinary` route).
`port-jincdream` (never pushed). The TS Web GUI on 3080 (PID 7952) is untouched.

A lookup parameter's wire name is its key plus `Id` (`agentId`, `workspaceFileScopeId`); the
inventory in `docs/gui-endpoints.json` lists only the TypeScript parameter name. The live TS host
rejects the bare name with `gateway/arguments-invalid`. Non-lookup parameters keep their plain
names. Endpoints not yet served that will need `agentId`: `dynamicCordisRunner/*`, `fileUploads/upload`, `terminal/*`.

## Parity campaign (2026-10-29)
Inventory refreshed to TS `dsh-v0.2.2-alpha.2`: 199 endpoints (178 unary, 21 stream). `docs/parity-plan.md` is the plan, `docs/parity-missing.txt` the ratchet (`tests/parity.rs` fails when it and `Router::endpoints()` disagree), `tools/parity-report.py` the counts. `src/gui/descriptor.rs` makes every endpoint refuse an unknown argument field from the inventory. Differential harness: `tests/oracle/diff.mjs` with cases in `tests/oracle/cases/` (TS host needs `?token=` exchanged for a cookie; per-endpoint path `/api/<method>`).
- [x] slice 1, session reads: `session/state`, `session/events`, `session/wait`, `session/observeState`, `session/observeEvent`, `session/observeProjections` (`src/gui/session_read.rs`, `tests/session_read.rs`, 27 tests). Live differential vs TS: 25 cases, 21 identical. The 4 that differ share one cause: a new TS session logs three setup rows (`permission/preset`, `sandbox/mode`, `approval/policy`, seq 0 to 2), so its `cursor` is 2 and its blank page has 3 records; a blank RS session has an empty log (`cursor` -1), because the permission plugin does not pin a session at creation (see the permission note below). Sub-agent addresses: accepted when the child's header names the parent; `continuable` is refused because RS sub-agents are one-shot. `pendingInput` reports `approval` and `question` only. A `session` address for a sub-agent child is accepted (TS: `session/agent-busy`).

- [x] slice 3, credentials write: `credentials/set`, `credentials/unset`, and a real `credentials/describe` (`src/gui/credentials.rs`, `tests/credentials.rs`, 19 tests; `serde_norway` added). One shared document with TS: `$DSH_HOME/.credentials.yaml` (`gui.credentialsFile` overrides), `version: 1` + `refs`, owner-only mode, cross-process `.lock` with dead-holder takeover, atomic rename, other sections (`records`, foreign keys) kept. File outranks the process environment. Live differential vs TS: 50 cases, 0 differ (validator issue lists, 64-ref cap, `__proto__`, spaces-only values, all wrong-type refusals). Divergences: `.env` fallback layers are not read; a write re-renders the document, so comments and hand layout inside `refs:` are not kept (TS patches the text in place); the document is re-read per call instead of watched. A document that cannot be trusted fails `describe` as `gateway/internal` and `set`/`unset` as `credential/rejected`, and is never rewritten.

- [x] boundary validation for every endpoint: `src/gui/descriptor.rs` now refuses an unknown or missing field (optional flag taken from `acceptsUndefined`) and a field whose JSON does not satisfy the descriptor's schema (`type`, `const`, `enum`, `oneOf`, `required`, `properties`, `items`), as `gateway/input-invalid` naming the parameter. Live differential vs TS (`tests/oracle/cases/boundary.mjs`): 545 cases over every served unary endpoint (valid, six wrong JSON types per field, extra field, each missing field), 0 differ. Found and fixed: `directoryPicker/list` without a path now lists the home directory (it used to refuse). Several older tests had sent shapes TS refuses (`session/prompt` without `requestId` and `mode`, `session/page` addresses without `kind`, `session/attachment` with a bare id); they now send what the page sends. Image blocks in a queue edit are a boundary failure, as in TS.

- [x] slice 4, sub-agents: `subagents/prompt`, `subagents/interruptByParent` (`src/gui/subagents.rs`, `src/gui/time_zone.rs`, `tests/subagents_gui.rs`, 10 tests). A message reaches a LIVE child of the named parent as a queued turn or a steer; images are admitted first (model gate, then store) and refused as `subagent/attachment-invalid`. Failures in TS order: bad ids (`gateway/bad-request`, one issue per empty id), `subagent/invalid-time-zone`, `subagent/parent-unavailable`, then `subagent/unauthorized` (another parent's child, or a session that is not a child) or `subagent/not-resumable` (no such child). `interruptByParent` cancels a live child and keeps its unclaimed inbox; an absent child is an accepted no-op. Zone names follow `canonicalClientTimeZone` against a bundled tz database (`jiff`, pinned). Live differential vs TS: 100 cases (76 zone spellings), 0 differ. **Divergences:** a child lives only while its parent's `subagent` tool call runs, so a stored child cannot be resumed (TS revives it); the zone is validated but, as for `session/prompt`, not recorded on the message.

- [x] slice 9, account (signed-out host): `account/getState`, `signOut`, `cancelSignIn`, `getProfile`, `getBalance`, `getUnnotifiedBonuses`, `ackBonusNotified`, `hasRunningAccountTasks`, and the `account/watchExpiry` stream (`src/gui/account.rs`, `tests/account.rs`, 8 tests). The host never signs in, so it reports the state TS reports before a sign-in. `hasRunningAccountTasks` is real: true while an agent runs a turn on the `deepseek-account` provider. Live differential vs TS: 52 cases, 51 identical; the one difference is `getState` carrying a cancelled attempt that an earlier manual `startSignIn` probe left on the TS oracle. **`account/startSignIn` is NOT served** (still in `docs/parity-missing.txt`): it runs the Platform OAuth browser flow against platform.deepseek.com, which needs the account platform credential provider; building it is a separate slice.

- [x] slice 2, schedule: `schedule/list`, `catalog`, `history`, `delete`, `deleteInactive`, `update`, the `schedule_create/list/update/delete` tools, and the delivery driver (`src/schedule/`, `src/gui/schedule.rs`, `src/plugins/schedule.rs`). One store, `$DSH_HOME/storages/schedule.json`, in the TS `storage-json` layout (TS's real parser and task schema read a file RS wrote, with all six kinds). Ground truth is the TS code itself, replayed: 467 cron expressions, 2,475 creations and 21,321 occurrence resolutions across DST gaps/overlaps and odd offsets (`tests/fixtures/schedule_domain_ts.json`), 1,314 updates and 215 stored-record decodes, 16 delivery-history scenarios, 534 tool-argument validations, and a 63-step tool battery run through the real TS tool bodies. Live differential vs the TS host: 67 cases, 0 differ. Also added: a gateway that now drops undeclared properties inside typed parameters (as TS's decoder does) and an exact port of TS's tool-argument validator (`src/tools/args_schema.rs`). Zone names are stored in ICU's spelling (`Asia/Kolkata` is `Asia/Calcutta`) via a generated alias table. **Divergences:** (1) a due reminder for a session that is neither live nor stored is retried every 30 s and stays active, where TS also retries; (2) the `schedule` plugin's tools need the `gui` plugin to be loaded first, because the `gui` plugin owns the service; (3) when an object carries several undeclared keys, the tool-argument violations list them sorted, not in the order the model wrote them (a parsed JSON object has no key order here); archive admission matches TS (armed reminders refuse `workspace/archiveSession` as `workspace/session-active` with a `schedule` activity entry; `stopActivity` deletes them; 7 live cases identical). RS wires this directly in the router rather than through TS's `workspace/session-activity` event, so a third-party activity provider cannot join it.

- [x] slice 3, terminal: the six unary `terminal/*` endpoints, the two streams (`terminal/attach`, `terminal/retain`), the pty process layer, and an xterm-exact screen serialiser (`src/terminal/`, `src/gui/terminal.rs`, `src/gui/terminal_stream.rs`, `src/gui/uplink.rs`). The screen is the hard part: the TypeScript host sends `SerializeAddon.serialize()`, so this build vendors `vt100` (in `vendor/vt100`, with the extra SGR attributes xterm tracks and xterm's own Unicode v6 width table) and ports the serialiser. Checked against xterm itself: 112 hand-written cases plus 400 random streams (text, colours, cursor moves, erases, scrolls, line and cell edits, mode switches, the alternate screen), all byte-identical. The pty is a real pseudo-terminal (the child gets the slave as its controlling terminal in its own session), covered by 8 tests. The unary half was written by GPT 6.1 Sol against `docs/specs/terminal-controller.md`; the stream half (the first bidirectional stream in this build) adds an uplink inbox to the mux with the TypeScript gateway's byte accounting and its two violations (`gateway/protocol` for an item after the end, `gateway/uplink-overflow` for an overfull window). Live differential vs the TS host: 46 unary cases and 7 stream frame sequences, 0 differ. **Divergences:** (1) a client's uplink items are counted with the whole frame's bytes, as TS does, but RS counts the frame text it parsed, so a frame the host rewrites would count differently; (2) the RS mux admits one socket per connection, as TS does, but a *second* socket on the same port is refused with 409, so a client with two windows needs one socket; (3) the RS terminal output buffer is a broadcast channel of 256 frames: a client that falls further behind is failed with `Terminal output consumer exceeded its buffer`, where TS fails the follower with a slightly different wording and no frame count.

- [x] `directoryPicker/pick`: the native directory chooser (`src/gui/picker.rs`). The TypeScript host opens the operating system's own chooser — an AppKit panel on macOS, `zenity` or `kdialog` on Linux — and answers the chosen path or nothing when the user dismisses it. This build runs the same AppKit script and the same two Linux commands, with the same answers and the same failures (`directory picker failed: …`, and `directory-picker/unavailable` when the deployment composes only the browse capability, as the Web host does). A host without the native capability says so rather than opening the wrong thing. 10 tests (6 unit, 4 through the gateway) with a scripted chooser, so no test opens a dialog. Not in the live differential: the TypeScript host would open a real chooser and block.

- [x] the terminal slice, reviewed by GPT 6.1 Sol and Grok 4.7 (both read-only, against the two-commit diff). Sol reported 10 findings, Grok 7; the defects that survived verification are fixed: a write that held the frame lock across the pty write (the terminal could stall), the process range a close ends (the shell's background jobs survived it: this host's `ps` names the session column `sess`, not `sid`), the idle reclamation that killed the process but kept the identity and the slot, a follower push that waited for the attachment reading it (one slow client stalled the others), the retain stream that ended as a cancellation where the gateway sends `end`, the refusal precedence (a terminal that ended refuses before it asks who holds the input), the rename that measured the trimmed title instead of the one as it arrived, the picker that was never wired into the plugin, the uplink violation that ended a stream without its error frame, the output that could vanish between the snapshot and the subscription (the follower now holds the snapshot's screen and its frames under one lock), the follower's byte bound that was a channel capacity instead of the configured limit, the UTF-8 that was split across pty reads, and the command that was not serialized with the output frames. Every attachment now has one ordered, byte-accounted queue, as the TypeScript host does. 890 tests pass; the live differential is unchanged at 46 unary cases and 7 stream sequences.

- [x] the terminal slice, second review round (GPT 6.1 Sol and Grok 4.7 again, on the fixes). Sol found six more, Grok five: the session scan that never matched (this host's `ps` has no session column, so the terminal's jobs were never ended — the process tree is walked now, as the TypeScript host does on macOS, and a job-control shell's background job dies with it, with a test), an activity scan that held the pty while it ran (a close could deadlock behind it), a natural process exit that ended the attachments (the TypeScript host keeps them, so a later input is answered `not-running`), the snapshot that went through the bounded queue (a reconnect could be refused for a screen the queue would not hold), the unbounded pty output queue (now bounded, so a noisy process cannot grow the host), the idle rule that ignored the terminal's own jobs and the input that ended it, an uplink item with an extra key (the TypeScript codec strips it; this one refused it), a wakeup lost between a producer and a consumer (`notify_waiters` stores no permit), the chooser's deadline (it waits for the operator now), and the activity poll that was raised to a second. The differential harness's takeover probe was dead — the stream it wrote to had already failed on a sequence gap — and now runs on a live attachment. 892 tests pass; the live differential is 46 unary cases and 9 stream sequences, all identical.

- [x] the terminal slice, third review round (GPT 6.1 Sol again): seven more findings, the important ones fixed — a write that held the frame lock while the pty write blocked could deadlock the terminal's own output reading (the acknowledgement is now queued before the write, so the echo cannot overtake it and the lock is not held across a blocking write), the retention monitor that reclaimed a terminal whose hold or input had arrived while its inspection ran (the observation is checked again after the scan, and a hold invalidates it), a closed identity that was still attachable while its process ended (the close now fences the registration under the frame lock, and the attachment ends with the terminal), and an input that was starved behind a busy output queue (the selection is no longer biased towards the frames). The differential harness also compared only the frame kinds, so an implementation with the right kinds and the wrong bytes passed it: it now compares the snapshot's screen and the output's text as well. Not fixed, and documented: the identity fence TypeScript puts on every signal (a recycled pid can be signalled after a long-lived terminal is closed) and the shell prompt evidence TypeScript reads from a shell hook, without which a shell running a builtin looks idle to the retention monitor.

- [x] slice 4, `officeToPdf`: the document preview's conversion (`src/gui/office.rs`). The TypeScript host converts with `@deepseek-ai/libreoffice-kit`; this build runs the same kit's command-line interface, so the converter, the failure categories (`invalid-document`, `unsupported-format`, `input-too-large`, `output-too-large`, `timeout`, `unavailable`, `failed`) and the answers are the same. The conversion is configured (`gui.officeConverter.command`); without one the host answers `unavailable` rather than pretending. The gateway gained what the TypeScript gateway has and this build did not: a result with binary parts is answered as multipart — the `bytes-0` part, then the `metadata` part that names it — so the document's bytes travel as bytes. 8 live cases identical to the TypeScript host, including a real `.docx` converted by both. The service's own generation is a fresh UUID per host, and the file's version is this host's own, as in TypeScript.

- [x] the parity wave: 78 endpoints in fourteen groups, each written by a pinned GPT 6.1 Sol run through the Harness2 rollout seam (one model, as the review policy now says) against the TypeScript package that owns the group, and each reviewed against the live TypeScript host where the host mounts the group. The groups: `toolResults/watch` and `userQuestions/attachWait` (the streams over state this host already keeps); `workspaceFiles/changes` (a polling watcher — the TypeScript host watches through the filesystem provider, this one samples the file's version, so a change reverted between two samples is not seen); `productAnalytics/{enabled,report,watchPolicy}` (the policy is a configuration value, and the intake accepts and drops, because this host has no telemetry backend); `actionCards/act` and `pluginRecommendations/describe` (both answer the TypeScript host's successful refusal `{status: "refused", reason: "unknown"}`, because this host logs no cards); `cotTranslation/{limits,translate}` (the limits and the preferences are served; a translation is refused with the TypeScript host's code, because this host has no translation provider); `claudeCodeMods/{pressBand,watchBand}` (the empty drawing, and the TypeScript host's own answer for an ignored press, which is a snapshot and not a failure); the nine `cuaAccess/*` (the computer-use access: this host has no native client, so the permission state is the state a host without one has, and the refusals carry the TypeScript host's codes); the twelve `liveshareShares/*` (this host shares nothing: the list is empty, the identity is unverified, and the rest answer the refusal for a share it does not know); the eighteen `forgePanel/*` (this host has no forge and no git: the reads are empty, the mutations are refused, and the two streams answer the baseline and wait); the fifteen `pluginManager/*` and four `pluginMarket/*` (the manager is an npm package manager and this host has no npm ecosystem, so its answers come from this host's own state — the profile's bundle names, the plugin roster — and every mutation answers the TypeScript host's operation-result shape with `application: "failed"` and a diagnostic that says so; the market's index list lives in the settings store, so a commit is durable before the call answers); the six `speech/*` (the catalog and the preferences are served; a preparation, a cancellation and a transcription are refused with the TypeScript host's code and message, because this host has no provider); and `account/startSignIn` (the Platform browser sign-in: the origin is checked, the PKCE pair is generated, the platform is asked for an authorize url, the browser is opened, the loopback callback is served and its `state` is checked before the exchange, and the grant is written to the credential document's records section — the store gained an atomic provider-record API for it). The settings store now serves three namespaces (`ui-settings-general`, `speech`, `plugin-market`) instead of one, with a revision per namespace and a document that holds one object per namespace.

- [x] the terminal slice, fourth review round (GPT 6.1 Sol, run through the Harness2 rollout seam with the model pinned): eight findings, none of them closed yet. They are the next work for this slice: (1) the process cleanup signals bare pids, with no identity fence, so a pid that was reused can be signalled — the TypeScript host keeps each process's start identity and rechecks it before every signal; (2) a write that blocks on a child that stopped reading holds the attachment's select branch, so neither the cancellation nor the output is delivered while it blocks, and the resize and the activity scan take the same pty mutex the blocked write holds; (3) the reclamation mistakes a shell running a builtin for an idle one — the TypeScript host reads the shell's own prompt evidence, which this build does not have; (4) the cleanup captures the descendants once, so a job spawned while the first round of signals ran escapes it — the TypeScript host rescans and checks the survivors; (5) closing can end an attachment before its queued frames are drained; (6) the write's acknowledgement is queued before the write, so a failed write is acknowledged as a success, and an exit during the write fails the stream instead of answering `not-running`; (7) a resize publishes the screen and the dimensions in three steps without the frame lock, so an attachment's baseline can catch a resized screen with the old dimensions; (8) the reclamation test proves neither the process's death nor the slot's release, because the terminal is marked reclaimed before its cleanup finishes.

## Phase 1 session actions
- [x] `session/rename` — TS-compatible `session/title` row, title projection, live sidebar update
- [x] `session/fork` — fork at a turn boundary, child takes the default model (TS parity)
- [x] `session/selectModel` — TS-compatible `model/selection` event with its own seq, v4 row, per-session routes
- [x] `session/updateQueue` — edit / remove / steer a pending queue item over a durable inbox
- [x] `session/initializeDefaultModel` — answers `{ok:true}` and does nothing: the default model comes from the profile; the account fallback needs the account layer, which this host lacks
- [x] `pluginInventory/list` — the plugins this process runs, read per call from the bundle's `PluginRoster` (no display `meta`, `agentPresets` or `managementAvailable`)
- [x] `pluginRegistryProbe/fastest` — races npm and npmmirror, shared in-flight probe, winner remembered, offline answers `null` and is retried; failures logged
- [x] `settings/openSettingsDocument` — `gateway/service-unavailable`, matching `settings/describe` `hasDocument:false`
- [x] `messageFeedback/{list,put,delete}`, `sessionFeedback/record` — log-only `feedback/*` events (v4 rows pass the TS validator), version compare-and-set, flushed before the reply; loaded sessions only
- [x] `ask_user_question` tool (`ask-user` plugin, blocking mode) and `userQuestions/answer` — question goes to the browser as a `user-questions/request` frame on `$events`; an answer must name each question once or it is refused and the question stays open; sub-agents are refused (`DELEGATED_CALLER`); no browser attached fails at once (`NO_PROVIDER`). Not ported: the timed mode (a question that outlives its wait), so `userQuestions/answer` answers `false` for a known session, and `userQuestions/attachWait`
- [x] `llm/discoverModels` — probes `GET {baseURL}/models` (OpenAI protocols, bearer) or `{root}/v1/models` (Anthropic, `x-api-key`); 4 MiB cap on bytes read; no redirects; only namespace `llm-pi-ai`; 24 gateway cases identical to live TS. Not ported: TS answers pi-ai catalog providers with no network call (this build has no catalog, so a provider without a `baseURL` is refused). Timeout is `gui.llmDiscoveryTimeoutMs`
- [x] `schedule/list`, `schedule/catalog` — empty (no scheduler) but they check their arguments and the session like TS (`schedule/list` takes `request.sessionId`)
- [x] slash commands — `commands/list` and `commands/execute` (`src/gui/commands.rs`, `tests/commands.rs`). Commands offered: `compact` (only when the compaction plugin is loaded), `feedback`, `goal`, with the TS descriptors in TS order. A line that is not a known command answers `{ok:true}` with no value and logs nothing (the client shows "unknown or malformed command"). A known command logs `command/run` (`args` omitted for `/feedback`), its effects, then `command/done`; both are v4 rows the TS validator accepts. `/goal` shares the `goals/*` code (`goals::apply`), with the TS grammar and texts, and attachments go through the same image/file admission as prompts, ahead of the grammar check, then follow the goal as a reference message. `/compact` runs `compact_if_needed(force)` and reports `sourceEventSeq` = the summary event. 62 of 63 live TS-parity cases are identical (`cmd_parity.mjs`, includes the log); the 63rd is the command list. Real Chrome: slash menu lists the three commands, the cards render for `/goal`, `/goal pause`, `/feedback`, `/compact`, and `/nope` goes to the model as plain text.
- [x] `job/kill` — this build keeps no background-job registry, so every job is unknown: `job/not-found` with the TS message (`Error: unknown job <id>`); malformed requests fail at the gateway first. 12 cases identical to live TS (key order aside). Needs a job registry before it can stop anything
- [x] `session/create` with `agentPreset`
- [x] file uploads — `fileUploads/upload` and the raw `POST /api/session/uploadFileBinary` route the browser uses. Bytes go to a content-addressed store (`gui.attachmentsDir`, an absolute path; unset means uploads answer `gateway/service-unavailable`, and the store is never placed in `~/.dsh`); the reply is a per-session receipt. A prompt cites it as `{type:"file", receiptId}`; the session log keeps only the `{attachmentId,name,bytes}` reference (a TS v4 `file` block, accepted by the TS validator), and the request rewrites it to the TS handle text naming the read-only copy. A receipt is spent when its prompt reaches the log, returned when the queued prompt is removed, and dropped when the session is disposed. 32 gateway cases identical to live TS (`up_parity.mjs`), and a real-Chrome run: attach, send, reload, oversize, remove.
- [ ] image attachments — `image` prompt blocks and `session/attachment` are still refused (`gateway/input-invalid`, field `content[i].type`). Needs an image decoder (media-type check, pixel and dimension limits) and a per-model image-capability flag; neither exists in this build
- [x] open-workspace-path actions — `session/canOpenWorkspacePath`, `session/workspacePathApplications`, `session/openWorkspacePath`

## The 12 points
- [x] 3 TS session-log v4 read/write — writer plus a TS-compatible reader; rows validated by the real TS reader
- [x] 4 Boot endpoint set — host, module graph, `/plugins`, index injection, presets, model catalog, settings, credentials, inventory
- [x] 5 GUI approvals surface — prompt over `$events`, allow once / reject; "allow for run" is not offered
- [x] 6 Goal plugin — independent verifier plus bounded continuation, both default off, mutation-checked
- [~] 7 CI endpoint diff against upstream — the parity generator (`tools/gui-endpoints.mjs`) exists; the live probe is ad-hoc
- [~] 8 Generated parity table — endpoint coverage is recorded above; the table itself is not generated into CI
- [~] 9 Search/fork/rename/settings/credentials/files/terminals/plugin manager — rename, fork,
  settings, credentials and the workspace files done; the search covers the stored sessions too;
  terminals and the plugin manager not started
- [ ] 10 Image/file prompt content — attachments
- [x] 11 Cold session reload — the v4 store is wired, every turn checkpoints the log, and a restart resumes the stored sessions with their history and route
- [x] 12 Health/readiness — `GET /api/health` behind the fence and token, mutation-checked
- [ ] 13 Token-level assistant streaming — replies still arrive as committed events
- [x] 14 `goals/*` (7 endpoints) — get, create, edit, pause, resume, complete, clear; TS-compatible `goal/change`
  rows (the real TS reader accepts a Rust-written log), the `goal` projection and its live push; a 26-step call
  sequence gives the same replies from the live TS host and the Rust host. No round driver in this build, so
  `roundsStarted` moves only through the log
- [x] 15 Desktop path endpoints (3) — open a verified workspace path with the default application, a chosen one,
  or the file manager, and list the registered applications (macOS `open`/`osascript`, graphical Linux
  `xdg-open`). 22 request cases and 4 real application listings give the same replies as the live TS host
  (key order aside). Every native command has a deadline (`gui.desktopCommandTimeoutMs`); tests use a
  scripted runner, so none start an application

## Next up (in order)
1. ~~`session/create` with `agentPreset`~~ — done: the roster is a validated profile field, an
   unknown preset is refused with `agent-preset/not-found` and the roster it offers, a blank
   session may change the preset it runs (the change is logged as `agent-preset/selected`, the
   TypeScript event), a session that started a turn is not reused, the preset is recorded on the
   session header, echoed in the response and the projection, written into the stored log header
   and kept by a resumed session, and a preset's own instructions reach the model.
2. ~~`session/search` beyond live sessions; `session/list` cursor~~ — done: the list serves the
   stored sessions no live agent covers as cold rows (`agentAvailable: false`, `formatStatus:
   current` or `migration-required`) and orders every row by activity, newest first; the search
   covers the live and the stored sessions, and an empty or overlong query is refused with
   `gateway/bad-request`.
3. token-level streaming, terminal (6) and jobs, account, plugin manager/market, the
   dynamic runner, the credential store's write side, the feedback log and the scheduler

## The cold-reload slice (this milestone)
- The session log reaches the disk: the logged request prefix is durable before the model is called
  and a recorded tool call is durable before its body runs, and the turn ends with a checkpoint of
  the response. A failed checkpoint stops the turn: the model and the tool body do not run, the way
  the TypeScript checkpoint policy fails closed.
- A turn is only reported idle once its log is on disk, so a caller that waits for idle knows the
  turn is durable.
- A restart finds the stored sessions again: the GUI adopts them at startup, the log is replayed
  into the session, and the agent keeps the route the session last used (the `model/selection`
  row, not the deployment default).
- The stored log is the session's own history, not an inherited seed: the v4 store reconciles the
  rows it already holds with the events the session resumes from, and the log on disk stays
  authoritative for its header. A log that cannot be reconciled is reported, never repaired.
- The working directory comes from the stored header, so a resumed session keeps the workspace it
  ran in. The Rust-native `jsonl` backend keeps no header and loses it.
- A resumed session is the session's own history, not an inherited seed: it keeps the stored created
  time and lineage, and the log on disk stays authoritative for its header.
- A stored log that cannot be read is reported with its session id, and the session is not resumed.

## The queue slice (previous milestone)
- The pending queue is a durable session event: every mutation appends `agent/inbox/spliced`
  (`target`, `start`, `removedCount`, `inserted`, `outcome`) and the `inbox` projection is folded from
  those rows, the way the TypeScript inbox projection folds them.
- `session/updateQueue` speaks the TS contract: `{sessionId, itemId, action}` with
  `edit` / `remove` / `steer`, `{accepted:true}` on success, `session/queue-item-not-found`,
  `session/steer-unavailable`, `session/attachment-invalid` (`QUEUE_EDIT_NON_TEXT`) and
  `gateway/bad-request` for a whitespace-only edit. The edit content is validated before the agent
  is resolved, and the eligibility check and the mutation happen together, so an item the driver
  claimed in between is reported instead of silently ignored.
- The client prompt identity rides along: `session/prompt` accepts `requestId`, stores it as
  `source.rpcId` on the queued row, and a retry with the same identity is accepted without queueing
  a second copy (the check and the queueing are one operation).
- The claim order matches TS: the step input first, then the queued turn message.
- One mutation lock serialises every inbox mutation, so the recorded splice and the live pending
  lists cannot disagree when two mutations interleave.

Stops and asks before: any cutover, any push, anything touching live `~/.dsh` or ports 3080/20128.

## Endpoint coverage (live probe, Rust GUI on 3081, 2026-10-25)
- Unary: 36 of 131 answer; 95 return `gateway/lookup-not-found`.
- Served: `agentPresets/list`, `commands/list`, `credentials/describe`, `dynamicCordisRunner/inventory`, `dynamicCordisRunner/syncInspectManifest`, `llm/listConfigurableProviders`, `llm/listProviders`, `permissionPresets/catalog`, `schedule/catalog`, `schedule/list`, `session/cancel`, `session/create`, `session/fork`, `session/list`, `session/modelCatalog`, `session/page`, `session/projections`, `session/prompt`, `session/rename`, `session/search`, `session/selectModel`, `settings/describe`, `settings/mutate`, `settings/replace`, `settings/update`, `skills/list`, `workspace/archiveSession`, `workspace/create`, `workspace/delete`, `workspace/initializeDefault`, `workspace/insertBefore`, `workspace/insertSessionBefore`, `workspace/pinSession`, `workspace/rename`, `workspace/unarchiveSession`, `workspace/unpinSession`.
- Unserved groups: pluginManager (12), dynamicCordisRunner (10), account (9), goals (7), session (6), terminal (6), speech (5), workspaceFiles (4), directoryPicker (3), messageFeedback (3), pluginMarket (3), schedule (3), agentPresets (2), cotTranslation (2), credentials (2), officeToPdf (2), productAnalytics (2), subagents (2), claudeCodeMods (1), commands (1), fileReferences (1), fileUploads (1), job (1), llm (1), sessionFeedback (1), sessionReferenceResolver (1), userQuestions (1).
- Streams served: `$events`, `session/follow`, `workspace/follow`, `remoteAccess/follow`, `session/control`, `account/watch`.
- Streams not served: `account/watchExpiry`, `claudeCodeMods/watchBand`, `job/follow`, `job/list`, `productAnalytics/watchPolicy`, `speech/follow`, `terminal/attach`, `terminal/retain`, `userQuestions/attachWait`, `workspaceFiles/changes`.

## What works (verified in a real Chrome against `dsh serve`, port 3081)
- The shipped TS web client boots from dsh-rs alone: module graph, `/plugins`, index injection.
- The shell answers `/` with the injected boot; a profile whose `assets` is not the directory
  holding `index.html` is refused when it loads, not answered with a 404 on every page.
- Preview notice dismisses and the acknowledgement persists (settings store).
- Model picker, preset and permissions render (shared `projection_values`).
- Prompt -> sonnet-5-5 reply renders; a second turn sees the first.
- Stop control appears while a turn runs and cancels it (`api-session/status` on `$events`).
- Bash tool cancel/timeout now kills the whole process group (test fails without the fix).
- New session, two sessions in the sidebar, reload restores history, search finds a session.
- Tool approval: prompt appears, nothing runs before the answer, Allow once runs it, Reject does not.
- Model selection: the picker lists two models, the choice changes the label live, and the next request uses the selected route (the log header shows it).
- Fork: the API returns `session-2` and `session-3`; both children appear after a refresh with the inherited turn and the default model.
- Beam Free and MiniMax M3.1 Flash each completed a GUI turn; selecting MiniMax updated the picker live after a nonempty event baseline and `lastUsed` advanced on its next request.
- The queue: a second prompt sent while a turn runs appears as a queued row in the client QueueDock
  (no stuck "sending" echo), Cmd/Ctrl+Enter steers it into the running turn, and edit and remove
  answer `{accepted:true}` and show up in the `inbox` projection. No console errors, no failed calls.
- Cold reload: a turn leaves a v4 log on disk, and after the server restarts the session is back in
  the sidebar with its prompt, its answer, its route and its working directory. A prompt sent to the
  resumed session runs there and appends to the same log (dense seqs, no duplicate rows).

## Known limits
- `session/search` is live-only, first match per session. `session/cancel` also drops queued follow-ups.
- Desktop: WSL and Windows report that opening is unavailable. The path check and the opener resolve the pathname separately, as in the TS host, so a process that can rewrite the directories on the path can redirect an open. `$BROWSER` is read as one executable name, as in TS. Linux has no application query: its list is empty.
- Prompts carry text and uploaded files; no images. `session/list` ignores the cursor. Wire pages encode the whole log before windowing.
- Projections are partial. `modelSelection` follows the agent's selected model and the last request header, and `title` the latest rename; the rest are fixed. `session/control` pushes those two keys when `model/selection`, `request/header`, or `session/title` is appended.
- `session/selectModel` offers only the default route plus the profile's `gui.models` list (an adapter cannot list its models) and refuses `reasoningEffort`. Selection appends a TS-compatible `model/selection` event and v4 row with a new seq; the selection survives a restart because the session log is checkpointed and resumed.
- `session/fork` works on live sessions only; the child takes the default model, not the source's. The child is returned by the API immediately but appears in the sidebar only after a refresh because live list updates are not implemented.
- No token-level assistant streaming (replies arrive as committed events). Rust does not mount the TS `decision-consumers` JEV failover: a Sonnet 429 closed the turn as Failed in the Rust browser test rather than switching providers. The separate TS Web/headless failover profile changes do not add this capability to Rust.
- The queue events live in the session log, and the log is checkpointed when a turn ends, so a
  pending row survives a restart. A resumed queue waits for a prompt: an inherited queue does not
  start a turn on its own (the TypeScript inbox behaves the same).
- `session/prompt` ignores `clientTimeZone`: the TypeScript controller refuses a zone that is not
  UTC or a real IANA name with `session/invalid-time-zone` and carries the zone on the message
  source. The Rust build has no time-zone database, so it accepts the field and drops it. The field
  is not model-visible.
- `schedule/list` and `schedule/catalog` are honest empty lists.
- Permission presets (`/permission`, plugin `permission`, `src/permission.rs`, `src/plugins/permission.rs`, `tests/permission.rs`): the `permission/preset`, `sandbox/mode` and `approval/policy` rows are logged in the TS shapes and folded by the TS rules (the last selected preset names the knobs while it still matches, else the first matching preset, else `custom`). The command texts, the catalog (`defaultOptions`, `defaultPreset`) and the live `permissions` projection (also pushed over the mux) match TS. Default presets: `profile` (the pre-existing behaviour: file access unconfined, the approvals plugin decides), `workspace-write` and `danger-full-access`, all configurable (`presets`, `sandbox`, `approval`, `fileTools`, `shellTools`; a bad table stops the profile). **Divergence: there is no OS sandbox**, so enforcement is a check before each tool runs, and it fails closed. `read-only` refuses every tool in `fileTools` (default `write_file`, `edit_file`) and `shellTools` (default `bash`); a tool in neither list (an MCP tool, say) is not covered. `workspace-write` refuses file changes whose `path` is missing, unresolvable or outside the working directory and the temp directories; paths are resolved component by component through links, so a dangling link or `link/..` cannot leave the workspace. It also refuses shell tools when approval is `never` or the approvals plugin is not composed, since nothing could approve or confine them. The workspace is the session's working directory, else the process's, and never `/`. A knob value this build does not know reads as `read-only` / `ask`. The check runs before the tool opens the file, so a path swapped in between is not caught. Approval `never` suppresses every approvals prompt (the approvals `deny` list still applies; TS instead rejects escalation requests). A switch writes its knob rows tightening-first, and a delegated sub-agent copies its parent's knob rows into its own log at spawn. Without the plugin the catalog is the single `auto` value as before. A new session starts on the configured defaults and is not pinned by a `session/created` row, so changing the configured defaults later changes what an unmarked old session reports.
- Plan mode (`/plan`, plugin `plan`, `src/plugins/plan.rs`, `tests/plan.rs`): `plan/mode` is logged; `/plan [off|message]` has the TS texts, attachments and steering message; a choice made during an open turn is applied at the next step by the new `agent/step-start` waterfall (`src/core/loop_driver.rs`; it runs before every step and lets a plugin add log-recorded messages and system text), where it is logged and, if the last request header described the other mode, narrated with the TS notice (`plan-mode` source, notice form). The `plan` projection follows the TS fold including the `command/run`/`command/done` pending state. The guidance (`plan.section`, default the TS text) joins the system prompt on every step while plan mode is active. `exit_plan_mode` presents the plan through the `userQuestions` channel (needs `ask-user`) with the TS review question; approval exits plan mode silently at the next step. A queued choice and an approved review are derived from the log (`command/done` with a different wanted mode; an `exit_plan_mode` result without an error while plan mode is on), so a restart does not lose them; the in-memory pending map only adds the narration flag. A step-start listener that fails, or an entry it returns that does not parse, is reported on stderr and skipped. **Divergences:** a dismissed review comes back as an error result rather than concluding the turn; plan guidance is not part of `prompt_service.assemble()` (it is appended per step); `exit_plan_mode` is registered only when the plugin is loaded. Enable both with `"permission": {}` and `"plan": {}` in a profile's `plugins`.
- Slash commands: `/export` is offered when a session store is composed; it validates the line (no argument) and the browser then downloads `GET /api/session.export?sessionId=…[&includeDescendants=true]`. The archive has the TS layout (`session.v4.jsonl`, `subagents/<id>/session.v4.jsonl`, `media/<id>.<ext>`, `files/<xx>/<digest>/<name>`, no manifest), reads each log from the store after flushing live sessions, and takes `gui.exportCompressionLevel` (0-9, default 6; 0 stores). It is built in memory and sent whole, so an unreadable descendant or attachment is a `500` before any byte is sent, where TS streams and can cut a download short. Descendants are found from the live and stored session headers' `parentSession`. The store must keep logs in the TS format (`store_format` v4); the JSONL backend answers `500` "does not keep logs in the session-log format". `/compact` is offered only when `plugins.compaction` is loaded. The anonymous user id `/feedback` prints is read from, and if missing written to, the TS host's own file, `$DSH_HOME/.anonymous-user-id` (else `~/.dsh/.anonymous-user-id`), so both hosts report one id; `gui.anonymousUserFile` (absolute path) overrides it. An id already in the file is never replaced, and a new one is published whole (written and synced to a private file, then hard-linked into place). With no home directory known the id lasts one process. The "compaction in progress" guard covers `/compact` only; automatic and tool-driven compaction are not serialised against it (TS shares one in-process guard). A goal command's read and write are two steps: `goals::apply` re-checks under its lock, so a concurrent `goals/*` call makes one of them fail instead of overwriting. `/compact` error texts for causes this build reports as one string (a summary that is not shorter, no summary model) read "Compaction failed: <reason>" instead of the fixed TS sentences.
- Uploads: a file is read whole into memory (the cap is `gui.maxUploadBytes`, default 32 MiB; the request body is cut at that size as base64 plus 64 KiB, answering 413), not streamed to disk as TS does. Receipts live in memory, so a restart drops unspent ones. Without `gui.attachmentsDir` the model is never shown a file handle, because no file can be staged. Compaction renders a file as `attached file "<name>"`. The store root is trusted, as in TS: a process that can swap directories or symlinks inside `gui.attachmentsDir` can redirect where a file is read from. A session's receipts are not capped in number (TS has no cap either); the per-file cap is the only bound.
- One `$events` socket at a time; the browser cannot grant "allow for run". Long-lived launch token.
- Default profile has no `approvals` plugin: bash runs unguarded unless the profile enables it.
- v4 log: Rust compaction is an ignorable row, 35 TS-only row types skipped on read; nothing was ever written into `~/.dsh`.

## Reviews
- JEV: baseline then rescore on the review fixes (correctness 6.7 -> 8.1, no regressions, observability 6.2 weakest).
- Astra (`cx/gpt-6-astra`): 10 findings; 1 false positive, 4 fixed (settings document validation and persisted revision, cwd-aware blank reuse, workspace lag resync), the rest recorded as limits above.
- Not reviewed: the `api-session/status` stream and the process-group kill landed after both reviews.
- Phase 1b (fork, selectModel, failed-turn close, control frames): Astra found 5 real issues, all fixed (control baseline race, stale model notification, reasoningEffort ignored, unsafe-integer `atSeq`, `session/not-found` code). JEV baseline 6.7-8.4 per metric; weakest observability 6.7 and documentation 7.0.
- Phase 1c (the queue): GPT 6 Sol (`cx/gpt-6-sol-high`) reviewed the queue diff and found 6 issues.
  Fixed: non-atomic inbox mutations (one mutation lock now covers the index, the recorded splice and
  the live list), a prompt identity that two concurrent retries could both pass (the check and the
  queueing are one operation), a vanished item that was silently accepted (the agent applies the
  mutation and reports what it did), a fold that repaired invalid splices instead of refusing them
  (the fold is shared with the agent and reports a broken log), and an inserted message whose role
  was lost on replay (the role round-trips now). The sixth finding — the durable events are not
  replayed into a reopened agent — is the cold-reload gap below; the part that was reachable today
  (a forked session) now restores the rows it inherits. JEV baseline for the slice: 7.0-8.5 per
  metric, every finding low severity.
- A second GPT 6 Sol pass over the fixed slice found 4 more races, all fixed: a steered prompt
  bypassed the prompt identity admission (it now goes through the same one), the pending counter was
  cleared by a claim instead of counting the rows that are still unclaimed, a retry that landed
  between the claim and the log was not recognised (a claimed prompt keeps its identity until it is
  logged), and a message whose identity was already pending was queued again (it is now refused with
  an agent error, because queueing it would write a log the inbox fold rejects).
- A JEV rescore of the last round (test quality 6.9 -> 7.7, performance 6.6 -> 7.9, no
  regressions) and a parity check against the TypeScript prompt path found one more gap: a prompt
  with no non-whitespace text is refused with `gateway/bad-request`, the way the TypeScript
  controller refuses it.
- A JEV rescore of the finished slice scored 6.6-8.4 per metric (weakest: performance 6.6, test
  quality 6.9); the review led to three more fixes: the fold tracked the pending identities while
  folding instead of rebuilding them per event, a regression test holds a turn after the driver
  claimed its prompt and shows that a retry in that window does not open a second turn, and the
  admission now reports whether a prompt was queued, repeated or refused, so the router no longer
  answers "accepted" for a prompt the agent refused. A claimed identity is released once the prompt
  reaches the log, which keeps the remembered set to the claims the log does not cover yet.
- Grok 4.7 was run over the slice twice; both attempts answered with a plan to read the code rather
  than findings, so it is not an approval. Its three candidate concerns were checked by hand: the
  router swallowed the admission result (now fixed), the claimed identities were never pruned (now
  fixed), and the claim could race the admission (the mutation lock covers both, so it cannot).
- Cold reload (checkpoint, resume): JEV baseline 7.3-8.1 per metric, every finding low severity
  (weakest: correctness 7.3 at 0.24 confidence, changeability 7.6, observability 7.5). The one
  improvement it named is in: a resume now reports how many sessions are live again and how many it
  could not read. GPT 6 Sol (`cx/gpt-6-sol-high`) found 5 issues. All fixed: the
  checkpoint ran after the side effects (it now runs before the model request and before the tool
  body, and a failure stops the turn), the end-of-turn flush was detached from the turn (the turn is
  idle only once the log is on disk), the resumed history was passed as an inherited seed (the
  resumed session keeps the stored header and owns its history), a stored log that could not be read
  was skipped in silence (it is reported with its session id), and a session with only a header
  disappeared on restart (it is resumed). Grok 4.7 answered with a plan to read the code instead of
  findings for the third time, so it is not an approval.
- Fixed on the way: a failed model request left the turn open forever (now closed with an `error` reason; the test fails without the fix).
- Grok 4.7 identified that an equal-seq `session/control` model frame is discarded by the TS projection store; `selectModel` now appends `model/selection` with its own seq and the v4 row passes the TS reader. Follow-up Grok attempts returned only an intention to inspect or timed out, so they are not approvals. JEV rescored the added tests from 5.7 to 6.5 for test quality; other scalar dimensions remain provisional.
- Browser-verified: the picker lists two models, the choice changes the label live and the next request used it (log header shows the route), a live rename updates sidebar and header and survives reload. Fork returned `session-2` and `session-3` from the API; both children appeared after a refresh with the inherited turn and default model. After rebuilding the Rust server, Beam Free and MiniMax M3.1 Flash each completed a GUI turn; selecting MiniMax updated the picker live after a nonempty event baseline, and `lastUsed` advanced on its next request. The previous Sonnet route returned 429; no further paid-Go requests were made.

## Never ported
Desktop, LSP, SSH, browser/computer-use, MCP reconnect.

## The list and search slice (this milestone)

- `session/list` serves every visible session: the live ones and the stored ones no live agent
  covers. A cold row names its working directory and its parent, reports `agentAvailable: false`,
  and reports `formatStatus: current` when its log can be read and `migration-required` when it
  cannot, the way the TypeScript host serves a stored record.
- The rows are ordered by activity, newest first, the way the TypeScript host orders them.
- `session/search` covers the live sessions in memory and the stored ones from their log. A session
  without a working directory is not visible, so it is not searched, the way the TypeScript host
  searches only the sessions it can see.
- An empty query and a query over 500 UTF-16 code units are refused with `gateway/bad-request`,
  the way the TypeScript host refuses them.

## The agent preset slice (this milestone)

- A deployment offers a roster of agent presets (`gui.agentPresets`): the profile validates it at
  load (a non-empty roster, unique ids, exactly one default).
- `session/create` accepts `request.agentPreset`, refuses an unknown one with
  `agent-preset/not-found` (details `agentPreset`, `available`) and answers with the session's
  `agentPreset`.
- The preset is recorded on the session header, reported in `session/projections`, written into the
  v4 log header and restored by a resume, so a session that restarts runs the preset it ran with.
- A preset change is recorded the way the TypeScript host records it: `agent-preset/selected` with
  the TypeScript event type and data, written as a v4 row the TypeScript reader accepts, read back
  from the log, and folded into the projection. The header keeps naming the preset the session
  started with; the selection names the one it runs.
- A blank session may change the preset it runs. The blankness check and the record are one
  operation under the session's event lock, so a turn cannot start between them, a session that
  started a turn is never reused, and two concurrent choices record one change. A session that
  started a turn falls back to a fresh session under the requested preset, the way the TypeScript
  host does when its preset change is refused (`agent-preset/locked` is the preset registry's error,
  not the create path's answer).
- A preset now decides something the model sees: the roster may give a preset its own
  instructions, they are mounted in the prompt service, appended to the assembled system prompt,
  and the request header logs them. The mounted instructions are replaced wholesale, so a roster
  that no longer offers a preset leaves no instructions behind, and a roster that names
  instructions without a prompt service to hold them fails the load.
- A session is bound to the REVISION of the preset's instructions it was created, resumed, or
  last changed with. A roster change therefore does not alter a session that already runs a
  preset: it keeps the instructions it was bound to. Selecting the same preset again on a blank
  session records the change and binds the current revision, the way the TypeScript host
  recomposes a changed preset. A child session joins the exact revision its parent retained.
- A subagent's child inherits the preset its parent runs.
- The log's rows and its notifiers see one order: the sequence number is taken under the event
  lock, and the append and its delivery are serialised, so a concurrent append cannot land after
  the turn start it precedes. A notifier that appends to its own session fails loudly instead of
  waiting for itself.
- Known limit (from the GPT 6 Sol review): this build composes ONE agent loop, so a preset differs
  by the instructions it contributes, not by the tools it gets; the TypeScript preset registry
  mounts plugins per preset, which this port does not do.

## Log
- 2026-10-27 14:55: slash commands (`src/gui/commands.rs`, `tests/commands.rs`; `command/run` and `command/done` events in `src/types/session.rs` and `src/session/v4.rs`; `goals::{run,apply,current_view}` shared with `goals/*`; `compaction::COMPACTION_SERVICE`; `plugin::uuid_v4`). 660 tests pass; clippy output identical to before the slice; live parity 62/63 (the difference is the three commands not offered). Reviews: GPT 6 Sol fixed: the `/compact` in-flight flag now clears when the request is cancelled (RAII guard); a goal changed by a `goals/*` call between the write and the reply is an error text, not a panic; a failed log flush after a failed handler is logged beside the handler's failure; the anonymous id is published whole with a hard link, never an empty or partial file. Declined: an extra `goal_serial` mutex (it would not cover `goals/*`; `apply` already re-checks under its own lock); "file-only attachments need no store" (TS refuses all attachments without a store, and so does this); "unknown line must beat the session lookup" (TS validates and looks up the session first, verified live: ghost session + plain text gives `session/not-found`).
- 2026-10-27 13:50: image attachments (`src/images.rs`, `src/gui/images.rs`, `src/attachments.rs`, `tests/images.rs`). Prompts accept `image` parts; `session/attachment` returns an image a session's log references; the page gets `imageLimits` in the session projection. Admission follows the TS order: model gate, canonical base64, batch count/bytes/type, per-image checks, then store. A clean 8-bit PNG/JPEG/WebP is kept byte-identical; anything else is oriented, resized and re-encoded. The log holds only the reference; the model request is built from the stored bytes (`ContentBlock::ImageData`, never logged), as OpenAI `image_url` parts. Also fixed this turn: a blank session listed as "56y" old (`session_updated_at`). 634 tests pass; 22 TS-parity cases identical (`img_parity.mjs`); real Chrome: the default model refuses with `MODEL_DOES_NOT_SUPPORT_IMAGES`, a configured vision model answered "Red" and the page rehydrated the image; `ui_suite` passes.
  **Parity limits.** Image capability is config (`gui.imageModels`, needs `attachmentsDir`) because no adapter reports modalities and every TS catalog model refuses images; an unlisted route is refused exactly as TS does, and its history shows a text note in place of the picture. A transparent image is re-encoded as lossless WebP (TS: lossy), so it can be larger. The 16-bit/non-sRGB check is approximated by ICC presence plus bit depth. Request bodies may reach the aggregate image limit (about 267 MB) only while some model accepts images.
  **GPT 6 Sol review.** Fixed: a stored image that cannot be read now fails the model request instead of being dropped with a warning; `selectModel` waits for an admission in flight, so the gate and the sent route agree; `read_image` checks the object size against the reference before reading; an unreadable stored log is an internal failure, not "not found". Declined, with the TS source: text-only prompts are not serialized in TS either; batch checks follow canonical decoding in TS too; TS also keeps the smallest encoding when none fits the byte target; TS `carriesMetadata` does not count PNG `tIME`. The allowlist gate is the documented design above.
- 2026-10-27 12:10: file uploads (`src/gui/uploads.rs`, `src/attachments.rs`, `tests/uploads.rs`). The browser run also found that `session/list` sent `"cwd": null` for sessions with no working directory, which made the page throw in `workspaceTitleOf`; TS omits the key, and so does Rust now. 596 tests pass; 32 TS-parity cases identical; real-Chrome suites (`ui_suite.mjs`, `ui_sweep.mjs`) pass with no console errors or failed API calls. A GPT 6 Sol review found four real defects (a duplicate prompt left its receipts bound to a message that never lands; a disposal race left a receipt behind; concurrent saves of the same bytes could swap the object inode under an alias; digest checks read whole files). All four are fixed with tests. Two findings were wrong: removing a queued prompt retires its receipts in TS too (`retirePrompt`), and a receipt cited by two prompts moves to the later one in TS as well.
- 2026-10-27 07:40: `job/kill` answers `job/not-found` for every job (`src/gui/jobs.rs`); there is no job registry to consult.
- 2026-10-27 07:00: `llm/discoverModels` and the list-argument fix are done (`src/gui/model_discovery.rs`,
  `src/gui/empty_lists.rs`, `tests/model_discovery.rs`, `tests/empty_lists.rs`). `commands/list` used to answer `[]`
  for any arguments, so a client sending the wrong names saw an empty list; it now takes `agentId` and checks the
  session. Live parity vs TS: 20 list cases (19 identical; the 20th is TS's real command list) and 24 discovery
  cases (0 differ; object key order is not compared). `serde_json` sorts object keys, so `models` maps are read
  with an order-preserving visitor. Reviews: GPT 6 Sol (no file access, so source was inlined) and Grok 4.7. Accepted: no
  redirects (the key would otherwise travel to another host) and an exact-4-MiB boundary test. Rejected with evidence: "TS keeps
  duplicate ids" (`LlmRuntime.discoverModels` dedupes first-wins; live TS agrees), "`Policy::none()` errors" (the
  test shows the 3xx is reported). Same as TS, kept: no egress filter on the typed URL, a key is sent over `http://`,
  a `?query` in the base swallows `/models`, the base URL is echoed in error details.
- 2026-10-27 05:00: the user-questions slice is done (`src/plugins/ask_user.rs`, `src/plugins/frontend.rs`,
  `src/gui/user_questions.rs`, `tests/ask_user.rs`, `tests/user_questions.rs`). The approvals broker's
  "one attached frontend, revoked on replace" logic moved to `Frontend<Q>`; questions and approvals both use it
  (approvals tests unchanged). `userQuestions/answer`: 18 gateway cases identical to live TS. A real model
  (`cx/gpt-6-sol-high`) called the tool against 3081, a malformed answer was refused, the good answer reached the
  model. GPT 6 Sol raised duplicate ids (now `BAD_ARGS`), an agentless caller (now allowed, as in TS) and missing
  `EventHub` question tests (added); JEV baseline had no priorities. Limits: the timed mode is not ported; the
  `ask-user` plugin must be listed in the profile (the default profile has none); `ask_user_question` has no deadline.
- 2026-10-27 03:30: the message feedback slice is done (`src/gui/feedback.rs`, `tests/feedback.rs`, three log-only
  `SessionEventData` variants with v4 rows). Live parity vs TS on a real session: 30 cases identical, including
  the TS quirks (note checked before the session, empty id is `gateway/internal` for list/put but not-found for
  record, extra request fields ignored). A rating survives a restart of 3081. GPT 6 Sol (Opus 5.5 was rate-limited;
  GPT 6 Sol and Grok 4.7 did the planning pass instead): remarks now flush before the reply, a missing sessions
  service is a logged failure, the builder rejects a zero note limit. Limits: only loaded sessions are served (a stored
  one gets `gateway/service-unavailable`); the TS remark is not flushed, ours is. `uuid` (v4 only) added for version tokens.
  The session_v4 corruption test now flips the frame header: frames carry no checksum, so a body flip is not always caught.
- 2026-10-27 02:10: the plugin inventory slice is done (`src/bundle/roster.rs`, `src/gui/inventory.rs`,
  `src/gui/registry_probe.rs`, `tests/inventory.rs`). The kernel lists no fibers, so the bundle records each
  plugin it starts; a row holds a state watch, not the fiber handle, because a handle owns the whole runtime.
  Live parity vs TS: 6 cases identical (inventory entry key order differs). GPT 6 Sol: probe failures were
  silent (now logged; `null` is TS's offline reply), concurrent offline readers each probed (now one shared
  flight), roster rows were append-only (one row per name now), a missing roster listed nothing (now
  `service-unavailable`, and native plugins refuse to start without one), roster provider listed (no longer).
  JEV: all 8.0+ except coupling 7.9. `gui.registryProbeTimeoutMs` is validated config. 479 tests pass.
  `settings/openSettingsDocument` is unavailable by design, not a stub.
- 2026-10-27 00:50: the desktop path slice is done (`src/gui/desktop.rs`, `tests/desktop.rs`). Verified
  against the live TS host: 22 request cases, 0 differ; listings for a directory, a text file, an HTML file
  and `~/code` are identical (key order ignored). A path beneath a regular file is a failed query
  (`gateway/internal`), not a refusal, in both. GPT 6 Sol found five issues: pathname race
  (inherent to `open`/`xdg-open`, documented, same as TS); `nativeOpen: true` could enable WSL (now only a
  supported platform honours it, and WSL is detected as unsupported); capturing an opener's pipes could hold
  a request for as long as the application it started ran (openers now launch without pipes); `$BROWSER` with
  arguments (kept as TS reads it, documented, pinned by a test); SVG icons (PNG only). Shared wire helpers
  `boundary` and `check_descriptor` moved from `goals.rs` to `router.rs`. Pre-existing and not touched:
  592 `cargo fmt` diffs and 14 clippy errors in older files.
- 2026-10-26 23:20: the `goals/*` slice is done. Replies match the live TS host step for step
  (descriptor errors, boundary errors, the eight domain messages, the empty-ok reply for no goal, the
  `goal` projection); `goal/change` rows round-trip through the real TS validator. GPT 6 Sol found four
  issues (a goal id that was not a UUID; a revision cast that differed from TS for odd numbers; a
  projection push that could report a later state; a router-scoped lock, which is process-wide
  because one router is built per process) and two more on the fixes (JS number printing; the
  memory-only session case, now documented). Grok 4.7 answered with a plan, then timed out twice, so
  only GPT 6 Sol reviewed. Wire-name finding above. Todo dock: `todo_write` is served and the
  `todos` projection is live, but models send `todos` as a JSON string, which the TS validator
  refuses (`json-schema.ts:590`); this is a TS-side issue and needs a decision.
- 2026-10-26 08:55: the GUI that had been running on 3081 for nine hours was serving a stale
  binary — it was started before the last review round was built, so the three fixes that round
  produced (a candidate's own creation time, the no-cwd rank, the brackets a title keeps) were
  never live. The profile it ran with named the GUI bundle as a file where the port expects the
  directory holding `index.html`, so `/` answered 404 and no browser check of this port could have
  passed. The port now refuses an `assets` path that is not a directory when the profile loads,
  and the profile names `apps/web/dist`. The server was rebuilt and restarted on the current
  commit: `/` answers 200 with the injected boot, a real Chrome boots the client from it, and the
  three fixes are live. 433 tests pass.
- 2026-10-25 14:20: the list and search slice is committed (`bd812d4`): the stored sessions no
  live agent covers are listed and searched, the rows are ordered by activity, and an empty query
  is refused as a bad request. 409 tests pass; the live check on 3081 shows the rows in activity
  order, the search hit with its snippet and the refusal.
- 2026-10-25 13:55: the preset slice was reviewed twice more. GPT 6 Sol found six defects in
  total: a preset change logged after a turn started, a started session still reused, concurrent
  choices duplicating the row, stale mounted instructions, a concurrent append landing after the
  turn start it precedes, and a roster change altering a running session's instructions. All six
  are fixed (`f45a091`, `76f2a27`): the choice is one operation under the session's event lock,
  the append and its delivery are serialised so the log and its notifiers see one order, and a
  session is bound to the revision of the preset's instructions it was created, resumed, or last
  changed with. Grok 4.7 answered with a plan to read the code instead of findings, twice, so it
  is not an approval. 405 tests pass; the live check on 3081 shows the roster, the recorded change,
  the projection, the preset's instructions in the request header and a fresh session for a
  started one.
- 2026-10-25 23:58: the session reference candidates are served (`905397a`). A mention names the
  sessions the caller can see: each carries the canonical `@[label](dsh-session:…)` mention, the
  sessions of the caller's own working directory rank first, and a query filters on the session
  id, the working directory and the titles. The base64 alphabets moved into `src/gui/base64.rs`,
  which the file endpoints share. 430 tests pass.
- 2026-10-25 23:45: the workspace file slice is committed (`c4ecca4` feature, `5c1a794` and `73adfcd`
  review fixes). A session's files are read through the filesystem: `workspaceFiles/stat`,
  `read`, `readBytes` and `list`, the ranked file candidates, and the directory browser with its
  directory creation. The TypeScript host's refusals are the port's: `workspace-file/not-found`,
  `not-regular-file`, `not-directory`, `outside-workspace`, `not-text` and `too-large`. GPT 6 Sol
  found eight defects; five were fixed (a symlink is neither read nor walked into, a listing of a
  file says `not-directory`, a byte window above the cap is `too-large`, a page window cannot wrap
  around, a file that grows past the cap is refused), two were the TypeScript host's own order
  (a missing path is `not-found` before the containment check, and only the page's NUL bytes
  refuse a read), and one was a wrong document (`agentPresets/read` now serves the preset's
  declared child plugin list, which is empty in this build, instead of a generated instructions
  document). A second round found four more; the symlinked directory in a path query is refused
  and a child whose destination cannot be read keeps its own kind. JEV scored the slice with
  security 7.6 (up from 7.1), reliability 7.9 and duplication 7.9. 429 tests pass; the live check
  on 3081 answers every file endpoint and every refusal.
- 2026-10-25 13:25: the preset slice grew the TypeScript `agent-preset/selected` event, so a
  preset choice is in the log and the TypeScript reader sees it; a blank session may change the
  preset it runs, a started session is not reused, and a preset's own instructions reach the model
  and are logged with the request. GPT 6 Sol found four defects (a preset change logged after a
  turn started, a started session still reused, concurrent choices duplicating the row, stale
  mounted instructions); all four are fixed (`f45a091`). JEV scored the slice 6.9-8.4 per metric
  (weakest: test quality 6.9), and its two named risks are in: the cold reload keeps the preset a
  session changed to, and the session unit test covers the blank and started cases. Grok 4.7
  answered with a plan to read the code instead of findings, so it is not an approval. 403 tests
  pass; the live check on 3081 shows the roster, the recorded change, the projection and a fresh
  session for a started one.
- 2026-10-25 11:55: the live check on 3081 found one more defect the cold reload had introduced: a
  session a previous run stored keeps its id, so the counter that mints ids handed that id to the
  next session and `session/create` failed with "an agent with id session-1 already exists". The
  minting now skips the ids in use (`ff4c81f`).
- 2026-10-25 11:50: the agent preset slice is committed and survived a GPT 6 Sol review
  (`cx/gpt-6-sol-high`): a reused session refuses a different preset, the roster is validated at
  load, a subagent's child inherits its parent's preset. The two remaining findings are recorded as
  limits (one composed agent loop; the preset lives in the session header, not in an
  `agent-preset/selected` event).
- 2026-10-25 11:30: the cold reload slice is committed (`a51dd91` feature, `8af31fb` review fixes,
  `568c52f` route restore, `a8cc056` resume outcome). 394 tests pass; the browser check on 3081 shows
  the resumed session and a turn that ran in it.
- 2026-10-25 11:21: the cold reload survived a GPT 6 Sol review (5 findings, all fixed) and gained
  the TypeScript checkpoint policy (the request prefix is durable before the model is called, a
  recorded tool call before its body, fail-closed). 394 tests pass; the browser check on 3081 shows
  the resumed session, a turn that ran in it, and the appended log.
- 2026-10-25 11:12: cold reload. The session log is checkpointed when a turn ends, and the GUI
  adopts the stored sessions at startup, so a restart finds the conversation, its route and its
  working directory again. 390 tests pass; the browser check on 3081 shows the resumed session with
  its prompt and answer after a server restart.
- 2026-10-25 11:00 `4fc3de5`: the queue slice is through three review rounds (two GPT 6 Sol
  passes, one Grok 4.7 attempt that produced no findings, and two JEV rescores). 388 tests pass;
  the browser check on 3081 still shows the queued row, the steer, and no console errors.
- 2026-10-25 10:55 `1732b4c`: the queue slice survived a second review round, a JEV rescore and a
  Grok 4.7 attempt. 387 tests pass; the browser check on 3081 still shows the queued row, the steer
  and no console errors.
- 2026-10-25 09:22 DSH restarted by the user. Live probe re-run against the Rust GUI on 3081: 36 of 131 unary endpoints answer, 95 missing, 6 of 15 streams served — unchanged from the recorded state. The Rust GUI was relaunched with the quota-free Go profile; the TS Web GUI on 3080 was left alone.
- 2026-10-25 08:55 `720d3b2` committed: model selection appends its own `model/selection` seq, so the TS projection store keeps it. 373 tests pass.
- 2026-10-24 live Web/headless profiles staged with JEV-judged failover and quota-free OpenCode Go routes only; validated with `--dump-config` and the 33 decision-consumers tests. The live GUI was not restarted at that time; the user has now restarted it.
- 2026-10-28: `/permission` and `/plan` (slices B and C). New `permission` and `plan` plugins, `agent/step-start` waterfall in the loop driver, real preset catalog and `permissions`/`plan` projections (initial and live over the mux), v4 rows for the four new events. Tests: `tests/permission.rs` (13), `tests/plan.rs` (13), v4 round trips; full `cargo test` green. Not reviewed yet by a second model; nothing pushed.
- 2026-10-28: review pass by Opus 5.5 on `/permission` and `/plan`. Fixed: dangling-link and `link/..` escapes, missing or non-string `path`, unknown knob values, sessions without a working directory, shell under `workspace-write` with no approvals plugin, sub-agents starting unconfined, non-tightening-first knob writes, a pending plan choice that a restart lost, a race between `/plan` and the step that applies it, a bare `/plan` logged without arguments, and silent step-start failures. Declined: an allowlist of read-only tools (MCP tools have no capability tag here; `fileTools`/`shellTools` are configurable instead), O_NOFOLLOW file opens (belongs in the file tools), caching knobs per call (log scan is O(n) per call, same as the approvals plugin). Live check against a real model on a throwaway server: commands, catalog and projections; `workspace-write` refuses a write outside the workspace and allows one inside; plan mode puts the guidance in each request, `exit_plan_mode` reaches the browser over `$events`, and approval leaves plan mode before the next step. Nothing pushed.
