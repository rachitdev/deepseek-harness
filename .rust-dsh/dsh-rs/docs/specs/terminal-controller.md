# The `terminal/*` contract

The TypeScript host's `@deepseek-ai/dsh-terminal-controller` is the reference. This
spec records what the Rust host must answer, so the two agree endpoint by endpoint.

Reference sources (read-only):

- `~/code/deepseek-harness/packages/api/terminal-controller/src/{index,terminal,stream,retention,shells,types}.ts`
- `~/code/deepseek-harness/packages/subprocess/subprocess-local/src/{index,terminal}.ts`

## Endpoints

| endpoint | kind | arguments | answer |
|---|---|---|---|
| `terminal/shells` | unary | `agentId` (lookup) | `TerminalShell[]` |
| `terminal/environment` | unary | `agentId` (lookup) | `TerminalEnvironment` |
| `terminal/list` | unary | `sessionId` (json) | `WebTerminalInfo[]` |
| `terminal/create` | unary | `agentId` (lookup), `request` (json) | `WebTerminalInfo` |
| `terminal/close` | unary | `agentId` (lookup), `id` (json) | ok, no value |
| `terminal/rename` | unary | `agentId` (lookup), `id`, `title` (json) | ok, no value |
| `terminal/attach` | stream | `agentId` (lookup), `id` (json) | `TerminalFrame` items |
| `terminal/retain` | stream | `sessionId`, `id` (json) | one `{type:"retained"}` item, then silence |

A lookup parameter travels as `<name>Id` (`agentId`, `sessionId`); a `json`
parameter keeps its name.

## Types

```ts
interface TerminalShell { path: string; args: string[]; name: string }
interface TerminalEnvironment {
  cwd: string; maxInputBytes: number; maxBufferedBytes: number
  maxCols: number; maxRows: number; scrollback: number
}
interface WebTerminalInfo {
  id: string; title: string; shell: TerminalShell; cwd: string
  cols: number; rows: number
  state: 'running' | 'exited' | 'failed'; exitCode: number | null; error?: string
}
```

The TypeScript defaults: `maxTerminals` 8, `maxCols` 500, `maxRows` 200,
`scrollback` 1000, `maxBufferedBytes` 2 MiB, `maxInputBytes` 64 KiB,
`disposeGraceMs` 1000, `unattendedTimeoutMs` 7 200 000.

## Shell discovery

`terminal/shells` answers the preferred shell first, then every candidate that
resolves, deduplicated by shell kind (the executable's lower-cased name without
`.exe`).

- The preferred shell is the configured one, else `$SHELL`, else `/bin/sh`.
- Candidates are `zsh`, `bash`, `fish`, `pwsh`, `powershell`, `cmd`, in that order.
- A candidate is profiled before it is resolved: `cmd` gets no arguments,
  `pwsh`/`powershell` get `-NoLogo`, everything else gets `-i`.
- The answer keeps the resolved absolute path, the executable's own name, and
  those arguments. On the reference host the answer is
  `[{"path":"/bin/zsh","name":"zsh","args":["-i"]},{"path":"/bin/bash","name":"bash","args":["-i"]}]`.

## Creation

`terminal/create` takes `request` `{id, cols, rows, shellPath?}`.

- `id` must match `^[\w-]{1,128}$`; otherwise `gateway/internal` with the
  message `Invalid terminal identity`.
- `cols` must be a safe integer in `2..=maxCols` and `rows` in `1..=maxRows`;
  otherwise `gateway/internal` with `Terminal dimensions exceed the configured limits`.
- A `shellPath` that is not one of the discovered shells is refused with
  `gateway/internal` and `Selected shell is not available in this execution environment`.
- Creating the same id twice answers the existing terminal's info.
- A session may hold at most `maxTerminals` terminals; the next creation is
  refused with `terminal/limit-reached`, message `Session terminal limit reached`,
  `details.limit` = the limit.
- The title starts as the shell's name; `cwd` is the session's working directory.

## Naming, closing, listing

- `terminal/rename` trims the title and refuses an empty or longer-than-120
  title with `gateway/internal` and `Terminal title must contain 1–120 characters`.
  The title it stores is the trimmed one.
- `terminal/close` closes the terminal, forgets it, and answers ok. Closing an
  unknown terminal is ok too. A closed id cannot be created again: the creation
  is refused with `terminal/unavailable` and `Terminal was closed in this Session`.
- Renaming or attaching to an unknown terminal is refused with
  `terminal/unavailable` and `Terminal no longer exists in this Session`.
- `terminal/list` answers the session's terminals, in creation order. A session
  with none answers `[]`, and so does an unknown session.

## Errors

| situation | code | message |
|---|---|---|
| unknown session (agent) | `session/not-found` | `session "<id>" not found` |
| unknown or closed terminal | `terminal/unavailable` | `Terminal no longer exists in this Session` |
| terminal limit | `terminal/limit-reached` | `Session terminal limit reached` |
| bad identity, dimensions, title, shell | `gateway/internal` | as quoted above |

## Attachment (stream)

`terminal/attach` opens a logical stream and answers frames:

```ts
type TerminalFrame =
  | { type: 'snapshot'; sequence: number; screen: string; info: WebTerminalInfo; writable: boolean; inputWindow?: { maxBytes: number; frameOverheadBytes: number } }
  | { type: 'output';   sequence: number; data: string }
  | { type: 'state';    info: WebTerminalInfo; writable: boolean }
  | { type: 'ack';      sequence: number; refusal?: 'read-only' | 'not-running' }
```

- The first frame is the `snapshot`: the screen as the TypeScript host's
  `SerializeAddon.serialize()` produced it, the terminal's info, whether the
  attachment may write, and the uplink window (`maxBytes` 262 144,
  `frameOverheadBytes` 40).
- The client sends `TerminalInput` items: `{type:'write', sequence, data}` or
  `{type:'resize', sequence, cols, rows}`. `sequence` must be the previous
  sequence plus one; a gap is `gateway/internal` with
  `Terminal input sequence has a gap`.
- A write larger than `maxInputBytes` is `gateway/internal` with
  `Terminal input exceeds the configured limit`.
- A write to a terminal that is not writable (another attachment holds the
  input) is answered with `{type:'ack', sequence, refusal:'read-only'}`; a write
  to an exited terminal is `{type:'ack', sequence, refusal:'not-running'}`. Both
  are ordinary frames, not stream failures.
- Output arrives as `{type:'output', sequence, data}` frames; the sequence
  counts every frame the terminal sends.
- The most recent attachment controls the input; the others become read-only and
  are told with a `{type:'state', info, writable:false}` frame.

## Retention (stream)

`terminal/retain` answers one `{type:'retained'}` frame and then holds the
stream open. A terminal that is closing or unknown is refused with
`terminal/unavailable` and `Terminal is closing or unavailable`.
