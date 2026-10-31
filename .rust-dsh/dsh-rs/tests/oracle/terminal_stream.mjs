#!/usr/bin/env node
// Differential harness for the terminal streams: `terminal/attach` and
// `terminal/retain`.
//
//   TS_URL=... TS_TOKEN=... RS_URL=... RS_TOKEN=... TS_S2=... RS_S2=... \
//     node tests/oracle/terminal_stream.mjs
//
// The streams use the second session: the unary case fills the first one with
// terminals, and a session holds only eight.
//
// One socket per host carries every stream, as a browser would: the Rust host
// answers a second upgrade on the same port with 409.
//
// The two hosts run the same shell, so the screen text and the number of output
// frames depend on timing. The comparison is therefore over the frame
// *sequence*: the frame kinds, the acknowledgement sequences, the refusals and
// the failure codes, with consecutive output frames collapsed into one entry.
const hosts = {
  ts: { url: process.env.TS_URL, token: process.env.TS_TOKEN, session: process.env.TS_S2 },
  rs: { url: process.env.RS_URL, token: process.env.RS_TOKEN, session: process.env.RS_S2 },
}

let counter = 0
const cookies = {}

async function headersFor(host) {
  const { url, token } = hosts[host]
  const headers = { 'content-type': 'application/json' }
  if (host === 'rs') return { ...headers, authorization: `Bearer ${token}` }
  if (cookies[host] === undefined) {
    const exchange = await fetch(`${url}/?token=${token}`, { redirect: 'manual' })
    const set = exchange.headers.get('set-cookie')
    if (!set) throw new Error(`${host}: token exchange gave no cookie (${exchange.status})`)
    cookies[host] = set.split(';')[0]
  }
  return { ...headers, cookie: cookies[host] }
}

async function call(host, method, args) {
  const { url } = hosts[host]
  const response = await fetch(`${url}/api/${method}`, {
    method: 'POST',
    headers: await headersFor(host),
    body: JSON.stringify({ type: 'client-request', rpcId: `s-${++counter}`, method, payload: { args } }),
  })
  const envelope = await response.json()
  return envelope.result ?? envelope
}

const sleep = ms => new Promise(resolve => setTimeout(resolve, ms))

/// A socket that records the frames it receives.
async function openSocket(host) {
  const { url } = hosts[host]
  const headers = await headersFor(host)
  // The TypeScript host takes its session cookie; the Rust host takes the
  // bearer token. Both are sent the way a browser would.
  const handshake = host === 'rs'
    ? { authorization: headers.authorization, origin: url }
    : { cookie: headers.cookie, origin: url }
  const socket = new WebSocket(`${url.replace('http', 'ws')}/api/remote.mux`, { headers: handshake })
  const frames = []
  socket.onmessage = event => frames.push(JSON.parse(event.data))
  await new Promise((resolve, reject) => {
    socket.onopen = resolve
    socket.onerror = () => reject(new Error(`${host}: the mux socket did not open`))
  })
  return {
    frames,
    send: message => socket.send(JSON.stringify(message)),
    close: () => socket.close(),
  }
}

/// What one stream carried: the screen its snapshot held and the text its
/// output frames carried, with the parts that depend on timing removed.
function contentOf(frames, streamId) {
  let screen = ''
  let output = ''
  let sequences = []
  for (const frame of frames) {
    if (frame.streamId !== streamId || frame.type !== 'item') continue
    const value = frame.value ?? {}
    if (value.type === 'snapshot') screen = value.screen ?? ''
    if (value.type === 'output') { output += value.data ?? ''; sequences.push(value.sequence) }
  }
  // The screen holds the prompt; the output holds the command and its answer.
  const prompt = /bash-[\d.]+[$#]|zsh|%/.test(screen)
  const echoed = output.includes('ready')
  // The sequence numbers increase by one.
  const increasing = sequences.every((value, index) => index === 0 || value === sequences[index - 1] + 1)
  return { screenNotEmpty: screen.length > 0, prompt, echoed, increasing, outputHasEcho: output.includes('echo ready') }
}

/// The frame sequence of one stream, with the timing-dependent parts collapsed.
function summary(frames, streamId) {
  const out = []
  for (const frame of frames) {
    if (frame.streamId !== streamId) continue
    if (frame.type === 'item') {
      const value = frame.value ?? {}
      if (value.type === 'output') {
        if (out[out.length - 1] === 'output') continue
        out.push('output')
      } else if (value.type === 'snapshot') {
        out.push('snapshot')
      } else if (value.type === 'ack') {
        out.push(value.refusal ? `ack:${value.sequence}:${value.refusal}` : `ack:${value.sequence}`)
      } else if (value.type === 'state') {
        out.push(`state:${value.writable ? 'writable' : 'read-only'}`)
      } else {
        out.push(value.type)
      }
    } else if (frame.type === 'error') {
      out.push(`error:${frame.error.code}:${frame.error.message}`)
    } else if (frame.type === 'end') {
      out.push('end')
    }
  }
  return out
}

async function scenario(host) {
  const { session } = hosts[host]
  // A fresh identity: a closed terminal may not be created again in its session.
  const id = `st-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`
  const created = await call(host, 'terminal/create', { agentId: session, request: { id, cols: 80, rows: 24 } })
  if (created.ok === false) throw new Error(`${host}: create failed: ${JSON.stringify(created)}`)

  const socket = await openSocket(host)
  const open = (streamId, endpoint, args) => socket.send({ type: 'open', streamId, endpoint, payload: { args } })
  const item = (streamId, value) => socket.send({ type: 'item', streamId, value })

  // The shell's own start-up output arrives at its own pace; the attachment
  // waits for it, so the two hosts are compared from the same state.
  await sleep(4000)
  open('a1', 'terminal/attach', { agentId: session, id })
  await sleep(800)
  item('a1', { type: 'write', sequence: 1, data: 'echo ready\n' })
  await sleep(900)
  item('a1', { type: 'resize', sequence: 2, cols: 100, rows: 30 })
  await sleep(600)
  const firstFrames = summary(socket.frames, 'a1')
  // The frame kinds are only half the story: the data the client receives is
  // compared too, so an implementation that emits the right kinds with the
  // wrong bytes fails here.
  const content = contentOf(socket.frames, 'a1')

  // A second attachment takes the input; the first becomes read-only and is
  // told so, and its own input is refused rather than answered.
  open('a2', 'terminal/attach', { agentId: session, id })
  await sleep(700)
  item('a1', { type: 'write', sequence: 3, data: 'y' })
  await sleep(600)
  const afterTakeover = summary(socket.frames, 'a1')
  const secondFrames = summary(socket.frames, 'a2')

  // A gap in the client's sequence ends that attachment.
  open('a3', 'terminal/attach', { agentId: session, id })
  await sleep(500)
  item('a3', { type: 'write', sequence: 4, data: 'x' })
  await sleep(600)
  const gapFrames = summary(socket.frames, 'a3')

  // Retention, and the refusals.
  open('r1', 'terminal/retain', { sessionId: session, id })
  open('r2', 'terminal/retain', { sessionId: session, id: 'nope' })
  open('r3', 'terminal/attach', { agentId: session, id: 'nope' })
  open('r4', 'terminal/attach', { agentId: 'ghost', id })
  open('r5', 'terminal/attach', { agentId: session })
  await sleep(900)
  const retainFrames = summary(socket.frames, 'r1')
  const retainMiss = summary(socket.frames, 'r2')
  const attachMiss = summary(socket.frames, 'r3')
  const agentMiss = summary(socket.frames, 'r4')
  const badArgs = summary(socket.frames, 'r5')

  socket.close()
  await call(host, 'terminal/close', { agentId: session, id })
  return { firstFrames, afterTakeover, secondFrames, gapFrames, retainFrames, retainMiss, attachMiss, agentMiss, badArgs, content }
}

const ts = await scenario('ts')
const rs = await scenario('rs')
const diffs = []
for (const key of Object.keys(ts)) {
  const a = JSON.stringify(ts[key])
  const b = JSON.stringify(rs[key])
  if (a !== b) diffs.push(`${key}\n  ts: ${a}\n  rs: ${b}`)
}
console.log(JSON.stringify({ compared: Object.keys(ts).length, differing: diffs.length }))
for (const diff of diffs) console.log('DIFF', diff)
process.exit(diffs.length === 0 ? 0 : 1)
