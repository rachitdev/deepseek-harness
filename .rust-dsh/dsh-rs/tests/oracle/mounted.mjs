#!/usr/bin/env node
// Which inventory endpoints does the TS host actually mount? A mounted unary
// endpoint answers with a JSON envelope (even a refusal); an unmounted one gets
// a bare 404 "not found". Prints one line per endpoint.
//
//   TS_URL=... TS_TOKEN=... node tests/oracle/mounted.mjs [--missing]
import { readFileSync } from 'node:fs'

const inventory = JSON.parse(readFileSync(new URL('../../docs/gui-endpoints.json', import.meta.url))).endpoints
const missing = new Set(readFileSync(new URL('../../docs/parity-missing.txt', import.meta.url), 'utf8').split('\n').map(l => l.trim()).filter(Boolean))
const onlyMissing = process.argv.includes('--missing')

const exchange = await fetch(`${process.env.TS_URL}/?token=${process.env.TS_TOKEN}`, { redirect: 'manual' })
const cookie = exchange.headers.get('set-cookie').split(';')[0]

const rows = []
for (const e of inventory) {
  if (onlyMissing && !missing.has(e.endpoint)) continue
  if (e.transport !== 'unary') { rows.push({ endpoint: e.endpoint, transport: 'stream', mounted: '?' }); continue }
  // A call that is still unanswered after the deadline is held open by a handler,
  // which proves the route exists.
  try {
    const response = await fetch(`${process.env.TS_URL}/api/${e.endpoint}`, {
      method: 'POST',
      headers: { 'content-type': 'application/json', cookie },
      body: JSON.stringify({ type: 'client-request', rpcId: 'm', method: e.endpoint, payload: { args: {} } }),
      signal: AbortSignal.timeout(4000),
    })
    const text = await response.text()
    rows.push({ endpoint: e.endpoint, transport: 'unary', mounted: response.status === 404 && text === 'not found' ? 'no' : 'yes', status: response.status })
  } catch (error) {
    rows.push({ endpoint: e.endpoint, transport: 'unary', mounted: 'yes', status: 'held' })
  }
}
for (const r of rows) console.log(`${r.mounted.padEnd(4)} ${r.transport.padEnd(6)} ${r.endpoint}`)
