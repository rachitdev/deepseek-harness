// Boundary validation: for every endpoint both hosts serve, send the minimal
// valid arguments and then each field with a wrong type, and compare only
// whether the call is refused AT THE GATEWAY (`gateway/input-invalid`,
// `gateway/arguments-invalid`) and with which field. Handler outcomes are
// ignored: the hosts hold different sessions.
import { readFileSync } from 'node:fs'

const inventory = JSON.parse(readFileSync(new URL('../../../docs/gui-endpoints.json', import.meta.url))).endpoints

const sample = schema => {
  if (!schema || typeof schema !== 'object') return 'x'
  if (schema.oneOf) return sample(schema.oneOf[0])
  if ('const' in schema) return schema.const
  if (schema.enum) return schema.enum[0]
  switch (schema.type) {
    case 'number': return 0
    case 'boolean': return false
    case 'null': return null
    case 'array': return []
    case 'object': {
      const out = {}
      for (const k of schema.required ?? []) out[k] = sample(schema.properties?.[k])
      return out
    }
    default: return 'x'
  }
}
const wires = e => e.parameters.map(p => ({ ...p, wire: p.source === 'lookup' ? `${p.name}Id` : p.name }))
const minimal = e => Object.fromEntries(wires(e).filter(p => !p.optional).map(p => [p.wire, p.source === 'lookup' ? 'ghost' : sample(p.schema)]))

const gatewayLevel = r => {
  const code = r?.error?.code
  return code === 'gateway/input-invalid' || code === 'gateway/arguments-invalid' ? `${code}:${r.error.details?.field ?? ''}` : 'handler'
}

export default async ({ call, hosts }) => {
  const served = new Set(JSON.parse(await (await fetch(`${hosts.rs.url}/api/endpoints`, { headers: { authorization: `Bearer ${hosts.rs.token}` } })).text()).endpoints)
  // The TypeScript Web host mounts only the production packages, so an
  // experimental endpoint is not on it at all: the comparison skips those.
  const exchange = await fetch(`${hosts.ts.url}/?token=${hosts.ts.token}`, { redirect: 'manual' })
  const cookie = exchange.headers.get('set-cookie').split(';')[0]
  const mounted = new Set()
  let cases = 0
  const diffs = []
  for (const e of inventory) {
    if (e.transport !== 'unary' || !served.has(e.endpoint)) continue
    if (!mounted.has(e.endpoint)) {
      // The probe carries an unexpected field, so the gateway refuses it at
      // the boundary and no handler runs: the answer says whether the
      // TypeScript host mounts the endpoint at all.
      const answer = await fetch(`${hosts.ts.url}/api/${e.endpoint}`, {
        method: 'POST',
        headers: { 'content-type': 'application/json', cookie },
        body: JSON.stringify({ type: 'client-request', rpcId: 'probe', method: e.endpoint, payload: { args: { zzExtra: 1 } } }),
      })
      if (!answer.headers.get('content-type')?.includes('application/json')) continue
      mounted.add(e.endpoint)
    }
    // Endpoints that start work or touch the desktop are not poked with junk.
    if (/^(session\/(prompt|create|fork|cancel|openWorkspacePath)|workspace\/(create|delete)|fileUploads|session\/updateQueue|llm\/discoverModels|directoryPicker\/(createDirectory|pick)|credentials\/(set|unset))/.test(e.endpoint)) continue
    const base = minimal(e)
    const variants = [['valid', base]]
    for (const p of wires(e)) {
      if (p.source === 'lookup') continue
      for (const [label, junk] of [['number', 7], ['string', 'junk'], ['null', null], ['array', []], ['object', {}], ['bool', true]]) {
        variants.push([`${p.wire}=${label}`, { ...base, [p.wire]: junk }])
      }
    }
    variants.push(['extra', { ...base, zzExtra: 1 }])
    for (const p of wires(e).filter(p => !p.optional)) {
      const { [p.wire]: _, ...rest } = base
      variants.push([`missing ${p.wire}`, rest])
    }
    for (const [label, args] of variants) {
      const [ts, rs] = await Promise.all([call('ts', e.endpoint, args), call('rs', e.endpoint, args)])
      cases++
      const a = gatewayLevel(ts); const b = gatewayLevel(rs)
      if (a !== b) diffs.push(`${e.endpoint} [${label}] ts=${a} rs=${b}`)
    }
  }
  console.log(`cases ${cases}, differing ${diffs.length}`)
  for (const d of diffs) console.log('DIFF', d)
}
