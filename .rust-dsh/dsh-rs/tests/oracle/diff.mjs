#!/usr/bin/env node
// Differential harness: send one request sequence to the TS host and the RS
// host and compare the replies, ignoring object key order.
//
//   TS_URL=http://127.0.0.1:3090 TS_TOKEN=... RS_URL=http://127.0.0.1:3081 RS_TOKEN=... \
//     node tests/oracle/diff.mjs tests/oracle/cases/<slice>.mjs
//
// A case file default-exports `async ({ call, both, scrub }) => void`:
//   call(host, method, args)  one unary call on `ts` or `rs`; returns the `result` envelope
//   both(method, args)        the same call on both; records a diff when they disagree
//   scrub(fn)                 register a value normaliser applied to both replies
// Ids and times differ per host; scrubbers map them to stable tokens.

const hosts = {
  ts: { url: process.env.TS_URL, token: process.env.TS_TOKEN },
  rs: { url: process.env.RS_URL, token: process.env.RS_TOKEN },
}

let counter = 0
const cookies = {}

// The TS host swaps the launch token in `?token=` for a signed cookie; the RS
// host takes a bearer token. Both are sent the way a browser would.
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

export async function call(host, method, args) {
  const { url } = hosts[host]
  const response = await fetch(`${url}/api/${method}`, {
    method: 'POST',
    headers: await headersFor(host),
    body: JSON.stringify({ type: 'client-request', rpcId: `d-${++counter}`, method, payload: { args } }),
  })
  const contentType = response.headers.get('content-type') ?? ''
  if (contentType.startsWith('multipart/form-data')) {
    // A binary answer: the parts, and the metadata that names them.
    const body = Buffer.from(await response.arrayBuffer())
    const boundary = contentType.split('boundary=')[1]
    const parts = []
    let metadata = null
    for (const chunk of body.toString('latin1').split(`--${boundary}`)) {
      const split = chunk.indexOf('\r\n\r\n')
      if (split < 0) continue
      const headers = chunk.slice(0, split)
      const content = chunk.slice(split + 4).replace(/\r\n$/, '')
      const name = /name="([^"]+)"/.exec(headers)?.[1]
      if (name === 'metadata') metadata = JSON.parse(content)
      else if (name) parts.push({ name, bytes: Buffer.from(content, 'latin1').length, digest: digest(Buffer.from(content, 'latin1')) })
    }
    return { multipart: true, metadata: metadata?.result ?? metadata, parts }
  }
  const envelope = await response.json()
  return envelope.result ?? envelope
}

/// A short digest of some bytes, so two hosts' binaries can be compared.
function digest(bytes) {
  let hash = 0
  for (const byte of bytes) hash = (hash * 31 + byte) >>> 0
  return hash.toString(16)
}

function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical)
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.keys(value).sort().map(k => [k, canonical(value[k])]))
  }
  return value
}

const scrubbers = []
const diffs = []
let compared = 0

function normalise(value) {
  let out = value
  for (const fn of scrubbers) out = fn(out)
  return canonical(out)
}

export async function both(method, args, label = '') {
  const [ts, rs] = await Promise.all([call('ts', method, args), call('rs', method, args)])
  compared++
  const a = JSON.stringify(normalise(ts))
  const b = JSON.stringify(normalise(rs))
  if (a !== b) diffs.push({ method, label, args, ts, rs })
  return { ts, rs }
}

const file = process.argv[2]
if (!file) throw new Error('usage: diff.mjs <cases file>')
const run = (await import(new URL(file, `file://${process.cwd()}/`))).default
await run({ call, both, scrub: fn => scrubbers.push(fn), hosts })
console.log(JSON.stringify({ compared, differing: diffs.length }))
for (const d of diffs) {
  console.log(`--- ${d.method} ${d.label}\n  args: ${JSON.stringify(d.args)}\n  ts:   ${JSON.stringify(d.ts)}\n  rs:   ${JSON.stringify(d.rs)}`)
}
process.exit(diffs.length === 0 ? 0 : 1)
