#!/usr/bin/env node
// Differential harness for the job streams: `job/list` and `job/follow`.
//
//   TS_URL=... TS_TOKEN=... RS_URL=... RS_TOKEN=... TS_S1=... RS_S1=... \
//     node tests/oracle/job_stream.mjs
//
// This build tracks no background jobs, so the rows are empty and every job is
// unknown; the frames must say exactly that.
const hosts = {
  ts: { url: process.env.TS_URL, token: process.env.TS_TOKEN, session: process.env.TS_S1 },
  rs: { url: process.env.RS_URL, token: process.env.RS_TOKEN, session: process.env.RS_S1 },
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

const sleep = ms => new Promise(resolve => setTimeout(resolve, ms))

async function openSocket(host) {
  const { url } = hosts[host]
  const headers = await headersFor(host)
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
  return { frames, send: message => socket.send(JSON.stringify(message)), close: () => socket.close() }
}

/// The frames of one stream, in the shape the comparison uses. The object keys
/// are sorted, so the comparison does not depend on the order the host wrote
/// them in.
function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical)
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])]))
  }
  return value
}

/// The frames of one stream, in the shape the comparison uses.
function summary(frames, streamId) {
  const out = []
  for (const frame of frames) {
    if (frame.streamId !== streamId) continue
    if (frame.type === 'item') out.push(canonical(frame.value))
    else if (frame.type === 'error') out.push(canonical({ error: frame.error }))
    else out.push(frame.type)
  }
  return out
}

async function scenario(host) {
  const { session } = hosts[host]
  const socket = await openSocket(host)
  socket.send({ type: 'open', streamId: 'j1', endpoint: 'job/list', payload: { args: { request: { sessionId: session } } } })
  socket.send({ type: 'open', streamId: 'j2', endpoint: 'job/follow', payload: { args: { request: { sessionId: session, jobId: 'nope' } } } })
  socket.send({ type: 'open', streamId: 'j3', endpoint: 'job/follow', payload: { args: { request: { sessionId: session } } } })
  socket.send({ type: 'open', streamId: 'j4', endpoint: 'job/list', payload: { args: { request: { sessionId: 'ghost' } } } })
  await sleep(1500)
  const result = {
    list: summary(socket.frames, 'j1'),
    follow: summary(socket.frames, 'j2'),
    followWithoutJob: summary(socket.frames, 'j3'),
    listGhost: summary(socket.frames, 'j4'),
  }
  socket.close()
  return result
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
