#!/usr/bin/env node
// Export the official GUI's remote endpoints as JSON, so the Rust host can be
// diffed against them on every upstream tag instead of drifting silently.
//
// Every generated package publishes `lib/typert.remote-client.js`, which
// describes each endpoint: its transport `mode` (stream or unary), the JSON
// Schema of each parameter, and the JSON Schema of the result. Reading those
// descriptors is the only measured source for what the GUI can call.
//
// Usage: node tools/gui-endpoints.mjs [--root <dsh checkout>] [--out <file>]
// Writes a stable, sorted JSON document; identical input gives identical bytes.

import { readdirSync, existsSync, writeFileSync, statSync } from 'node:fs'
import { createHash } from 'node:crypto'
import { join, relative } from 'node:path'
import { pathToFileURL } from 'node:url'

function parseArgs(argv) {
  const opts = { root: '.', out: null }
  for (let i = 0; i < argv.length; i += 1) {
    if (argv[i] === '--root') opts.root = argv[++i]
    else if (argv[i] === '--out') opts.out = argv[++i]
  }
  return opts
}

/** Every `packages/<group>/<pkg>/lib/typert.remote-client.js`, sorted. */
function descriptorFiles(root) {
  const files = []
  const groups = join(root, 'packages')
  if (!existsSync(groups)) throw new Error(`not a DSH checkout: ${groups} is missing`)
  for (const group of readdirSync(groups, { withFileTypes: true })) {
    if (!group.isDirectory() || group.name.startsWith('.')) continue
    const groupPath = join(groups, group.name)
    for (const pkg of readdirSync(groupPath, { withFileTypes: true })) {
      if (!pkg.isDirectory()) continue
      const file = join(groupPath, pkg.name, 'lib', 'typert.remote-client.js')
      if (existsSync(file)) files.push(file)
    }
  }
  return files.sort()
}

/**
 * Transport for one endpoint. `mode: 'stream'` on the descriptor is the only
 * stream signal: a unary endpoint's `result.mode` is its codec strictness
 * (`strict`), not a transport, and `invocation` is `direct` for every endpoint.
 */
function transportOf(method) {
  return (method.mode ?? method.result?.mode) === 'stream' ? 'stream' : 'unary'
}

const opts = parseArgs(process.argv.slice(2))
const files = descriptorFiles(opts.root)
const endpoints = []
const digest = createHash('sha256')
let imported = 0

for (const file of files) {
  digest.update(relative(opts.root, file)).update('\0')
  const module = await import(pathToFileURL(join(process.cwd(), file)).href)
  const info = module.TYPERT_REMOTE ?? module.default
  const methods = info?.methods ?? {}
  const pkg = relative(opts.root, file).split('/lib/')[0]
  imported += 1
  for (const [name, method] of Object.entries(methods)) {
    endpoints.push({
      endpoint: name,
      package: pkg,
      transport: transportOf(method),
      service: method.service ?? null,
      description: method.description ?? null,
      parameters: (method.parameters ?? []).map((parameter) => ({
        name: parameter.name,
        source: parameter.source ?? null,
        optional: parameter.acceptsUndefined === true,
        schema: parameter.codec?.jsonSchema ?? null,
      })),
      cancellation: method.cancellation?.parameter ?? null,
      resultSchema: method.result?.jsonSchema ?? null,
    })
  }
}

endpoints.sort((a, b) => (a.endpoint < b.endpoint ? -1 : a.endpoint > b.endpoint ? 1 : 0))
const duplicates = endpoints.filter((e, i) => i > 0 && endpoints[i - 1].endpoint === e.endpoint)

const doc = {
  source: 'packages/*/*/lib/typert.remote-client.js',
  descriptorFiles: imported,
  descriptorDigest: digest.digest('hex'),
  endpoints,
}
const text = `${JSON.stringify(doc, null, 2)}\n`
if (opts.out) writeFileSync(opts.out, text)
else process.stdout.write(text)

const byTransport = endpoints.reduce((acc, e) => ({ ...acc, [e.transport]: (acc[e.transport] ?? 0) + 1 }), {})
process.stderr.write(
  `exported ${endpoints.length} endpoints from ${imported} descriptor files ${JSON.stringify(byTransport)}` +
    `${duplicates.length ? ` WARNING duplicate names: ${duplicates.map((d) => d.endpoint).join(', ')}` : ''}\n`,
)
