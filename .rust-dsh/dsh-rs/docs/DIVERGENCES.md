# Where this build differs from the TypeScript host

The port answers 199 of 199 endpoints. This file is the register of every
place where its answer is not the TypeScript host's, and of why.

An entry is a **gap** when the Rust host could have done the same thing and
did not — a missing behaviour, a shortcut, a bug to fix. An entry is a
**consequence** when the Rust host has no state the TypeScript host has — no
npm ecosystem, no JavaScript engine, no speech provider — so the answer
differs because the thing being asked about is not there.

The order is the order of the parity table, so a reader can walk the
endpoints and check each one.

## The session log

- **Closed.** A new session logs three setup rows (`permission/preset`,
  `sandbox/mode`, `approval/policy`, seq 0 to 2) on creation matching the
  TypeScript host when the permission plugin is active. `session/state` and
  `session/page` now exhibit 0 differential drift across empty sessions.

## Sub-agents

- **Consequence.** A child lives only while the parent's `subagent` tool call
  runs, so a stored child cannot be resumed; the TypeScript host revives it.
  A `continuable` address is refused.

## Credentials

- **Consequence.** The `.env` fallback layers (`<cwd>/.env`, `$DSH_HOME/.env`)
  are not read, so a reference that exists only in a `.env` file shows as not
  configured.
- **Gap.** A write re-renders the whole document through the YAML parser, so
  comments and hand formatting inside `refs:` are not preserved the way the
  TypeScript host preserves them (it patches the text in place). Unrelated
  top-level sections are kept.
- **Gap.** The document is re-read per call instead of watched, so an external
  edit is seen on the next call rather than immediately.

## Tool arguments

- **Gap.** When an object carries several undeclared keys, the violations
  list them sorted rather than in the order the model wrote them: a parsed
  JSON object has no key order in this build.

## Schedule

- **Consequence.** The `schedule` plugin's tools need the `gui` plugin to be
  loaded first, because the `gui` plugin owns the service.
- **Gap.** The activity that blocks an archive is wired directly in the
  router rather than through the TypeScript host's
  `workspace/session-activity` event, so a third-party activity provider
  cannot join it.

## The terminal

- **Gap.** A client's uplink items are counted with the whole frame's bytes,
  as the TypeScript host does, but this build counts the frame text it
  parsed, so a frame the host rewrites would count differently.
- **Gap.** The mux admits one socket per connection, as the TypeScript host
  does, but a *second* socket on the same port is refused with 409, so a
  client with two windows needs one socket.
- **Gap.** The terminal's output buffer is a broadcast channel of 256 frames:
  a client that falls further behind is failed with `Terminal output consumer
  exceeded its buffer`, where the TypeScript host fails the follower with a
  slightly different wording and no frame count.
- **Gap.** The process cleanup signals bare pids. The TypeScript host keeps
  each process's start identity and rechecks it before every signal, so a pid
  that was recycled cannot be signalled here.
- **Gap.** The retention monitor cannot read the TypeScript host's shell
  prompt evidence, so a shell that is running a builtin looks idle to it.
- **Gap.** The write's acknowledgement is queued before the write, so a write
  that fails is acknowledged as a success.
- **Gap.** A resize publishes the screen and the dimensions in three steps
  without the frame lock, so an attachment's baseline can catch a resized
  screen with the old dimensions.
- **Gap.** The reclamation test proves neither the process's death nor the
  slot's release, because the terminal is marked reclaimed before its cleanup
  finishes.
- **Gap.** A write that blocks on a child that stopped reading holds the
  attachment's select branch, so neither the cancellation nor the output is
  delivered while it blocks.

## The office document preview

- **Gap.** The converted document's `version` is this host's own, and the
  service's `generation` is a fresh UUID per host. The TypeScript host's two
  hosts would answer the same way.

## The settings store

- **Gap.** The document holds one object per namespace
  (`{"<ns>": {"user": {…}, "revision": n}}`). The TypeScript host writes these
  values into the profile's own entries, so the file's shape differs.

## The plugin manager and the plugin market

- **Consequence.** This host has no npm ecosystem: no plugin store, no
  `pnpm`, no registry. The manager's answers come from this host's own state
  — the profile's bundle names and the plugin roster — and every mutation
  answers the TypeScript host's own operation-result shape with
  `application: "failed"` and a diagnostic that says this host has no npm
  bundle store. The npm version, the package description and the localized
  metadata are absent, because this host has none.
- **Gap.** The market's `search` and `readEntryDetail` do not read an index:
  they answer `fetch-failed`. The fetching, the parsing, the searching, the
  annotation, the caching and the changed-event emission are not ported.
- **Consequence.** The market's index list is kept in the settings store's
  `plugin-market` namespace rather than in a volatile plugin entry.

## Speech

- **Consequence.** This host has no speech provider: the catalog advertises
  none, and a preparation, a cancellation and a transcription are refused with
  the TypeScript host's code and message. The catalog's limits are the
  TypeScript host's defaults.

## The reader's translation

- **Consequence.** This host has no translation provider, so a translation is
  refused with the TypeScript host's code. The limits and the preferences are
  served.

## Computer-use access

- **Consequence.** This host has no native computer-use client. The
  permission state is the state a host without one has, the application list
  is empty, and the permission requests are refused with the TypeScript
  host's codes.

## Live sharing

- **Consequence.** This host shares nothing: the share list is empty, the
  owner identity is unverified, and every call that acts on a share answers
  the refusal for a share this host does not know.

## The forge panel

- **Consequence.** This host has no forge and no git. The branches, the
  diffs and the texts are empty, the mutations are refused with the
  TypeScript host's codes, and the two streams answer their baseline and
  wait.

## Dynamic Cordis plugins

- **Consequence.** This host has no JavaScript engine, and its browser client
  has no dynamic-plugin support, so no plugin run can complete. The ten
  endpoints answer the TypeScript host's own shape and wording for "no such
  plugin" and "no such run", including its missing-plugin message.

## Action cards and plugin recommendations

- **Consequence.** This host logs no cards, so both endpoints answer the
  TypeScript host's successful refusal `{status: "refused", reason: "unknown"}`.

## Product analytics

- **Consequence.** This host has no telemetry backend and no identity. The
  intake accepts an event and drops it; the policy is a configuration value.

## The workspace file watch

- **Gap.** The TypeScript host watches through the filesystem provider and
  sees a change as the operating system reports it. This build samples the
  file's version on an interval, so a change reverted between two samples is
  not seen, and a change is seen up to one interval late.

## User questions

- **Consequence.** This host has no timed question, so `userQuestions/
  attachWait` ends without frames — which is the TypeScript host's own answer
  when no wait exists, but not its timed behaviour.

## Tool results

- **Gap.** The TypeScript host keeps the live value of a tool result until it
  is read; this build reads the recorded payload from the session log, so the
  value is the one that was committed.

## The Platform sign-in

- **Gap.** The grant is written to the credential document's `records`
  section. The TypeScript host keeps provider records in the same document,
  under its own key.