# dsh-rs — everything is a plugin, in Rust

[![crates.io](https://img.shields.io/crates/v/dsh-rs)](https://crates.io/crates/dsh-rs)
[![docs.rs](https://img.shields.io/docsrs/dsh-rs)](https://docs.rs/dsh-rs)

> [**中文版 README**](README.zh-CN.md) · [English](README.md)

A standalone Rust port of [DeepSeek Harness](https://github.com/deepseek-ai/deepseek-harness)
(`dsh`): an agent harness where **everything is a plugin**, built on
[cordis-rust](https://github.com/redoop/cordis-rust) — the Rust port of the
cordis kernel that powers the original DSH plugin system. Published on
crates.io as a **single crate**:

```toml
[dependencies]
dsh-rs = "0.2"
```

```console
$ cargo install dsh-rs     # the dsh binary (provider config via ~/.dsh/config.json)
$ dsh run --prompt "hi" --provider opencode
```

The mental model is identical to the reference implementation: model adapters,
tools, session persistence, the system-prompt assembler, and the agent loop
itself are all plugins providing services on one shared `Context`. There is no
privileged core to patch — you extend the harness by mounting a plugin beside
the others, and every registration is an effect that unwinds when its plugin
unloads.

## The one idea

Nothing wires anything by hand. A **plugin** declares what it *requires*
(`inject`) and what it *provides* (`ctx.provide`). The cordis kernel computes
the dependency graph and converges every fiber toward its desired epoch:

```text
register consumer first (PENDING)
      |  provider appears
      v
provider LOADING --> ACTIVE --notify--> consumer re-checks --> ACTIVE
```

In dsh-rs the agent loop is a plugin that `inject`s `sessions`, `systemPrompt`,
`tools`, `llm`, and `llmStreams` — so it only activates once the whole spine is
live, and unloads (LIFO) when any dependency disappears.

## Architecture vs the reference harness

| DeepSeek Harness (TS) | dsh-rs | `ctx` key |
| --- | --- | --- |
| `packages/llm` — Message / ContentBlock / StreamChunk, adapter seam | `dsh-llm` | `llm`, `llmStreams` |
| `packages/core/session` — append-only `SessionEvent` log | `dsh-session` | `sessions` |
| `packages/core/tools` — scoped registry + guarded pipeline | `dsh-tools` | `tools` |
| `packages/core/system-prompt` — prompt sections/variables | `dsh-core` | `systemPrompt` |
| `packages/core/agent` + `agent-loop` — Agent handle + driver | `dsh-core` | `agents` |
| `dsh-base` bundle composition | `dsh-bundle` | — |
| `dsh` headless CLI | `dsh-cli` | — |

The event vocabulary and turn flow mirror the reference:

```text
turn/start
  claim next-step input plus one queued message
  assemble prompt sections + tool schemas
  -> agent/pre-step                   reject | enter(messages)
     step/start
     append entered messages as user/message
     derive model history from the log
     agent/request -> llm/stream -> assistant/chunk* -> assistant/message
     tool/call* -> tools/pre-execute -> guards -> tools/execute -> tool/result*
     step/end
     tools owe another request, or next-step input arrived -> next step
  -> agent/turn-stopping
turn/end
```

`turn/*`, `step/*`, `user/message`, `assistant/*`, and `tool/*` are durable
session events; `agent/pre-step`, `agent/request`, `llm/stream`, and the three
`tools/*` events are cordis **waterfalls** — listeners may veto or rewrite by
calling `next()`. The `session/event` firehose feeds persistence, and
`session/flush` is the awaited durability checkpoint.

## Layout (single crate)

```text
Cargo.toml       package dsh-rs 0.2.0 (lib `dsh_rs` + bin `dsh`)
src/
  types/         shared vocabulary — messages, streams, session events, agent options
  api/           interface layer — service traits + wrappers, typed events, manifest
  llm/           adapter seam — mock + OpenAI-compatible SSE adapter, block assembler,
                 provider routing (feature "openai")
  session/       append-only SessionEvent log, SessionStore, surface projection
                 (derive_messages), JSONL persistence with crash-turn repair
  tools/         ToolRegistry with the guarded pipeline (pre-execute -> guards ->
                 execute -> post-execute); built-ins: bash, read_file, write_file,
                 edit_file, glob, grep, todo_write
  core/          SystemPromptService (sections/contexts/variables), agent registry +
                 handle (followup/steer/inject/cancel/when_idle), the agent-loop driver
  bundle/        base-bundle composition, JSON profile installer, manifest registry,
                 dynamic-plugin host
  cli/           runner helpers + terminal chat UI; the `dsh` binary is src/main.rs
tests/           integration tests (chat flow, TUI state, dynamic plugins, ...)
crates/
  dsh-plugin-contract/   published C-ABI contract (0.2.0, crates.io)
  dsh-plugin-hello/      example cdylib plugin (built standalone)
```

## Build & test

```sh
cargo build     # single crate: build from the repository root
cargo test      # 70 tests, all green
```

## Run

The `mock` provider needs no network: it echoes the last user message, and can
be scripted for tool-call flows (used throughout the test suite).

```sh
# One prompt against the mock provider
./target/debug/dsh run --prompt "list the files" --cwd /tmp

# Interactive chat — a full TUI on a terminal, line mode when piped
./target/debug/dsh chat          # TUI: enter to send, ↑/↓ history,
                                 #      pgup/pgdn scroll, ctrl-u clear,
                                 #      ctrl-c / ctrl-q quit (IME-safe: bare
                                 #      'q' never quits)
printf "hello\n" | ./target/debug/dsh chat   # piped input → line mode
./target/debug/dsh chat --line   # force line mode

# Persist sessions to a JSONL store, then read a transcript back
./target/debug/dsh run --prompt "hello" --store /tmp/dsh-sessions
./target/debug/dsh transcript session-1 --store /tmp/dsh-sessions

# OpenAI-compatible provider (chat-completions over SSE)
./target/debug/dsh run --prompt "hello" \
    --provider openai --model gpt-4o-mini \
    --openai-base https://api.openai.com/v1 --openai-key sk-...

# JSON profile: { "bundles": ["base"], "config": { "store_dir": ..., "openai": ... } }
./target/debug/dsh run --prompt "hello" --profile profile.json
```

## DeepSeek configuration

`dsh` auto-discovers a default config at `$DSH_CONFIG`, `~/.dsh/config.json`, or
`./dsh.json` (first match wins; `dsh.json` is gitignored so keys never get
committed). With it in place, `dsh run` and `dsh chat` (TUI) talk to DeepSeek
out of the box — the agent loop streams, and tool calls (bash, file tools,
...) execute for real:

```json
{
  "provider": "deepseek",
  "model": "deepseek-v4-flash",
  "store_dir": "/tmp/dsh-sessions",
  "openai": {
    "providers": ["deepseek", "openai"],
    "base_url": "https://api.deepseek.com/v1",
    "api_key": "sk-...",
    "model": "deepseek-v4-flash"
  }
}
```

```sh
./target/debug/dsh chat                 # TUI against DeepSeek
./target/debug/dsh run --prompt "请执行 date 命令并把结果告诉我"
```

The OpenAI-compatible adapter is tuned for DeepSeek's thinking mode: reasoning
deltas get their own block index, `reasoning_content` is echoed back on later
requests (DeepSeek rejects calls that omit it), and empty `tool_calls` arrays
are omitted from wire messages.`

## Testing `dsh chat`

`crates/dsh-cli/tests/chat_flow.rs` verifies the chat TUI at three layers:

- **Binary smoke test** — spawns the real `dsh` binary in a hermetic
  environment (temp `HOME`, no default config, mock provider) and asserts a
  full piped conversation: banner, one echo per line, clean exit.
- **Key handling** — drives `ChatTui::handle_key` against a live harness:
  typing, Enter-to-submit, `↑/↓` history recall, `Esc` clear, and `q` /
  Ctrl-C quit semantics.
- **Rendering** — renders frames into a ratatui `TestBackend` buffer and
  asserts the visible layout (title + session id, conversation lines, input
  prompt, status/hint), including a scripted tool-call turn showing
  `bash` → result → final answer.

```sh
cargo test --test chat_flow   # 16 tests, no terminal required
cargo test                      # 70 tests, all green
```

## Dynamic plugins (independent compilation + runtime loading)

A plugin can be **compiled independently as a cdylib** and **loaded at runtime**
— the host dlopens it and adapts it to the cordis `Plugin` trait. The plugin
speaks only a zero-dependency C-ABI contract (`dsh-plugin-contract`) and
**never links cordis or tokio**: the host implements the framework, the plugin
only computes (the same design as cordis-rust's `dynhost`).

```sh
# 1. compile the example plugin as a standalone library
(cd crates/dsh-plugin-hello && cargo build)

# 2. load it into a running harness; it declares its tools and registers them
./target/debug/dsh plugin load crates/dsh-plugin-hello/target/debug/libdsh_plugin_hello.so
#   loaded plugin: dsh-plugin-hello (state: Active)
#   declared tools: dsh_hello
#   tools now registered: bash, dsh_hello, edit_file, ...

# 3. the agent loop can now call dsh_hello like any built-in tool
./target/debug/dsh run --prompt "use dsh_hello" --provider mock
```

A plugin's JSON declaration (`describe`) lists the services it needs
(`inject`) and the tools it provides; each tool's `execute` calls back into
the library via `invoke`. Loading, registration, and unload are all cordis
fiber effects — unloading the plugin unregisters its tools automatically.
A plugin with `inject: ["tools"]` converges only after that service is live,
exactly like the static plugins. Add your own plugin: implement
`DshPluginExports` (see `crates/dsh-plugin-hello`) and export it as
`dsh_plugin_exports`.

## Extending

Add a tool: build a `ToolDefinition` and register it on `ctx.tools`. Add a
provider: implement `LlmAdapter` and register it on `ctx.llm`. Add a prompt
section: `ctx.systemPrompt.section(...)`. Intercept behavior: listen on the
`agent/pre-step`, `agent/request`, `tools/pre-execute`, or `llm/stream`
waterfalls. Everything is a plugin, and everything is replaceable from
configuration.

## DSH ecosystem plugins

dsh-rs runs on [jincdream/cordis-rs](https://github.com/jincdream/cordis-rs) through the `crates/cordis-compat` shim, which keeps the old cordis API and pins the kernel by git `rev`. Ecosystem plugins are named in a profile's `plugins` object (`dsh run --profile FILE`); `${VAR}` placeholders in string values expand from the environment (an unset variable is an error), and `"enabled": false` skips an entry. See `profiles/ecosystem.example.json`.

| Plugin | What it does |
|---|---|
| `cortexdb-memory` | `cortexdb_memory_search/store/forget` over the sidecar REST API, plus auto-recall before each prompt. `readOnly` hides the write tools. |
| `skillforge-router` | `skillforge_route` / `skillforge_pack_atoms` via the `skillforge` CLI; optional `autoHint` (labelled `unranked`). |
| `skills` | Installs every `<dir>/<name>/SKILL.md`: a catalog in the system prompt and a `skill` loader tool. |
| `mcp-client` | Stdio MCP client; each server tool becomes `mcp__<server>__<tool>` (e.g. `jev_review`). |
| `ts-dsh-bridge` | `dsh_ecosystem_run` delegates a task to the TypeScript DSH through [`deepseek-harness-sdk`](https://github.com/42ch-dev/dsh-rust-sdk) (Apache-2.0, pinned by git rev). Runs unattended; approval-gated actions fail closed. |

### Limits and behaviour worth knowing

- **Startup is all-or-nothing.** Bundle names, the `plugins` map and every plugin name are validated before anything starts; if one plugin fails or does not activate (for example a service it needs is missing), everything already started is unloaded and the error names it. Errors about configuration report types, not values, so secrets are not echoed.
- **A provider key that is named but missing is an error.** `api_key_env` pointing at an unset or empty variable fails at load instead of sending unauthenticated requests.
- **CortexDB:** every response is shape-checked (a malformed answer is an error, not an empty result) and capped at 4 MiB. `cortexdb_memory_forget` deletes by id and cannot be scoped; the sidecar has no per-user access control, so treat it as a single-user local service.
- **Skillforge / MCP:** CLI output is streamed with a 4 MiB cap and the child is killed at the cap or the timeout. MCP lines are capped at 8 MiB, `tools/list` is paginated (20 pages, 500 tools), the protocol version is checked, requests the server makes of the client are answered "method not found", a stalled server is dropped rather than waited on, and there is no automatic reconnect. Server names must not contain `__`; duplicate tool names are skipped with a log line.
- **Skills:** `SKILL.md` must be a regular file of at most 512 KiB; a symlinked manifest is refused (a symlinked skill *directory* is fine). The catalog section is removed when the plugin unloads.
- **Bridge:** work delegated to the TS DSH that times out or is cut off by an unload is abandoned on this side; what it had already done is unknown, and the error says so and gives the session id. Calls after unload are refused.
- **Not provided:** per-plugin hot reload of the TS ecosystem inside Rust (the bridge delegates, it does not load TS plugins natively), JEV re-ranking of Skillforge hints, and the Responses API (the OpenAI adapter uses chat-completions).

Providers can name an env var instead of a key: `"adapters": {"omniroute": {"base_url": "...", "api_key_env": "OMNIROUTE_API_KEY"}}`.

### Built-in agent plugins

Named in the same `plugins` object; see `profiles/agent.example.json`. Every setting is validated at load, and a bad value fails the profile with the setting's name.

| Plugin | What it does | Settings (defaults) |
|---|---|---|
| `web` | `web_fetch` and `web_search`. Fetches only http(s) to public addresses: DNS is resolved and every address vetted (loopback, private, link-local, mapped/NAT64/6to4 included), the connection is pinned to the vetted address, each redirect is re-checked, and the body is capped. Results are labelled as untrusted. Search uses Exa; a missing `$EXA_API_KEY` fails the load unless `searchProvider` is `"none"`. | `allowPrivateHosts` (false), `maxRedirects` (5), `maxBodyBytes` (2 MiB), `maxChars` (20000), `fetchTimeoutMs` (20000), `searchProvider` (`exa`), `searchApiKeyEnv` |
| `approvals` | Asks before listed tools run. `dsh chat` shows a prompt: `y` allow once, `a` allow for the rest of the run, `n`/Esc deny. With no approver attached (`dsh run`, the bridge) or on timeout the call is **denied**. Patterns may end in `*`; a tool in both `ask` and `allow`/`deny` is a load error. | `ask` (`bash`, `write_file`, `edit_file`, `dsh_ecosystem_run`), `allow`, `deny`, `default` (`allow`), `timeoutMs` (120000) |
| `compaction` | Before a turn, if the visible history is estimated above the trigger, the model summarises everything older than the last `keepRecentTurns` user turns and a `compaction` event is logged. If the summary fails, is empty or is not shorter, nothing changes and the turn runs with the full history. | `triggerTokens` (60000), `keepRecentTurns` (2), `minReclaimTokens` (2000), `summaryMaxTokens` (1500), `timeoutMs` (60000), `provider`/`model` (the agent's own) |
| `subagent` | `subagent` tool: a fresh agent with the same tools and model runs a self-contained task and its final answer comes back. It does not see the parent's conversation. | `maxDepth` (1), `maxConcurrent` (4), `timeoutMs` (600000), `maxResultChars` (20000) |
| `goal` | `goal_set`, `goal_get`, `goal_update`: one durable objective per session, kept in the log. An active goal is shown to the model once per compaction window. | `maxObjectiveChars` (2000), `maxNoteChars` (1000), `remind` (true) |

Limits:

- **Compaction** estimates tokens as characters / 4, cuts only at a user-turn boundary (one very long turn cannot be compacted until it ends), and cannot be interrupted by cancelling the turn, only by its timeout. A reloaded session shows exactly the history the model saw.
- **Sub-agents** cannot ask the parent questions, return no partial answer on timeout, and are subject to the same approvals as the parent (denied when nothing can approve).
- **Goals** are recorded and surfaced; the plugin does not loop the agent until the goal is met and cannot verify a claim of "achieved".
- **Web:** the fetch tool does not run JavaScript; only text-like content types are returned.

### Node sidecar: the TypeScript plugin manager and the dynamic Cordis runner

Two subsystems of the TypeScript host cannot be reimplemented in Rust without
a second implementation of the thing that makes them work:

- **`pluginManager`** (15 endpoints) installs, updates, removes and versions
  npm plugin bundles with `pnpm`. This host has no npm ecosystem — no plugin
  store, no registry.
- **`dynamicCordisRunner`** (10 endpoints) evaluates untrusted plugin code in
  a JavaScript sandbox (`sandbox.ts`, ~1,073 lines of the ~3,400-line package)
  and drives a plugin's client half in the browser. This host has no
  JavaScript engine.

The plan, the protocol and the trust boundary for hosting both in a **Node
sidecar** are written down in `docs/specs/node-sidecar.md`, and the Rust half
of the bridge (`src/sidecar/protocol.rs`, `src/sidecar/client.rs`) is written
and tested: a framed JSON protocol over an inherited socket, a token
handshake, calls with cancellations, and one recovery policy. In this build
the endpoints answer natively instead:

| Endpoint group | What this host answers |
|---|---|
| `pluginManager/listBundles`, `listPlugins`, `registries`, `listVersions`, `listVersionExemptions`, `readBundleDetail` | Read-only shapes from the profile's bundle names and the plugin roster |
| `pluginManager/*` mutations | The TypeScript host's operation-result shape with `application: "failed"` and a diagnostic that says this host has no npm bundle store |
| `pluginMarket/listIndexes`, `setIndexes` | The settings store's `plugin-market` namespace, so a commit is durable before the call answers |
| `pluginMarket/search`, `readEntryDetail` | The TypeScript host's own refusal shapes |
| `dynamicCordisRunner/*` (10 endpoints) | The TypeScript host's own shape and wording for "no such plugin" and "no such run" |

Running the TypeScript manager in a sidecar would install bundles into a
directory this host never reads, which is worse than refusing: it would look
like it worked. The honest answer is the one above, and `docs/DIVERGENCES.md`
records every place it differs.

### Updating from GitHub

```sh
scripts/update.sh kernel check        # pinned vs upstream cordis-rs
scripts/update.sh kernel apply [REV]  # bump the pin, run cargo test, restore on failure
scripts/update.sh self check          # commits on origin not yet merged
scripts/update.sh self apply          # merge origin, run cargo test, reset on failure
```

Both `apply` modes refuse a dirty tree and never push. If tests fail, or the run is interrupted with Ctrl-C/SIGTERM, tracked files are restored (the manifest and lockfile, or `HEAD`); untracked files a failing test created are left, and SIGKILL cannot be trapped. A signal is acted on once the running `cargo test` exits.

## License

MIT
