# The Node sidecar

## Why

Twenty-five of the endpoints this build still does not serve are TypeScript
subsystems, not wire formats:

| group | endpoints | the TypeScript package |
|---|---|---|
| `pluginManager` | 15 | `@deepseek-ai/dsh-plugin-manager` — the bundle registry: install, update, remove, versions, exemptions |
| `dynamicCordisRunner` | 10 | `@deepseek-ai/dsh-cordis-host-runner` — a sandbox that evaluates plugin code and drives a Cordis registry |

`dynamicCordisRunner` evaluates untrusted plugin code in a JavaScript sandbox
(`packages/extensions/cordis-host-runner/src/sandbox.ts` and `guard.ts`,
1,073 lines) and drives a plugin registry, an inspection registry and a client
half over a duplex channel (`index.ts`, `registry.ts`, `inspect-registry.ts`,
`lifecycle.ts`, `types.ts`, 2,318 lines). Reimplementing that in Rust would be a
second implementation of a sandbox, with its own divergences in the one
component whose whole job is to run foreign code safely. The honest port is to
run the TypeScript subsystem where it already lives, and let the Rust host
serve its answers.

The same sidecar later carries `pluginMarket` (4), `pluginRecommendations`
(1), `actionCards` (1) and `cotTranslation` (2): every one of them is a
TypeScript package with its own protocol, and none of them is a wire format a
Rust reimplementation would be simpler for.

## What the implementation decided

The note below was written first, as the plan. The implementation did not
follow it, and this section records why, so the note is not read as a
description of something that was built.

**The sidecar was not built.** The two groups it was written for turned out
not to need it:

- **`pluginManager` (15 endpoints) and `pluginMarket` (4).** The manager is a
  package manager for npm bundles: it resolves versions, runs `pnpm`, and
  installs into the host's plugin store. This host has no npm ecosystem — no
  plugin store, no `pnpm`, no registry — so there is nothing for any
  implementation to install *into*. Running the TypeScript manager in a
  sidecar would install bundles into a directory this host never reads, which
  is worse than refusing: it would look like it worked. The port is native
  instead: the manager's answers come from this host's own state (the profile's
  bundle names, the plugin roster, the settings store), and every mutation
  answers the TypeScript host's own operation-result shape with
  `application: "failed"` and a diagnostic that says this host has no npm
  bundle store. The market's index list lives in the settings store's
  `plugin-market` namespace, so a commit is durable before the call answers.

- **`dynamicCordisRunner` (10 endpoints).** The runner evaluates untrusted
  plugin code in a JavaScript sandbox and drives the plugin's client half in
  the browser. A sidecar could host the host half, but the client half runs in
  the browser, and this host's browser client has no dynamic-plugin support —
  so no run could ever complete, and a `runHostHalf` that answers `ok: false`
  is the truthful answer. The port answers each of the ten endpoints with the
  TypeScript host's own shape and wording for "no such plugin" or "no such
  run", including its missing-plugin message.

What is left of the note is the protocol and the trust boundary, which are
still the right description of how a TypeScript subsystem would be hosted if
one ever needed it. The Rust half of it — `src/sidecar/protocol.rs` and
`src/sidecar/client.rs` — is written and tested, and is not yet wired to an
endpoint.

## The shape

```
┌──────────────────────── dsh-rs host ────────────────────────┐
│  gateway ── router ──► endpoint group                       │
│              │                                               │
│              │  pluginManager/*  dynamicCordisRunner/*        │
│              ▼                                               │
│         sidecar client ──(inherited socket)──► sidecar      │
└──────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
                    ┌──────────────────────────────┐
                    │ Node process: the TypeScript │
                    │ packages, on their own       │
                    │ services                     │
                    └──────────────────────────────┘
```

The sidecar is a Node process that boots the TypeScript packages on their own
services and answers their remote methods. It holds no session, agent or tool
state: the plugin registry and the sandbox are its whole world. The Rust host
owns the sessions, the agent loop, the tools, the files, the schedule, the
terminal, the account's own state, the gateway and the Remote stream protocol
(the harness's duplex client protocol, implemented in
`packages/api/gateway/src/stream-server.ts` and
`packages/typert/protocol/src/`). Neither side holds the other's state, and
the only thing that crosses between them is the bridge below.

## The inventory

The contract of every endpoint is the inventory the gateway already uses:
`docs/gui-endpoints.json`, generated from the TypeScript descriptors by
`tools/gui-endpoints.mjs`. It names each endpoint, its transport, its
arguments (with their wire names and schemas) and its result schema. The
gateway checks a call against it before the call is forwarded, and the
sidecar's answers are checked against it on the way back, so a malformed
answer is refused rather than passed to a client.

## The protocol

### Framing

One request per frame, one answer per frame. A frame is a 4-byte big-endian
length followed by that many bytes of UTF-8 JSON. The length is the frame's
payload, not its header, and a frame longer than 8 MiB is a protocol error:
the plugin inventory and a bundle's detail are the largest answers, and they
are far below that bound. A frame that is not valid UTF-8, or not valid JSON,
or not a frame this protocol defines, is a protocol error.

The framing is length-prefixed, not newline-delimited: a plugin's output can
contain a newline, and a length prefix makes the reader's job independent of
the payload's bytes.

### Messages

```jsonc
// the Rust host → the sidecar
{ "v": 1, "id": "17", "kind": "call", "method": "pluginManager/listBundles", "args": { … } }
// the sidecar → the Rust host
{ "v": 1, "id": "17", "kind": "result", "value": { … } }
{ "v": 1, "id": "17", "kind": "failure", "error": { "code": "…", "message": "…", "details": { … } } }
```

- `v` is the protocol version. A frame with another version is refused, and so
  is a sidecar that answers with another version.
- `id` correlates the answer with the call. The host's ids are unique within
  a run.
- `kind` is `call`, `result` or `failure` for an ordinary endpoint.
- `method` is the endpoint name with its namespace, so the sidecar answers the
  endpoints the inventory names and nothing else, apart from the two protocol
  methods of the startup exchange (`hello` and `handshake`), which are not
  endpoints.
- `args` is the endpoint's argument object, after the gateway's own field and
  type checks. The sidecar receives the arguments a TypeScript client would
  send.
- The answer carries the endpoint's result value, or the endpoint's failure.
  The host passes a failure through unchanged, so a client sees the
  TypeScript error code, message and details.

### The duplex channel

`dynamicCordisRunner` has a *client half*: the plugin's browser-side code runs
in the user's browser and talks to the host, and the host half answers it. The
channel between them is the one place the bridge is duplex, and it is the one
place a second kind of message appears.

```jsonc
// the sidecar → the Rust host: a channel is offered
{ "v": 1, "kind": "channelOpen", "id": "c3", "endpoint": "dynamicCordisRunner/attachClient",
  "callId": "17", "context": "<opaque>", "args": { … } }
// the Rust host → the sidecar: the channel is accepted, or refused
{ "v": 1, "kind": "channelAccepted", "id": "c3", "grant": "<opaque>" }
{ "v": 1, "kind": "channelRefused", "id": "c3", "error": { "code": "…", "message": "…", "details": { … } } }
// either side → the other: a message on the channel
{ "v": 1, "kind": "channelMessage", "id": "c3", "value": { … } }
// either side → the other: the channel ends
{ "v": 1, "kind": "channelEnd", "id": "c3", "reason": "closed" | "cancelled" | "failed" }
```

- A channel is offered by the sidecar and accepted or refused by the host. A
  refused channel has an error the client can read, and no channel exists.
- The `context` is an opaque token the host minted for the plugin instance
  that is offering the channel, naming the browser connection the channel
  belongs to. The host resolves it: a channel whose context does not name a
  live connection, or names another plugin's, is refused. The channel's
  ownership is therefore established by the host, not claimed by the sidecar.
- The context reaches the sidecar with the **call** that starts the plugin
  instance: the host sends it in the `args` of the call, and the sidecar
  keeps it for the instance it starts. A sidecar that offers a channel with a
  context the host did not issue has one that is refused.
- The `callId` correlates the channel with the call that is waiting for it, so
  the host can answer that call when the channel ends.
- The messages on a channel keep their order, in both directions. The host's
  frames for one channel are written in order, and the sidecar's messages are
  read in order.
- A channel ends when either side sends `channelEnd`, when the host cancels
  it, when the browser connection it belongs to goes, or when the sidecar
  dies. The reason says which: `closed` (the plugin finished), `cancelled`
  (the host cancelled it) or `failed`.
- A channel that is not accepted leaves nothing behind: the host keeps no
  entry for it, and the plugin instance's budget is released with it.

### The exchange, end to end

The whole path, in the order it happens:

1. The host starts the sidecar with the inherited socket, and the sidecar
   answers `hello` with the token. The host accepts it.
2. The host asks for the handshake and learns the sidecar's protocol version
   and inventory digest.
3. The host calls a `dynamicCordisRunner` endpoint — `runHostHalf` — and its
   `args` carry the **context**: the token naming the browser connection and
   the plugin instance the call starts.
4. The sidecar starts the instance and offers a channel for it:
   `channelOpen` with the context it was given and the `callId` of the call
   that is waiting.
5. The host resolves the context, accepts the channel, and mints the
   **grant** for that instance. The `channelAccepted` frame carries it.
6. The sidecar passes the grant to the plugin instance, which uses it in the
   `request` frames it sends for a file. The host looks the grant up, checks
   the session and the path, and answers.
7. When the instance finishes, the sidecar sends `channelEnd`, the host
   revokes the grant, and the call that was waiting for the channel is
   answered.

Every step is defined: the context is delivered with the call, the grant
with the acceptance, and neither outlives the instance that owns it.

### The plugin's requests to the host

A plugin may need a file from the session that runs it. The host does not let
the sidecar read it directly: the sidecar has no session store, no file store
and no credential store, so a plugin's request is a callback the host answers
after its own checks.

```jsonc
// the sidecar → the Rust host: a plugin asks for a file
{ "v": 1, "kind": "request", "id": "r5", "method": "workspaceFiles/read", "args": { … },
  "grant": "<opaque>" }
// the Rust host → the sidecar
{ "v": 1, "kind": "result", "id": "r5", "value": { … } }
```

- The `grant` is not a bearer token the plugin may pass around: it is an
  entry in a table the **host** keeps, and it names the plugin instance, the
  session, the operations the plugin may ask for and the sidecar's
  generation. The host looks the entry up; it does not decode the token. The
  grant is minted by the host when it accepts the channel and is delivered to
  the sidecar in the `channelAccepted` frame, which is where the plugin
  instance learns the identity it must present.
- The host mints a grant when it accepts a channel, and it names in the grant
  only what the plugin's own session could do. A grant is revoked when the
  session ends, when the plugin instance is removed, when the sidecar's
  generation changes, and when the host stops. A revoked grant is refused
  even if the plugin still holds it.
- The session comes from the grant, never from the plugin's arguments: a
  plugin that names another session in its arguments is not believed.
- The path is checked the way the session's own client would: the path is
  resolved, and the resolved path must be inside the session's workspace
  after the links are followed. A symlink out of the workspace is refused.
- The host's answers carry the same codes as its own answers to a client, so
  a refusal reads the same.

### Resource limits

A sidecar hosts every plugin, so one runaway plugin must not take the rest
down. The limits are part of the protocol, and each one has a defined answer.

| limit | value | what happens when it is reached |
|---|---|---|
| concurrent calls | 32 | the call is answered `gateway/service-unavailable` |
| concurrent calls per plugin instance | 4 | that instance's call is answered `gateway/service-unavailable` |
| queued call bytes, per direction | 1 MiB | the call is answered `gateway/service-unavailable` |
| queued callback requests | 64 | the request is refused with `gateway/service-unavailable` |
| callback requests per second, per instance | 16 | the request is refused with `gateway/service-unavailable` |
| open channels | 64 | the channel is refused with `terminal/limit-reached` |
| channels per plugin instance | 8 | the channel is refused with `terminal/limit-reached` |
| a channel's queued bytes, per direction | 256 KiB | the sender waits; it is not dropped |
| a call's deadline | 60 s | the call is answered `gateway/internal` with `the sidecar did not answer in time` |
| the sidecar's heap | 512 MiB | the sidecar is ended by the host |

- The queues are bounded **per direction and per plugin instance**, so one
  plugin cannot spend the whole budget. A channel's sender waits when its
  budget is spent: the bytes are not dropped, they are not sent yet.
- The control frames — `cancel`, `channelEnd`, the handshake — are served
  before the ordinary ones, so a cancellation is never stuck behind a flood.
- The host's own accounting is per plugin instance, so one instance's flood
  does not stop another's calls.

**What the limits do not do.** The sidecar is one Node process: its event loop,
its heap and its CPU are shared. A plugin that blocks the event loop blocks
every other plugin, and a plugin that allocates outside the V8 heap can
exhaust the process. The TypeScript sandbox's own limits (its execution
timeout, its memory ceiling) bound what a plugin can do *inside* the sandbox,
and this bridge bounds what one plugin can ask of the host; neither makes the
process immune to a plugin that exhausts it. **A shared-process denial of
service is possible, and a deployment that needs stronger isolation must run
one sidecar per plugin** — the bridge is written for that, because the grant,
the budgets and the generation are per instance.

## The trust boundary

Two different things are separated here, and they have different guarantees.

**The plugin code** runs inside the TypeScript sandbox
(`packages/extensions/cordis-host-runner/src/sandbox.ts`), which is what the
TypeScript host relies on: the code is evaluated in a VM with the guards of
`guard.ts` in front of it. The plugin code never runs in the Rust process, and
the Rust process never evaluates plugin source. That is the guarantee the
bridge adds: the Rust host's memory, its session store and its credentials are
not the plugin's to touch.

**The sidecar process** runs as the operator, with the operator's privileges.
It can read the files the operator can read, and it is not a sandbox. That is
a deliberate choice: the sandbox is the TypeScript package's own, and a second
sandbox around the sandbox would be a new thing to get wrong. The consequence
is stated plainly: **a sandbox escape in the sidecar is an escape with the
operator's privileges**, not a confinement. A deployment that needs a stronger
boundary runs the sidecar under a different user, in a container, or with the
file permissions the plugins need removed — the sidecar's own configuration,
not the bridge's.

What the bridge does guarantee:

- The plugin code does not run in the Rust process.
- The sidecar is given no store: it has no session store, no file store and
  no credential store, so a plugin's file request is a callback the host
  answers after its own checks.
- The Rust host does not trust the sidecar's answers: they are validated
  against the inventory before a client sees them.

### Reaching the socket

The transport is an **inherited connected socket**: the host creates a
`socketpair`, keeps one end, and passes the other to the sidecar as file
descriptor 3 when it starts the process. There is no listener, no port and no
connection opportunity, so another process on the machine has nothing to
connect to.

The connection is authenticated anyway, because the socket's other end could
be inherited by a process that did not start the sidecar. The startup
exchange is:

```jsonc
// the sidecar → the Rust host: the first frame carries the token
{ "v": 1, "kind": "hello", "token": "<32 bytes, base64url>" }
// the Rust host → the sidecar: the token is accepted
{ "v": 1, "kind": "accepted" }
// the Rust host → the sidecar: the host asks what the sidecar is
{ "v": 1, "kind": "handshake" }
// the sidecar → the Rust host: what the sidecar is
{ "v": 1, "kind": "handshake", "protocol": 1, "inventory": "<digest>", "packages": { "<name>": "<version>" } }
```

- The token is 32 bytes from the host's CSPRNG, carried in the first frame
  and compared in constant time. A frame with the wrong token, or a first
  frame that is not `hello`, is answered with a protocol error and the
  connection is closed.
- The later frames do not carry the token: the connection is already
  authenticated, and the token is not repeated where a log would keep it.
- The handshake reports the sidecar's protocol version, the digest of the
  inventory it was started with, and the versions of the packages it loaded.
  The host refuses a sidecar whose protocol version or inventory digest does
  not match its own: a checkout with different endpoints answers questions
  the host cannot check.
- `hello`, `accepted` and `handshake` are the protocol's own methods, not
  endpoints; the inventory's endpoints are the rest.

## The lifecycle

- The `gui` plugin starts the sidecar when `gui.sidecar.path` names a
  checkout. It waits for the handshake (below) with a deadline of 30 s. A
  sidecar that does not answer in time is killed, and the host starts with the
  sidecar's endpoints answering `gateway/service-unavailable` — the same
  answer a host without a sidecar gives. The sidecar never blocks the host's
  start.
- The **handshake** is the first exchange: the sidecar answers `handshake`
  with its protocol version, the digest of the inventory it was started with,
  and the versions of the packages it loaded. The host refuses a sidecar whose
  protocol version or inventory digest does not match: a checkout with
  different endpoints answers questions the host cannot check.
- The sidecar is stopped with the host: the host closes the socket, the
  sidecar's pending calls are cancelled, and the Node process is asked to
  exit. A sidecar that does not exit within 10 s is killed, and its process
  tree with it — the same rule the terminal uses for a shell.
- A sidecar that dies is not restarted in place. Its endpoints answer
  `gateway/service-unavailable` until the host is restarted, because a plugin
  registry rebuilt from scratch is a different registry, and a client that
  believed it was the same one would be misled.
- A call that is in flight when the sidecar dies is answered
  `gateway/internal` with `the sidecar ended`. A mutating call whose answer
  was lost may or may not have taken effect: the registry is the TypeScript
  package's own durable store, so the next `pluginManager/listBundles` says
  what is there. The host does not retry a mutating call on its own.
- **A timeout is a cancellation, and its outcome is uncertain.** When a call
  passes its deadline the host sends `{ "kind": "cancel", "id": … }` and waits
  for the sidecar's `{ "kind": "cancelled", "id": … }`. The sidecar stops the
  work it can stop and acknowledges; work it cannot stop keeps running, and
  the host keeps the call's slot and its budget until the sidecar says the
  work ended. A result that arrives after the timeout is dropped. A mutating
  call that timed out may or may not have taken effect, exactly as a mutating
  call whose sidecar died may or may not have taken effect: the registry
  says what is there.
- **One recovery policy for every exit cause.** The sidecar is never
  restarted in place, whatever ended it — its exit, its heap, a crash. Its
  endpoints answer `gateway/service-unavailable` until the host restarts,
  because a plugin registry rebuilt from scratch is a different registry, and
  a client that believed it was the same one would be misled. A restart would
  also invalidate the grants, the channels and the pending calls, which is
  why it is not done. The heap limit ends the sidecar; it does not replace
  it.
- The sidecar's output is drained into the host's log, bounded: at most 64 KiB
  a second, at most 1 MiB in the host's buffer for the sidecar, with the
  plugin's name on every line and control characters removed. A plugin that
  floods the log is truncated and the truncation is reported, not hidden.

## The Node version and the checkout

The TypeScript packages need Node 24.9 or newer, and the harness's own
commands run with that Node. The sidecar runs with the Node the operator's
`PATH` names, and the host refuses to start one that is too old: a sidecar
that cannot load the packages would fail every call with a message that says
nothing about the cause.

The path to the TypeScript checkout is configuration, never a relative path
and never a guess:

```jsonc
// in the `gui` plugin's configuration
{ "sidecar": { "path": "/Users/rachit/code/deepseek-harness", "node": "/opt/homebrew/bin/node" } }
```

- The checkout must have its packages installed (`pnpm install`) and built
  (`pnpm run build:artifacts`), because the sidecar loads the packages from
  it. The host does not build them: a build is the operator's step, and a
  build that fails should not be hidden inside a host start.
- The entry point is a small script the checkout provides
  (`packages/extensions/cordis-sidecar/bin/serve.ts`), which boots the
  packages on their own services and answers the bridge protocol.
- The sidecar inherits the host's environment only where the configuration
  says so. It gets no session token, no credential and no model key: the
  packages it hosts do not need them, and a plugin must not find them.

## What stays in Rust

Everything that is a *host* concern stays in the Rust host: the sessions, the
agent loop, the tools, the files, the schedule, the terminal, the account's
own state, the gateway and the Remote stream protocol. The sidecar is for the
subsystems that are TypeScript packages with their own protocols, and for
nothing else.

The rule: an endpoint whose contract is a *wire format* — a schema in the
inventory, a request and a response — belongs in Rust, because the Rust host
already speaks it. An endpoint whose contract is a *package* — a plugin
registry, a bundle format, a sandbox — belongs in the sidecar, because the
package is the implementation. `terminal/create` is the first kind (its
contract is the inventory's schema) and `pluginManager/installBundle` is the
second (it is the package's installer). The whole of `pluginManager` is the
second kind: its fifteen endpoints are the package's own remote methods, so
the sidecar answers all fifteen and Rust forwards them. The endpoints Rust
serves itself are the ones whose contract the inventory already fixes.

## The sequence

1. The sidecar process: the framing, the handshake, the token, the limits, the
   lifecycle, the version check. It answers one `handshake` method, so the
   bridge can be tested on its own.
2. `pluginManager` (15 endpoints): the endpoints are the package's remote
   methods, so the sidecar answers all fifteen and the Rust host forwards
   them.
3. `dynamicCordisRunner` (10 endpoints): the same bridge, plus the duplex
   channel and the plugin's callbacks.
4. `pluginMarket`, `pluginRecommendations`, `actionCards`, `cotTranslation`:
   the same bridge, each a package.

## Open findings

The fourth EGM review (run `cfd873f3c6ea`) left eight findings open. They are
recorded here with the answer each one gets, so the note stays the contract
the slice is built from.

1. **How the sidecar obtains the token.** It is passed in its environment: the
   host sets `DSH_SIDECAR_TOKEN` when it starts the process, and the sidecar
   reads it and sends it in `hello`. The environment is the same channel the
   inherited socket uses, so nothing new is opened.
2. **Where the context lives in a call.** The host adds it under a reserved
   argument name, `__context`, which the inventory's descriptor does not
   declare — so the sidecar strips it before the call reaches the package, and
   the package never sees it.
3. **What makes a grant non-transferable.** The host's table holds the grant,
   and the request carries the *instance* the grant belongs to as well. The
   host looks up the grant, checks that it names that instance, and that the
   instance is the one the request's channel belongs to. A grant another
   instance presents is refused.
4. **The duplex exchange against the real endpoints.** `runHostHalf` is
   unary and returns when the host half is running, not when the plugin ends;
   the client half is attached by `dynamicCordisRunner/attachClient`, which
   the inventory declares as a stream. The channel therefore rides that
   endpoint's stream, and the note's `channelOpen` message is the sidecar's
   internal way of describing it. The note's example is wrong and the
   implementation follows the inventory.
5. **The host integrations the runner needs.** The runner takes the `Agent` it
   drives as a lookup parameter, so the host resolves it and passes the
   agent's identity, not the agent: the sidecar asks the host to steer or
   inject through a callback, and the host does it with the agent it holds. The
   sidecar never holds an agent.
6. **Backpressure without blocking the other channels.** The channel's flow
   control is a credit count, not a pause: the host grants a channel a number
   of message bytes it may have in flight and the sidecar may not send more
   until the host grants more. A channel that runs out of credit waits, and
   the other channels keep their own budgets.
7. **The cancellation deadline.** A cancelled call has ten seconds to be
   acknowledged. A call that does not acknowledge keeps its slot, and the
   host's answer says so, because the work is still running; the slot is
   released when the sidecar says the work ended or when the sidecar dies.
8. **The sidecar's own service dependencies.** The packages need the host's
   services (the agent registry, the file store, the tools). The sidecar asks
   the host for them through the callback channel, so the sidecar hosts no
   store of its own — which is the boundary the note claims, and the reason
   the packages' own `inject` lists are satisfied by proxies.
