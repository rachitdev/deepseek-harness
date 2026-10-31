// The workspace files' scope refusals.
//
// The scope is a lookup the gateway resolves, so an identity it cannot resolve
// is the gateway's own refusal; the file's own refusals follow it.
const env = process.env
const sessions = { ts: env.TS_S1, rs: env.RS_S1 }

export default async ({ call }) => {
  let cases = 0
  const diffs = []
  const canon = v => Array.isArray(v) ? v.map(canon) : v && typeof v === 'object'
    ? Object.fromEntries(Object.keys(v).sort().map(k => [k, canon(v[k])])) : v
  const mask = (v, h) => JSON.parse(JSON.stringify(v)
    .replaceAll(sessions[h], '<S1>'))
  const check = async (method, build, label) => {
    const [ts, rs] = await Promise.all([call('ts', method, build('ts')), call('rs', method, build('rs'))])
    cases++
    const a = JSON.stringify(canon(mask(ts, 'ts')))
    const b = JSON.stringify(canon(mask(rs, 'rs')))
    if (a !== b) diffs.push(`${label}\n  ts: ${a.slice(0, 700)}\n  rs: ${b.slice(0, 700)}`)
    return { ts, rs }
  }

  // The scope does not resolve.
  await check('workspaceFiles/list', () => ({ workspaceFileScopeId: 'ghost', path: '.' }), 'list unknown scope')
  await check('workspaceFiles/stat', () => ({ workspaceFileScopeId: 'ghost', path: 'x' }), 'stat unknown scope')
  await check('workspaceFiles/read', () => ({ workspaceFileScopeId: 'ghost', path: 'x' }), 'read unknown scope')

  // The scope resolves, and the path does not exist.
  await check('workspaceFiles/stat', h => ({ workspaceFileScopeId: sessions[h], path: 'no-such-file.txt' }), 'stat missing file')
  await check('workspaceFiles/read', h => ({ workspaceFileScopeId: sessions[h], path: 'no-such-file.txt' }), 'read missing file')

  // The scope resolves, and the path is a directory.
  await check('workspaceFiles/read', h => ({ workspaceFileScopeId: sessions[h], path: '.' }), 'read a directory')

  console.log(`cases ${cases}, differing ${diffs.length}`)
  for (const d of diffs) console.log('DIFF', d)
}